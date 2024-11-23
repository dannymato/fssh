use crate::ssh_config::config::SSHConfig;

#[test]
fn test_parsing_and_saving_config() -> anyhow::Result<()> {
    let test_values = "key value\n\n\n\n      host value \"value 2\"";

    let config = SSHConfig::from_string(test_values)?;

    assert_eq!(
        config.global_config.other_options.get("key"),
        Some(vec!["value".to_string()].as_ref())
    );

    match config.host_specific_config.get("value") {
        Some(host_config) => {
            assert_eq!(host_config.host, Some("value".to_string()));
        }
        None => assert!(false, "No host-specific config for \"value\""),
    }
    Ok(())
}

#[test]
fn test_parsing_host_and_hostname() -> anyhow::Result<()> {
    let test_values = "host ahost\nhostname yourhostname";

    let config = SSHConfig::from_string(test_values)?;

    match config.host_specific_config.get("ahost") {
        Some(host_config) => {
            assert_eq!(host_config.host, Some("ahost".to_string()));
            assert_eq!(host_config.hostname, Some("yourhostname".to_string()));
        }
        None => assert!(false, "No host-specific config for \"value\""),
    }
    Ok(())
}
