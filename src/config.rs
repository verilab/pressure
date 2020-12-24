use serde::{Deserialize, Serialize};

use crate::{Instance, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub site: SiteConfig,
    pub web: WebConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebConfig {
    pub posts_per_page_on_index: u32,
}

pub fn load_config(instance: &Instance) -> Result<Config> {
    let config_str = std::fs::read_to_string(instance.root_folder.join("config.toml"))?;
    Ok(toml::from_str(&config_str)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let instance = Instance::new("tests/test_inst");
        println!("{}", instance.root_folder.to_str().unwrap());
        let config = load_config(&instance).unwrap();
        assert_eq!(config.site.title, "My Blog");
        assert!(config.site.subtitle.is_some());
        assert_eq!(config.site.subtitle.unwrap(), "Here is my blog.");
        assert_eq!(config.web.posts_per_page_on_index, 5);
    }
}
