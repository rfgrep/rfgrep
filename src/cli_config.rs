use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigAction {
    /// Show current configuration
    Show {
        /// Show merged configuration from all sources
        #[arg(long)]
        merged: bool,

        /// Show configuration source for each value
        #[arg(long)]
        sources: bool,

        /// Output format
        #[arg(long, default_value = "toml")]
        format: ConfigFormat,
    },

    /// Initialize config file
    Init {
        /// Location (user, project, or path)
        #[arg(default_value = "user")]
        location: ConfigLocation,

        /// Format
        #[arg(long, default_value = "toml")]
        format: ConfigFormat,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },

    /// Get a configuration value
    Get {
        /// Key in dot notation (e.g., search.mode)
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Key in dot notation
        key: String,

        /// Value
        value: String,

        /// Configuration level
        #[arg(long, default_value = "user")]
        level: ConfigLocation,
    },

    /// Unset a configuration value
    Unset {
        /// Key in dot notation
        key: String,

        /// Configuration level
        #[arg(long, default_value = "user")]
        level: ConfigLocation,
    },

    /// List all configuration keys
    List {
        /// Filter by prefix
        #[arg(long)]
        prefix: Option<String>,
    },

    /// Validate configuration
    Validate {
        /// Path to config file
        path: Option<PathBuf>,
    },

    /// Edit configuration file
    Edit {
        /// Configuration level
        #[arg(default_value = "user")]
        level: ConfigLocation,
    },

    /// Export configuration
    Export {
        /// Output format
        #[arg(long, default_value = "toml")]
        format: ConfigFormat,

        /// Output file (stdout if not specified)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Import configuration
    Import {
        /// Input file
        path: PathBuf,

        /// Configuration level
        #[arg(long, default_value = "user")]
        level: ConfigLocation,

        /// Merge with existing config
        #[arg(long)]
        merge: bool,
    },
}

#[derive(Clone, ValueEnum, Debug)]
pub enum ConfigLocation {
    System,
    User,
    Project,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

use crate::error::{Result as RfgrepResult, RfgrepError};

pub async fn handle_config_action(action: ConfigAction) -> RfgrepResult<()> {
    match action {
        ConfigAction::Show {
            merged,
            sources: _,
            format: _,
        } => {
            if merged {
                let manager = crate::config::ConfigManager::new()?;
                println!("{:#?}", manager.merged_config);
            } else {
                let config = crate::config::Config::load()?;
                println!("{:#?}", config);
            }
            Ok(())
        }
        ConfigAction::Init {
            location,
            format: _,
            force,
        } => {
            let path = match location {
                ConfigLocation::User => dirs::config_dir()
                    .ok_or(RfgrepError::Other("No config directory found".to_string()))?
                    .join("rfgrep/config.toml"),
                ConfigLocation::Project => PathBuf::from(".rfgreprc"),
                ConfigLocation::System => PathBuf::from("/etc/rfgrep/config.toml"),
            };

            if path.exists() && !force {
                println!(
                    "Config file already exists at {:?}. Use --force to overwrite.",
                    path
                );
                return Ok(());
            }

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    RfgrepError::Other(format!("Failed to create directory: {}", e))
                })?;
            }

            let config = crate::config::Config::default();
            let content = toml::to_string_pretty(&config)
                .map_err(|e| RfgrepError::Other(format!("Failed to serialize config: {}", e)))?;
            std::fs::write(&path, content)
                .map_err(|e| RfgrepError::Other(format!("Failed to write config file: {}", e)))?;
            println!("Initialized default configuration at {:?}", path);
            Ok(())
        }
        ConfigAction::Get { key } => {
            let config = crate::config::Config::load()?;
            let json =
                serde_json::to_value(&config).map_err(|e| RfgrepError::Other(e.to_string()))?;

            let mut current = &json;
            for part in key.split('.') {
                if let Some(val) = current.get(part) {
                    current = val;
                } else {
                    return Err(RfgrepError::Other(format!("Key not found: {}", key)));
                }
            }

            if let Some(s) = current.as_str() {
                println!("{}", s);
            } else {
                println!("{}", current);
            }
            Ok(())
        }
        ConfigAction::List { prefix } => {
            let config = crate::config::Config::load()?;
            let json =
                serde_json::to_value(&config).map_err(|e| RfgrepError::Other(e.to_string()))?;

            fn print_keys(val: &serde_json::Value, prefix: &str, filter: Option<&str>) {
                if let Some(obj) = val.as_object() {
                    for (k, v) in obj {
                        let new_key = if prefix.is_empty() {
                            k.clone()
                        } else {
                            format!("{}.{}", prefix, k)
                        };
                        if v.is_object() {
                            print_keys(v, &new_key, filter);
                        } else {
                            if let Some(f) = filter {
                                if !new_key.starts_with(f) {
                                    continue;
                                }
                            }
                            println!("{} = {}", new_key, v);
                        }
                    }
                }
            }

            print_keys(&json, "", prefix.as_deref());
            Ok(())
        }
        _ => {
            println!("Config action not fully implemented yet");
            Ok(())
        }
    }
}
