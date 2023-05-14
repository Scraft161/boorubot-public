// Get config from `config.toml`

use std::path::Path;
use std::fs;

use serde::Deserialize;

lazy_static!{
    static ref CONFIG_LOCATIONS: Vec<String> = vec!["./config.toml".to_string(), "$HOME/.config/boorubot.toml".to_string(), "$HOME/.config/boorubot/config.toml".to_string()];
    static ref DEFAULT_CONFIG: Config = Config {
        token: None,
        booru_config: BooruConfig {
            blacklist: Vec::new(),
            force_block: Vec::new(),
            nsfw_tags: Vec::new(),
        }
    };
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: Option<String>,
    pub booru_config: BooruConfig,
}

#[derive(Deserialize, Debug)]
pub struct BooruConfig {
    /// List of tags to reject
    pub blacklist: Vec<String>,
    /// List of tags to alwlays exclude; tags in this list will also get rejected
    pub force_block: Vec<String>,
    /// List of tags for NSFW channels, in SFW whannels these will be excluded from results and
    /// when entered by the user will get rejected.
    pub nsfw_tags: Vec<String>,
}

pub fn read() -> Config {
    let mut config: Option<Config> = None;

    // Read config from file
    for config_file in &*CONFIG_LOCATIONS {
        if Path::new(&config_file).exists() {

            config = match toml::from_str(&fs::read_to_string(config_file).unwrap()) {
                Ok(data) => Some(data),
                Err(why) => panic!("Could not parse config file: {}", why),
            };

            break;
        }
    }

    // Read config from env
    //todo!();

    if let Some(data) = config {
        data
    } else {
        panic!("No config found!");
    }
}

//fn config_from_env() {}
