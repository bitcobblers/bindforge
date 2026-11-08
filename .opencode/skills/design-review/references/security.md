# Security checklist

1. Trust boundaries: every data source classified — user, network, or
   shared binding file from elsewhere. Shared binding files are UNTRUSTED.
2. Parsing untrusted data: size limits, schema validation, no code
   evaluation, no decompression bombs.
3. Path handling: canonicalized; reject `..` and absolute paths from
   user-controlled input; never interpolate paths into shell strings.
4. Secrets and PII: where stored, what may be logged, what leaves the
   machine when bindings are shared.
5. Every dependency named in the design: webfetch its advisories/CVEs;
   a known unfixed advisory is a blocker.
6. Privileges the design needs (device access, filesystem): least
   privilege?
7. Malformed/hostile input must fail closed: reject cleanly, never crash
   or partially apply.
