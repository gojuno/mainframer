extern crate yaml_rust;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use self::yaml_rust::Yaml;
use self::yaml_rust::YamlLoader;

#[derive(Debug, PartialEq, Eq)]
pub struct IntermediateConfig {
    pub remote: Option<IntermediateRemote>,
    pub push: Option<IntermediatePush>,
    pub pull: Option<IntermediatePull>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntermediateRemote {
    pub host: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntermediatePush {
    pub compression: Option<i64>
}

#[derive(Debug, Eq, PartialEq)]
pub struct IntermediatePull {
    pub compression: Option<i64>
}

impl IntermediateConfig {
    pub fn from_file(file_path: &Path) -> Result<IntermediateConfig, String> {
        let mut content = String::new();

        let mut file = match File::open(file_path) {
            Err(_) => return Err(format!("Could not open config file '{}'", file_path.to_string_lossy())),
            Ok(value) => value,
        };

        file.read_to_string(&mut content)
            .unwrap_or_else(|_| panic!("Could not read config file '{}'", file_path.to_string_lossy()));

        match parse_config_from_str(&content) {
            Err(message) => Err(format!("Error during parsing config file '{}'\n{}", file_path.to_string_lossy(), message)),
            Ok(config) => Ok(config)
        }
    }
}

fn parse_config_from_str(config_content: &str) -> Result<IntermediateConfig, String> {
    let yaml = match YamlLoader::load_from_str(config_content) {
        Err(error) => return Err(format!("YAML parsing error {:#?}", error)),
        Ok(content) => content[0].to_owned()
    };

    let remote = match &yaml["remote"] {
        Yaml::Hash(remote_machine) => {
            let host = match &remote_machine.get(&Yaml::String(String::from("host"))) {
                Some(host) => match host {
                    Yaml::String(host) => Some(host.to_string()),
                    Yaml::Null => None,
                    _ => return Err(String::from("remote.host must be a string"))
                },
                None => None
            };

            Some(IntermediateRemote {
                host,
            })
        }
        Yaml::Null | Yaml::BadValue => None,
        ref something_else => return Err(format!("'remote' must be an object, but was {:#?}", something_else))
    };

    let push = match &yaml["push"] {
        Yaml::Hash(push) => {
            let compression = match push.get(&Yaml::String(String::from("compression"))).cloned() {
                Some(compression) => match compression {
                    Yaml::Integer(compression) => if compression >= 1 && compression <= 9 {
                        Some(compression)
                    } else {
                        return Err(format!("'push.compression' must be a positive integer from 1 to 9, but was {:#?}", compression));
                    },
                    Yaml::Null | Yaml::BadValue => None,
                    ref something_else => return Err(format!("'push.compression' must be a positive integer from 1 to 9, but was {:#?}", something_else))
                },
                None => None
            };

            Some(IntermediatePush {
                compression
            })
        }
        Yaml::Null | Yaml::BadValue => None,
        _ => return Err(String::from("'push' must be an object"))
    };

    let pull = match &yaml["pull"] {
        Yaml::Hash(pull) => {
            let compression = match pull.get(&Yaml::String(String::from("compression"))).cloned() {
                Some(compression) => match compression {
                    Yaml::Integer(compression) => if compression >= 1 && compression <= 9 {
                        Some(compression)
                    } else {
                        return Err(format!("'pull.compression' must be a positive integer from 1 to 9, but was {:#?}", compression));
                    }
                    ,
                    Yaml::Null | Yaml::BadValue => None,
                    ref something_else => return Err(format!("'pull.compression' must be a positive integer from 1 to 9, but was {:#?}", something_else))
                },
                None => None
            };

            Some(IntermediatePull {
                compression
            })
        }
        Yaml::Null | Yaml::BadValue => None,
        _ => return Err(String::from("'pull' must be an object"))
    };

    Ok(IntermediateConfig {
        remote,
        push,
        pull,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_from_str_all_fields_2_spaces_indent() {
        let content = "
remote:
  host: computer1
push:
  compression: 5
pull:
  compression: 2"
        ;
        assert_eq!(parse_config_from_str(content), Ok(IntermediateConfig {
            remote: Some(IntermediateRemote {
                host: Some(String::from("computer1")),
            }),
            push: Some(IntermediatePush {
                compression: Some(5)
            }),
            pull: Some(IntermediatePull {
                compression: Some(2)
            }),
        }));
    }

    #[test]
    fn parse_config_from_str_all_fields_strings_in_quotes() {
        let content = "
remote:
  host: \"computer1\"
push:
  compression: 5
pull:
  compression: 2"
        ;
        assert_eq!(parse_config_from_str(content), Ok(IntermediateConfig {
            remote: Some(IntermediateRemote {
                host: Some(String::from("computer1")),
            }),
            push: Some(IntermediatePush {
                compression: Some(5)
            }),
            pull: Some(IntermediatePull {
                compression: Some(2)
            }),
        }));
    }

    #[test]
    fn parse_config_from_str_all_fields_4_spaces_indent() {
        let content = "
remote:
    host: computer1
push:
    compression: 5
pull:
    compression: 2"
        ;
        assert_eq!(parse_config_from_str(content), Ok(IntermediateConfig {
            remote: Some(IntermediateRemote {
                host: Some(String::from("computer1")),
            }),
            push: Some(IntermediatePush {
                compression: Some(5)
            }),
            pull: Some(IntermediatePull {
                compression: Some(2)
            }),
        }));
    }

    #[test]
    fn parse_config_from_str_only_remote_machine_host() {
        let content = "
remote:
  host: computer1
";
        assert_eq!(parse_config_from_str(content), Ok(IntermediateConfig {
            remote: Some(IntermediateRemote {
                host: Some(String::from("computer1")),
            }),
            push: None,
            pull: None,
        }));
    }

    #[test]
    fn parse_config_from_str_only_push_compression() {
        let content = "
push:
  compression: 5
";
        assert_eq!(parse_config_from_str(content), Ok(IntermediateConfig {
            remote: None,
            push: Some(IntermediatePush {
                compression: Some(5)
            }),
            pull: None,
        }));
    }

    #[test]
    fn parse_config_from_str_only_pull_compression() {
        let content = "
pull:
  compression: 2
";
        assert_eq!(parse_config_from_str(content), Ok(IntermediateConfig {
            remote: None,
            push: None,
            pull: Some(IntermediatePull {
                compression: Some(2)
            }),
        }));
    }

    #[test]
    fn parse_config_from_str_compression_valid_range() {
        let mut destinations: Vec<String> = Vec::new();

        destinations.push(String::from("push"));
        destinations.push(String::from("pull"));

        for destination in destinations {
            for compression_level in 1..9 {
                let content = format!("
{:#?}:
  compression: {:#?}
", destination, compression_level);

                assert_eq!(parse_config_from_str(&content), Ok(IntermediateConfig {
                    remote: None,
                    push: if destination == "push" {
                        Some(IntermediatePush {
                            compression: Some(compression_level),
                        })
                    } else {
                        None
                    },
                    pull: if destination == "pull" {
                        Some(IntermediatePull {
                            compression: Some(compression_level),
                        })
                    } else {
                        None
                    },
                }));
            }
        }
    }

    #[test]
    fn parse_config_from_str_compression_invalid_range() {
        let mut destinations: Vec<String> = Vec::new();

        destinations.push(String::from("push"));
        destinations.push(String::from("pull"));

        let mut invalid_compression_levels: Vec<i64> = Vec::new();

        invalid_compression_levels.push(0);
        invalid_compression_levels.push(10);
        invalid_compression_levels.push(-1);

        for destination in destinations {
            for compression_level in &invalid_compression_levels {
                let content = format!("
{:#?}:
  compression: {:#?}
", destination, compression_level);

                assert_eq!(
                    parse_config_from_str(&content),
                    Err(format!("'{}.compression' must be a positive integer from 1 to 9, but was {}", destination, compression_level))
                );
            }
        }
    }

    #[test]
    fn parse_config_from_str_push_compression_not_an_integer() {
        let content = "
push:
  compression: yooo
";
        assert_eq!(parse_config_from_str(content), Err(String::from("'push.compression\' must be a positive integer from 1 to 9, but was String(\n    \"yooo\"\n)")));
    }

    #[test]
    fn parse_config_from_str_pull_compression_remote_not_an_integer() {
        let content = "
pull:
  compression: yooo
";
        assert_eq!(parse_config_from_str(content), Err(String::from("'pull.compression\' must be a positive integer from 1 to 9, but was String(\n    \"yooo\"\n)")));
    }
}
