use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    SteamLinux,
    SteamWindows,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::SteamLinux => write!(f, "steam-linux"),
            Platform::SteamWindows => write!(f, "steam-windows"),
        }
    }
}

/// The namespace half of a discovered game's identity (ADR-024).
///
/// The namespace used to ride inside the slug as an `unknown/` prefix, which made
/// the row's id fail the shared slug grammar a manifest `id` must satisfy. It is
/// now its own field, so a plugin that learns a publisher later says so here
/// without touching the slug (ADR-025 owns any second value).
///
/// Unlike [`Platform`], this enum serialises in kebab-case: the design states the
/// mixed spelling of the two closed enums as a deliberate decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Namespace {
    Unknown,
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Namespace::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredGame {
    pub slug: String,
    pub namespace: Namespace,
    pub display_name: String,
    pub install_path: PathBuf,
    pub platform: Platform,
    pub codec_id: Option<String>,
}
