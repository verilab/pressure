use std::path::PathBuf;

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
