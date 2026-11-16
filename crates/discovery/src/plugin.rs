use crate::game::DiscoveredGame;
use crate::registry::DiscoveryError;

pub trait DiscoveryPlugin {
    fn name(&self) -> &'static str;
    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError>;
}
