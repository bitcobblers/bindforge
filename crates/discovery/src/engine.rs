use crate::game::DiscoveredGame;
use crate::plugin::DiscoveryPlugin;
use crate::registry::DiscoveryError;

pub struct DiscoveryEngine {
    plugins: Vec<Box<dyn DiscoveryPlugin>>,
}

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn DiscoveryPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn discover_all(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
        let mut all_games = Vec::new();
        for plugin in &self.plugins {
            match plugin.discover() {
                Ok(mut games) => all_games.append(&mut games),
                Err(e) => {
                    // We don't want one plugin failing to stop everything.
                    // In a real implementation, we might want to collect errors too.
                    eprintln!("Error from plugin {}: {:?}", plugin.name(), e);
                }
            }
        }
        Ok(all_games)
    }
}

impl Default for DiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}
