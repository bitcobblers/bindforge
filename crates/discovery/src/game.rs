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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredGame {
    pub slug: String,
    pub display_name: String,
    pub install_path: PathBuf,
    pub platform: Platform,
    pub codec_id: Option<String>,
}
