use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    SteamLinux,
    SteamWindows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredGame {
    pub slug: String,
    pub display_name: String,
    pub install_path: PathBuf,
    pub platform: Platform,
    pub codec_id: Option<String>,
}
