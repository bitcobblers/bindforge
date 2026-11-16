#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryError {
    PluginError(String),
    RegistryError(String),
}
