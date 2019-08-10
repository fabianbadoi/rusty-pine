use serde::{Deserialize, Serialize};

use crate::cache::{make_config, Cache, DefaultCache};
use std::cell::RefCell;
use std::ffi::OsString;
use std::path::Path;
use log::info;

pub fn get_config_file_name(file: &str) -> OsString {
    let path = Path::new(&std::env::var("HOME").unwrap())
        .join(".config/rusty-pine")
        .join(file);

    path.into()
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            user: "root".to_owned(),
            password: "<password>".to_owned(),
            host: "localhost".to_owned(),
            port: 3306,
        }
    }
}

pub fn read() -> Config {
    FileProvider::new(&Path::new(&get_config_file_name("config.json"))).get()
}

pub trait ConfigProvider {
    fn get(&self) -> Config;
}

pub struct FileProvider {
    store: RefCell<DefaultCache>,
    tag: String,
    file_path: OsString,
}

impl FileProvider {
    pub fn new(path: &Path) -> FileProvider {
        let base_dir = path
            .parent()
            .expect(&format!("Invalid path specified: {:?}", path));
        let tag = path
            .file_name()
            .expect(&format!("Invalid path specified: {:?}", path))
            .to_str()
            .unwrap()
            .to_owned();

        let store = RefCell::new(make_config(&base_dir));

        FileProvider {
            store,
            tag,
            file_path: OsString::from(path),
        }
    }

    fn create_default_config(&self) -> Config {
        info!("Setting up default config");

        if (&*self.store.borrow() as &dyn Cache<Config>).has(&self.tag) {
            panic!("Invalid config file is already present at {:?}, please fix or remove it");
        }

        let default_config = Default::default();
        self.store.borrow_mut().set(&self.tag, &default_config);

        println!("Created config file at {:?}", self.file_path);

        default_config
    }
}

impl ConfigProvider for FileProvider {
    fn get(&self) -> Config {
        info!("Reading config from file: {:?}", self.file_path);

        let from_file = self.store.borrow().get(&self.tag);

        match from_file {
            Some(config) => config,
            None => self.create_default_config(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provides_default_if_not_present() {
        let path = std::env::temp_dir()
            .join("rusty-pine-tests")
            .join("config")
            .join("config.json");
        let _makeing_sure_file_doesnt_exist = std::fs::remove_file(&path);

        let provider = FileProvider::new(&path);

        provider.get();

        let default_file_was_created = path.exists();
        assert!(default_file_was_created);
    }

    #[test]
    #[should_panic]
    fn does_not_override_invalid_config_files() {
        let path = std::env::temp_dir()
            .join("rusty-pine-tests")
            .join("config")
            .join("config2.json");

        let _makeing_sure_file_doesnt_exist = std::fs::remove_file(&path);
        {
            std::fs::File::create(&path).unwrap(); // empty file now exists
        }

        let provider = FileProvider::new(&path);

        provider.get();
    }
}
