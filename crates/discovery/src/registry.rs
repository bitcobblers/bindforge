#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryError {
    PluginError(String),
    RegistryError(String),
}

/// The error's own voice: the variant name and the message the plugin or the
/// registry reader carried (design 025's suites format the failure with `{}`,
/// and the CLI quotes the `Debug` shape at its named refusal line).
impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PluginError(message) => write!(f, "PluginError: {message}"),
            Self::RegistryError(message) => write!(f, "RegistryError: {message}"),
        }
    }
}

impl std::error::Error for DiscoveryError {}
