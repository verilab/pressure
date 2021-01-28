use std::{fs, path::PathBuf};

use comrak::{markdown_to_html, ComrakOptions};
use regex::Regex;
use serde::{Deserialize, Serialize};
use yaml_rust::{yaml, Yaml, YamlLoader};

use crate::{PressError, PressResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Site {
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    pub posts_per_index_page: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Info {
    site: Site,
    config: Config,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub root_folder: PathBuf,
    pub static_folder: PathBuf,
    pub template_folder: PathBuf,
    pub theme_static_folder: PathBuf,
    pub posts_folder: PathBuf,
    pub pages_folder: PathBuf,
    pub raw_folder: PathBuf,
    pub site: Site,
    pub config: Config,
}

impl Instance {
    pub fn new<T: Into<PathBuf>>(root_folder: T) -> PressResult<Instance> {
        let root_folder = root_folder.into();
        let static_folder = root_folder.join("static");
        let template_folder = root_folder.join("theme").join("templates");
        let theme_static_folder = root_folder.join("theme").join("static");
        let posts_folder = root_folder.join("posts");
        let pages_folder = root_folder.join("pages");
        let raw_folder = root_folder.join("raw");
        let Info { site, config } =
            toml::from_str(&std::fs::read_to_string(root_folder.join("pressure.toml"))?)?;
        Ok(Instance {
            root_folder,
            static_folder,
            template_folder,
            theme_static_folder,
            posts_folder,
            pages_folder,
            raw_folder,
            site,
            config,
        })
    }

    pub fn load_post(
        &self,
        year: u16,
        month: u8,
        day: u8,
        name: &str,
        meta_only: bool,
    ) -> PressResult<Entry> {
        let filename = format!("{:04}-{:02}-{:02}-{}.md", year, month, day, name);
        let mut post = load_entry(self.posts_folder.join(filename), meta_only)?;
        let meta = post.meta.as_hash_mut().unwrap();
        let title_key = Yaml::String("title".to_string());
        if !meta.contains_key(&title_key) {
            meta.insert(
                title_key,
                Yaml::String(name.split("-").collect::<Vec<&str>>().join(" ")),
            );
        }
        Ok(post)
    }

    pub fn load_posts(&self, meta_only: bool) -> PressResult<Vec<Entry>> {
        lazy_static! {
            static ref POST_FILE_NAME_RE: Regex =
                Regex::new(r#"^(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})-(?P<name>.+).md$"#)
                    .unwrap();
        }
        Ok(fs::read_dir(&self.posts_folder)?
            .filter_map(|dirent| {
                let dirent = dirent.ok()?;
                let filename = dirent.file_name().to_str()?.to_string();
                let caps = POST_FILE_NAME_RE.captures(&filename)?;
                let (year, month, day, name) = (
                    caps["year"].parse::<u16>().unwrap(),
                    caps["month"].parse::<u8>().unwrap(),
                    caps["day"].parse::<u8>().unwrap(),
                    &caps["name"],
                );
                self.load_post(year, month, day, name, meta_only).ok()
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub filepath: PathBuf,
    pub meta: Yaml,
    pub content: String,
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            filepath: "".into(),
            meta: Yaml::Hash(yaml::Hash::new()),
            content: "".into(),
        }
    }
}

impl Entry {
    pub fn load_content(&mut self) {
        let entry = load_entry(&self.filepath, false).unwrap();
        self.content = entry.content;
    }
}

/// Load a Markdown entry, either a post or a page.
/// If ok, the entry.meta field is guarenteed to be an Yaml::Hash.
fn load_entry<P>(filepath: P, meta_only: bool) -> PressResult<Entry>
where
    P: Into<PathBuf>,
{
    let mut entry = Entry {
        filepath: filepath.into().canonicalize()?,
        ..Entry::default()
    };
    let file_content = fs::read_to_string(&entry.filepath)?;
    let lines: Vec<&str> = file_content.lines().collect();
    if lines.len() == 0 {
        return Ok(entry);
    }
    let mut remained = &lines[..];
    if lines[0] == "---" {
        let tmp_lines = &lines[1..];
        if let Some(fm_end) = tmp_lines.iter().position(|&x| x == "---") {
            let front_matter = tmp_lines[..fm_end].join("\n");
            entry.meta = YamlLoader::load_from_str(&front_matter)?[0].clone();
            match entry.meta {
                Yaml::Hash(_) => {}
                _ => {
                    return Err(PressError::new(
                        "Frontmatter must be a valid YAML hash map.",
                    ))
                }
            }

            for key in vec!["categories", "tags"] {
                if let Some(val) = entry.meta.get_mut(key) {
                    if let Yaml::String(_) = val {
                        *val = Yaml::Array(vec![val.clone()])
                    }
                }
            }

            remained = &tmp_lines[fm_end + 1..];
        }
    }
    entry.content = if !meta_only {
        markdown_to_html(remained.join("\n").trim(), &ComrakOptions::default())
    } else {
        "".to_string()
    };
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_instance() {
        let root_folder = PathBuf::from("tests/test_inst").canonicalize().unwrap();
        let inst = Instance::new(&root_folder).unwrap();
        assert_eq!(inst.root_folder, root_folder);
        assert_eq!(inst.site.title, "My Blog");
        assert_eq!(inst.site.subtitle.unwrap(), "Here is my blog.");
        assert_eq!(inst.config.posts_per_index_page, 5);
    }

    #[test]
    fn test_load_entry() {
        let entry = load_entry("tests/test_inst/pages/test.md", false).unwrap();
        assert!(entry.content.contains("<p>BAZ</p>"));
        assert!(entry.content.contains("<p>FOO BAR!</p>"));
        assert_eq!(entry.meta["title"].as_str().unwrap(), "Foo bar 中文");
        assert_eq!(entry.meta["tags"][0].as_str().unwrap(), "foo");
        assert_eq!(entry.meta["categories"][0].as_str().unwrap(), "bar");
    }

    #[test]
    fn test_load_entry_meta_only() {
        let entry = load_entry("tests/test_inst/pages/test.md", true).unwrap();
        assert!(entry.content.is_empty());
        assert_eq!(entry.meta["title"].as_str().unwrap(), "Foo bar 中文");
    }

    #[test]
    fn test_load_entry_failed() {
        let res = load_entry("tests/test_inst/pages/nonexistent.md", false);
        assert!(res.is_err());
    }

    #[test]
    fn test_load_post_no_content() {
        let inst = Instance::new("tests/test_inst").unwrap();
        let post = inst
            .load_post(2020, 12, 27, "test-no-content", false)
            .unwrap();
        assert_eq!(post.meta["title"].as_str().unwrap(), "test no content");
    }

    #[test]
    fn test_load_post() {
        let inst = Instance::new("tests/test_inst").unwrap();
        let post = inst.load_post(2020, 8, 31, "test", false).unwrap();
        assert_eq!(post.meta["title"].as_str().unwrap(), "测试");
        assert_eq!(
            post.meta["categories"].as_vec().unwrap()[0]
                .as_str()
                .unwrap(),
            "Dev"
        );
        assert!(post.content.contains("<h2>喵</h2"));
    }

    #[test]
    fn test_load_posts() {
        let inst = Instance::new("tests/test_inst").unwrap();
        let posts = inst.load_posts(false).unwrap();
        assert_eq!(posts.len(), 2);
        let mut content_count = 0;
        for post in &posts {
            if !post.content.is_empty() {
                content_count += 1;
            }
        }
        assert_ne!(content_count, 0);

        let posts = inst.load_posts(true).unwrap();
        let mut content_count = 0;
        for post in &posts {
            if !post.content.is_empty() {
                content_count += 1;
            }
        }
        assert_eq!(content_count, 0);
    }
}
