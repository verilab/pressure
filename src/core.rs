use std::{fs, path::PathBuf};

use yaml_rust::{Yaml, YamlLoader};

use crate::Result;

#[derive(Clone)]
pub struct Instance {
    pub root_folder: PathBuf,
    pub static_folder: PathBuf,
    pub template_folder: PathBuf,
    pub theme_static_folder: PathBuf,
    pub posts_folder: PathBuf,
    pub pages_folder: PathBuf,
    pub raw_folder: PathBuf,
}

impl Instance {
    pub fn new<T: Into<PathBuf>>(root_folder: T) -> Instance {
        let root_folder = root_folder.into();
        let static_folder = root_folder.join("static");
        let template_folder = root_folder.join("theme").join("templates");
        let theme_static_folder = root_folder.join("theme").join("static");
        let posts_folder = root_folder.join("posts");
        let pages_folder = root_folder.join("pages");
        let raw_folder = root_folder.join("raw");
        Instance {
            root_folder,
            static_folder,
            template_folder,
            theme_static_folder,
            posts_folder,
            pages_folder,
            raw_folder,
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub meta: Option<Yaml>,
    pub content: String,
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            meta: None,
            content: "".to_string(),
        }
    }
}

fn load_entry(fullpath: &str, meta_only: bool) -> Result<Entry> {
    let file_content = fs::read_to_string(fullpath)?;
    let lines: Vec<&str> = file_content.lines().collect();
    if lines.len() == 0 {
        return Ok(Entry::default());
    }
    let mut meta = None;
    let mut content = "".to_string();
    let mut remained = &lines[..];
    if lines[0] == "---" {
        if let Some(fm_end) = lines[1..].iter().position(|&x| x == "---") {
            let front_matter = lines[1..][0..fm_end].join("\n");
            meta = Some(YamlLoader::load_from_str(&front_matter)?[0].clone());
            remained = &lines[1..][fm_end + 1..];
        }
    }
    if !meta_only {
        content = remained.join("\n");
    }
    Ok(Entry { meta, content })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_entry() {
        let entry = load_entry("tests/test_inst/pages/wow.md", false).unwrap();
        println!("{:?}", entry);
    }
}
