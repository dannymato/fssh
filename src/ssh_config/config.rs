use std::collections::BTreeMap;

use crate::ssh_config::parser::{Keyword, ssh_config_value_parser};
#[derive(Debug, Clone)]
pub struct SSHConfigValues {
    pub host: Option<String>,
    pub hostname: Option<String>,
    pub other_options: BTreeMap<String, Vec<String>>,
}

impl SSHConfigValues {
    fn new() -> Self {
        SSHConfigValues {
            host: None,
            hostname: None,
            other_options: BTreeMap::new(),
        }
    }

    fn with_host(host: String) -> Self {
        SSHConfigValues {
            host: Some(host),
            hostname: None,
            other_options: BTreeMap::new(),
        }
    }

    fn insert_config(&mut self, key: String, arguments: Vec<String>) {
        self.other_options.insert(key, arguments);
    }
}

#[derive(Debug, Clone)]
pub struct SSHConfig {
    pub global_config: SSHConfigValues,
    pub host_specific_config: BTreeMap<String, SSHConfigValues>,
}

impl SSHConfig {
    pub fn from_string(str: &str) -> anyhow::Result<Self> {
        let (_, ssh_key_values)= ssh_config_value_parser::<()>(str)?;
        Ok(Self::from_parser(ssh_key_values))
    }

    fn from_parser(mut parsed_key_values: Vec<(Keyword, Vec<&str>)>) -> Self {
        let mut config = SSHConfig {
            global_config: SSHConfigValues::new(),
            host_specific_config: BTreeMap::new(),
        };

        let mut current_config = &mut config.global_config;

        for (keyword, arguments) in parsed_key_values.drain(..) {
            if keyword == Keyword::Host {
                current_config = config
                    .host_specific_config
                    .entry(arguments[0].to_string())
                    .or_insert(SSHConfigValues::with_host(arguments[0].to_string()));
                continue;
            }
            if keyword == Keyword::Hostname {
                current_config.hostname = Some(arguments[0].to_string());
            }

            current_config.insert_config(
                keyword.into(),
                arguments.iter().map(|s| s.to_string()).collect(),
            );
        }

        config
    }
}
