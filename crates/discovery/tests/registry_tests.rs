use discovery::registry::DiscoveryError;

#[test]
fn test_discovery_error_equality() {
    let err1 = DiscoveryError::PluginError("error".to_string());
    let err2 = DiscoveryError::PluginError("error".to_string());
    let err3 = DiscoveryError::RegistryError("error".to_string());

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
