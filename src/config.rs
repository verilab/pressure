use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Result;

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

impl Config {
    pub fn from_str(config_str: &str) -> Result<Config> {
        Ok(toml::from_str(&config_str)?)
    }

    pub fn load<T: Into<PathBuf>>(config_path: T) -> Result<Config> {
        let config_str = std::fs::read_to_string(config_path.into())?;
        Self::from_str(&config_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_str() {
        let config_str = "
[site]
title = \"BLOG\"
subtitle = \"SOME BLOG.\"
author = \"NAME\"
timezone = \"Asia/Shanghai\"

[web]
posts_per_page_on_index = 10
        ";
        let config = Config::from_str(config_str).unwrap();
        assert_eq!(config.site.title, "BLOG");
        assert_eq!(config.site.author.unwrap(), "NAME");
        assert_eq!(config.web.posts_per_page_on_index, 10);
    }

    #[test]
    fn test_load_config() {
        let config = Config::load("tests/test_inst/config.toml").unwrap();
        assert_eq!(config.site.title, "My Blog");
        assert!(config.site.subtitle.is_some());
        assert_eq!(config.site.subtitle.unwrap(), "Here is my blog.");
        assert_eq!(config.web.posts_per_page_on_index, 5);
    }
}
