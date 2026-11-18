use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// A game's launch platform. The serialized form is the kebab-case name
/// `Display` below prints and ADR-022 documents (`steam-linux`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredGame {
    pub slug: String,
    pub display_name: String,
    pub install_path: PathBuf,
    pub platform: Platform,
    pub codec_id: Option<String>,
}
