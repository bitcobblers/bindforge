#!/usr/bin/env bash
# Local test harness for the registry + discovery pair (ADR-026/027 topology).
#
# Starts `registry serve --root --corpus` in the background against a sandboxed
# `XDG_DATA_HOME` under `target/local-registry/`, so the developer's real data
# directory and the shipped `data/catalog/v1/catalog.json` seed both stay
# untouched: the corpus flag points at a copy of the seed inside the sandbox
# (the admin face is a corpus writer), and the discovery CLI's match store
# (`<data>/bindforge/matches.json`) lands in the same sandbox.
#
# Steam posture: with `BINDFORGE_STEAM_WEB_API_KEY` set in the caller's
# environment the instance runs with that key and the store supply live;
# keyless runs use the suites' sentinel-key + planted-snapshot posture, so the
# resolver is active but nothing dials the store. `BINDFORGE_REGISTRY_ADMIN_
# TOKEN` passes through the same way as the key; on the loopback bind that is
# all the admin face needs.
#
# Usage:
#   scripts/local-registry.sh start [--port N]   start the instance (port 8787)
#   scripts/local-registry.sh stop               stop the instance
#   scripts/local-registry.sh status             pid + GET /v1/health answer
#   scripts/local-registry.sh logs [-f]          the instance's log
#   scripts/local-registry.sh discovery [args..] ensure it is up, then run
#                                                `discovery` with the endpoint
#                                                and sandbox already set
#   scripts/local-registry.sh query <key>        one-shot `registry query`
#                                                (takes the data write lock:
#                                                run it while the instance is
#                                                stopped)
#   scripts/local-registry.sh reset              stop, then delete the sandbox
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
sandbox="$repo_root/target/local-registry"
xdg="$sandbox/xdg"                      # the instance's and the CLI's <data>
corpus="$sandbox/catalog-v1.json"       # the sandbox's copy of the seed
pidfile="$sandbox/registry.pid"
portfile="$sandbox/registry.port"
logfile="$sandbox/registry.log"
binary="${CARGO_TARGET_DIR:-$repo_root/target}/debug/registry"
default_port=8787

die() { printf 'error: %s\n' "$1" >&2; exit 1; }

running_pid() {
    # The pid whose instance owns the sandbox, if any.
    [ -f "$pidfile" ] || return 1
    local pid
    pid="$(cat "$pidfile")"
    [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null || return 1
    printf '%s' "$pid"
}

saved_port() { cat "$portfile" 2>/dev/null || printf '%s' "$default_port"; }

# The keyless posture the registry's own suites pin (`hosted_end_to_end`): a
# configured-but-never-used sentinel key with a freshly planted snapshot. The
# resolver is ACTIVE — steam misses answer `200 []` and installed/curated keys
# answer normally instead of aborting discovery with a 503 — while startup
# *loads* the snapshot instead of pulling, so nothing dials the store. The
# refresh cadence is the 24 h `STEAM_SNAPSHOT_REFRESH_SECONDS`; a local
# instance that outlives it lets the refresh thread try the sentinel and fail
# into a warning line in the log, nothing more.
sentinel_key="bindforge-local-dev-sentinel-key"

plant_snapshot() {
    # Rewritten fresh on every keyless start so startup always finds a
    # loadable, in-age snapshot. The handful of real appids below lets a host
    # that has one of them exercise the mint path end to end.
    mkdir -p "$xdg/bindforge/registry"
    printf '{"version":1,"fetched_at":%s,"apps":[{"appid":440,"name":"Team Fortress 2"},{"appid":480,"name":"Left 4 Dead 2"},{"appid":550,"name":"Battlefield Hardline"}]}' \
        "$(date +%s)" >"$xdg/bindforge/registry/steam-snapshot.json"
}

# ADR-027 §Concurrency's runbook step, performed by the operator's script: a
# killed run leaves `write.lock` behind, and clearing a *dead* owner's lock is
# recovery, not adjudication between live writers. A lock stamped with a live
# pid is left alone, so the registry's own named refusal still speaks.
clear_dead_lock() {
    local lock="$xdg/bindforge/registry/write.lock" owner
    [ -f "$lock" ] || return 0
    owner="$(head -n1 "$lock" 2>/dev/null | tr -cd '0-9')"
    if [ -n "$owner" ] && kill -0 "$owner" 2>/dev/null; then
        return 0
    fi
    rm -f "$lock"
    if [ -n "$owner" ]; then
        printf 'cleared the write lock left by dead owner %s\n' "$owner"
    else
        printf 'cleared a write lock with no owner recorded\n'
    fi
}

# One HTTP GET over bash's /dev/tcp, printed raw. The registry speaks plain
# HTTP/1.1 on the serving face, so no client binary is a dependency here.
probe() {
    local port="$1" path="$2"
    (
        exec 3<>"/dev/tcp/127.0.0.1/$port" || exit 1
        printf 'GET %s HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n' "$path" >&3
        head -c 4096 <&3
    ) 2>/dev/null
}

do_start() {
    local port="$default_port"
    while [ $# -gt 0 ]; do
        case "$1" in
            --port) [ $# -ge 2 ] || die "--port needs a value"
                    [[ "$2" =~ ^[0-9]+$ ]] || die "--port $2 is not a port number"
                    port="$2"; shift 2 ;;
            *) die "start takes only --port N, got: $1" ;;
        esac
    done
    if pid="$(running_pid)"; then
        printf 'already running: pid %s at http://127.0.0.1:%s\n' "$pid" "$(saved_port)"
        return 0
    fi
    clear_dead_lock
    mkdir -p "$xdg"
    # The seed corpus is contract data and the admin face may rewrite whatever
    # --corpus names, so the instance never runs against the shipped file.
    [ -e "$corpus" ] || cp "$repo_root/data/catalog/v1/catalog.json" "$corpus"

    cargo build -q -p bindforge-registry

    if [ -n "${BINDFORGE_STEAM_WEB_API_KEY:-}" ]; then
        printf 'steam resolver: ACTIVE with the configured key (a stale snapshot pulls from the store at startup)\n'
    else
        export BINDFORGE_STEAM_WEB_API_KEY="$sentinel_key"
        plant_snapshot
        printf 'steam resolver: ACTIVE, sentinel key + planted snapshot (nothing dials the store; set BINDFORGE_STEAM_WEB_API_KEY for the real one)\n'
    fi

    nohup env XDG_DATA_HOME="$xdg" \
        "$binary" serve --root --corpus "$corpus" --host 127.0.0.1 --port "$port" \
        >>"$logfile" 2>&1 &
    local pid=$!
    printf '%s\n' "$pid" >"$pidfile"
    printf '%s\n' "$port" >"$portfile"

    # Readiness is the health route answering, not just the process surviving:
    # a refused startup is one line and a dead pid, and that line is the log's.
    for _ in $(seq 1 60); do
        if probe "$port" /v1/health >/dev/null; then
            printf 'registry up: pid %s at http://127.0.0.1:%s\n' "$pid" "$port"
            printf '  sandbox data:  %s\n' "$xdg"
            printf '  corpus served: %s\n' "$corpus"
            printf '  log:           %s\n' "$logfile"
            printf '\ndiscovery against it:\n'
            printf '  scripts/local-registry.sh discovery list\n'
            return 0
        fi
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.25
    done
    printf 'error: the registry did not come up on port %s; its log says:\n' "$port" >&2
    tail -n 5 "$logfile" >&2 || true
    rm -f "$pidfile"
    return 1
}

do_stop() {
    pid="$(running_pid)" || { printf 'not running\n'; return 0; }
    kill "$pid"
    for _ in $(seq 1 40); do
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.25
    done
    kill -0 "$pid" 2>/dev/null && { printf 'error: pid %s ignored the stop\n' "$pid" >&2; return 1; }
    rm -f "$pidfile"
    printf 'stopped (the sandbox and its documents are kept; `reset` deletes them)\n'
}

do_status() {
    local port; port="$(saved_port)"
    if pid="$(running_pid)"; then
        printf 'running: pid %s at http://127.0.0.1:%s\n' "$pid" "$port"
        probe "$port" /v1/health || printf '  (no health answer on port %s)\n' "$port"
    else
        printf 'not running\n'
    fi
}

do_discovery() {
    # `query` takes the sandbox's write lock, so discovery is the running
    # instance's only companion; the endpoint and <data> go to the CLI the
    # same way an operator's deployment would set them.
    running_pid >/dev/null || do_start
    local port; port="$(saved_port)"
    local -a args=("$@")
    [ ${#args[@]} -gt 0 ] || args=(list)
    env XDG_DATA_HOME="$xdg" \
        BINDFORGE_CATALOG_URL="http://127.0.0.1:$port" \
        cargo run -q -p discovery-cli -- "${args[@]}"
}

do_query() {
    [ $# -ge 1 ] || die "query needs one lookup key, e.g. query steam/730"
    if running_pid >/dev/null; then
        die "a running instance owns the data write lock: stop it before \`query\`"
    fi
    clear_dead_lock
    mkdir -p "$xdg"
    [ -e "$corpus" ] || cp "$repo_root/data/catalog/v1/catalog.json" "$corpus"
    cargo build -q -p bindforge-registry
    # Same posture as `start`: keyless, the sentinel and a fresh planted
    # snapshot; with a configured key, the real supply.
    if [ -z "${BINDFORGE_STEAM_WEB_API_KEY:-}" ]; then
        export BINDFORGE_STEAM_WEB_API_KEY="$sentinel_key"
        plant_snapshot
    fi
    env XDG_DATA_HOME="$xdg" \
        "$binary" query --root --corpus "$corpus" "$1"
}

do_reset() {
    do_stop
    case "$sandbox" in
        "$repo_root"/target/*) rm -rf "$sandbox" ;;
        *) die "refusing to delete $sandbox: it is not inside the target tree" ;;
    esac
    printf 'sandbox deleted: %s\n' "$sandbox"
}

command="${1:-help}"
[ $# -gt 0 ] && shift
case "$command" in
    start)    do_start "$@" ;;
    stop)     do_stop ;;
    status)   do_status ;;
    logs)     if [ "${1:-}" = "-f" ]; then tail -f "$logfile"
                    else tail -n "${1:-60}" "$logfile"; fi ;;
    discovery) do_discovery "$@" ;;
    query)    do_query "$@" ;;
    reset)    do_reset ;;
    help|-h|--help) sed -n '2,30p' "${BASH_SOURCE[0]}" ;;
    *) die "unknown command \`$command\` (see: scripts/local-registry.sh help)" ;;
esac
