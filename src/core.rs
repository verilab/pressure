use std::{fs, path::PathBuf};

use yaml_rust::{yaml, Yaml, YamlLoader};

use crate::{Config, Error, Result};

#[derive(Clone)]
pub struct Instance {
    pub root_folder: PathBuf,
    pub static_folder: PathBuf,
    pub template_folder: PathBuf,
    pub theme_static_folder: PathBuf,
    pub posts_folder: PathBuf,
    pub pages_folder: PathBuf,
    pub raw_folder: PathBuf,
    pub config: Config,
}

impl Instance {
    pub fn new<T: Into<PathBuf>>(root_folder: T) -> Result<Instance> {
        let root_folder = root_folder.into();
        let static_folder = root_folder.join("static");
        let template_folder = root_folder.join("theme").join("templates");
        let theme_static_folder = root_folder.join("theme").join("static");
        let posts_folder = root_folder.join("posts");
        let pages_folder = root_folder.join("pages");
        let raw_folder = root_folder.join("raw");
        let config = Config::load(root_folder.join("config.toml"))?;
        Ok(Instance {
            root_folder,
            static_folder,
            template_folder,
            theme_static_folder,
            posts_folder,
            pages_folder,
            raw_folder,
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
    ) -> Result<Entry> {
        let filename = format!("{:04}-{:02}-{:2}-{}.md", year, month, day, name);
        let mut post = load_entry(self.posts_folder.join(filename), meta_only)?;
        if let Yaml::Hash(meta_hash) = &mut post.meta {
            let title_key = Yaml::String("title".to_string());
            if !meta_hash.contains_key(&title_key) {
                meta_hash.insert(
                    title_key,
                    Yaml::String(name.split("-").collect::<Vec<&str>>().join(" ")),
                );
            }
        }
        Ok(post)
    }
}

#[derive(Debug)]
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

fn load_entry<P>(filepath: P, meta_only: bool) -> Result<Entry>
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
                _ => return Err(Error::new("Frontmatter must be a valid YAML hash map.")),
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
        remained.join("\n").trim().to_string()
    } else {
        "".to_string()
    };
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_entry() {
        let entry = load_entry("tests/test_inst/pages/test.md", false).unwrap();
        assert_eq!(entry.content, "BAZ\n\nFOO BAR!");
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
}
