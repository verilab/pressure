//! This module handles entry loading.

use std::{fs, path::PathBuf};

use chrono::{NaiveDate, NaiveDateTime};
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

        #[derive(Deserialize)]
        struct Info {
            site: Site,
            config: Config,
        }

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
        let mut post = load_entry(EntryType::Post, self.posts_folder.join(filename), meta_only)?;
        post.canonicalize_meta(EntryMetaDefaults {
            title: Some(name.split("-").collect::<Vec<&str>>().join(" ")),
            created: Some(
                NaiveDate::from_ymd(year.into(), month.into(), day.into()).and_hms(0, 0, 0),
            ),
            ..Default::default()
        })?;
        Ok(post)
    }

    pub fn load_posts(&self, meta_only: bool) -> PressResult<Vec<Entry>> {
        lazy_static! {
            static ref POST_FILE_NAME_RE: Regex =
                Regex::new(r#"^(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})-(?P<name>.+).md$"#)
                    .unwrap();
        }
        let mut posts: Vec<Entry> = fs::read_dir(&self.posts_folder)?
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
            .collect();
        posts.sort_by(|p1, p2| {
            if p1.created.is_none() && p1.created.is_none() {
                std::cmp::Ordering::Equal
            } else if p1.created.is_none() {
                std::cmp::Ordering::Less
            } else if p2.created.is_none() {
                std::cmp::Ordering::Greater
            } else {
                // the bigger the datetime, the more front it should be
                p2.created.unwrap().cmp(&p1.created.unwrap())
            }
        });
        Ok(posts)
    }

    pub fn load_page<T: Into<PathBuf>>(&self, rel_url: T) -> PressResult<Entry> {
        let mut filepath = self.pages_folder.join(rel_url.into());
        if !filepath.starts_with(&self.pages_folder) {
            return Err(PressError::new("Bad URL"));
        }
        if filepath.is_dir() {
            // e.g. foo/bar/ -> foo/bar/index.md
            filepath.push("index.md");
        } else if filepath.extension().unwrap_or_default() == "html" {
            // e.g. foo/bar.html -> foo/bar.md
            filepath.set_extension("md");
        } else if let Some(filename) = filepath.file_name() {
            // e.g. foo/bar -> foo/bar.md
            let filename = filename.to_owned();
            filepath.set_file_name(filename.to_str().unwrap().to_string() + ".md");
        }
        if filepath.extension().unwrap_or_default() != "md" {
            return Err(PressError::new("Bad page path"));
        }
        let mut page = load_entry(EntryType::Page, &filepath, false)?;
        page.canonicalize_meta(EntryMetaDefaults {
            title: Some(
                filepath
                    .file_stem()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap()
                    .split("-")
                    .collect::<Vec<&str>>()
                    .join(" "),
            ),
            ..Default::default()
        })?;
        Ok(page)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EntryType {
    Post,
    Page,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub etype: EntryType,
    pub filepath: PathBuf,
    pub url: Option<String>,
    pub meta: Yaml,
    pub created: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
    pub content: String,
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            etype: EntryType::Unknown,
            filepath: "".into(),
            url: None,
            meta: Yaml::Hash(yaml::Hash::new()),
            created: None,
            updated: None,
            content: "".into(),
        }
    }
}

struct EntryMetaDefaults {
    title: Option<String>,
    created: Option<NaiveDateTime>,
}

impl Default for EntryMetaDefaults {
    fn default() -> Self {
        Self {
            title: None,
            created: None,
        }
    }
}

impl Entry {
    fn canonicalize_meta(&mut self, defaults: EntryMetaDefaults) -> PressResult<()> {
        // ensure categories and tags are arrays
        for key in vec!["categories", "tags"] {
            if let Some(val) = self.meta.get_mut(key) {
                if let Yaml::String(_) = val {
                    *val = Yaml::Array(vec![val.clone()])
                }
            } else {
                self.meta
                    .as_hash_mut()
                    .unwrap()
                    .insert(Yaml::String(key.to_owned()), Yaml::Array(vec![]));
            }
        }

        // insert default title
        if let None = self.meta.get("title") {
            self.meta.as_hash_mut().unwrap().insert(
                Yaml::String("title".to_string()),
                Yaml::String(defaults.title.unwrap_or_default()),
            );
        }

        // parse created datetime
        if let Some(Yaml::String(dt_str)) = self.meta.get_mut("created") {
            if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S") {
                self.created = Some(dt);
            } else if let Ok(d) = NaiveDate::parse_from_str(dt_str, "%Y-%m-%d") {
                dt_str.push_str(" 00:00:00");
                self.created = Some(d.and_hms(0, 0, 0));
            } else {
                dt_str.clear(); // clear invalid datetime
            }
        } else if let Some(created) = defaults.created {
            self.meta.as_hash_mut().unwrap().insert(
                Yaml::String("created".to_string()),
                Yaml::String(format!("{}", created.format("%Y-%m-%d %H:%M:%S"))),
            );
            self.created = Some(created);
        } else {
            self.meta.as_hash_mut().unwrap().insert(
                Yaml::String("created".to_string()),
                Yaml::String("".to_string()),
            );
        }

        // parse updated datetime
        if let Some(Yaml::String(dt_str)) = self.meta.get_mut("updated") {
            if let Ok(dt) = NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%d %H:%M:%S") {
                self.updated = Some(dt);
            } else if let Ok(d) = NaiveDate::parse_from_str(dt_str, "%Y-%m-%d") {
                dt_str.push_str(" 00:00:00");
                self.updated = Some(d.and_hms(0, 0, 0));
            } else {
                dt_str.clear(); // clear invalid datetime
            }
        } else {
            self.meta.as_hash_mut().unwrap().insert(
                Yaml::String("created".to_string()),
                Yaml::String("".to_string()),
            );
        }

        Ok(())
    }

    pub(crate) fn load_content(&mut self) {
        let entry = load_entry(EntryType::Post, &self.filepath, false).unwrap();
        self.content = entry.content;
    }
}

/// Load a Markdown entry, either a post or a page.
/// If ok, the entry.meta field is guarenteed to be an Yaml::Hash.
fn load_entry<P>(etype: EntryType, filepath: P, meta_only: bool) -> PressResult<Entry>
where
    P: Into<PathBuf>,
{
    let mut entry = Entry {
        etype,
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
            if entry.meta.as_hash().is_none() {
                return Err(PressError::new(
                    "Frontmatter must be a valid YAML hash map.",
                ));
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
        let entry = load_entry(EntryType::Page, "tests/test_inst/pages/test.md", false).unwrap();
        assert!(entry.content.contains("<p>BAZ</p>"));
        assert!(entry.content.contains("<p>FOO BAR!</p>"));
        assert_eq!(entry.meta["title"].as_str().unwrap(), "Foo bar 中文");
        assert_eq!(entry.meta["tags"][0].as_str().unwrap(), "foo");
        assert_eq!(entry.meta["categories"].as_str().unwrap(), "bar");
    }

    #[test]
    fn test_load_entry_meta_only() {
        let entry = load_entry(EntryType::Page, "tests/test_inst/pages/test.md", true).unwrap();
        assert!(entry.content.is_empty());
        assert_eq!(entry.meta["title"].as_str().unwrap(), "Foo bar 中文");
    }

    #[test]
    fn test_load_entry_failed() {
        let res = load_entry(
            EntryType::Page,
            "tests/test_inst/pages/nonexistent.md",
            false,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_load_post_no_content() {
        let inst = Instance::new("tests/test_inst").unwrap();
        let post = inst
            .load_post(2020, 12, 27, "test-no-content", false)
            .unwrap();
        assert_eq!(post.meta["title"].as_str().unwrap(), "test no content");
        assert_eq!(
            post.created.unwrap(),
            NaiveDate::from_ymd(2020, 12, 27).and_hms(0, 0, 0)
        );
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
