use serde::{Deserialize, Serialize};

use crate::error::PineError;
use std::path::Path;
use std::ffi::OsString;
use crate::cache::{DefaultCache, make_config, Cache};
use std::cell::RefCell;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
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
          let base_dir = path.parent().expect(&format!("Invalid path specified: {:?}", path));
          let tag = path.file_name().expect(&format!("Invalid path specified: {:?}", path)).to_str().unwrap().to_owned();

          let store = RefCell::new(make_config(&base_dir));

          FileProvider {
              store,
              tag,
              file_path: OsString::from(path),
          }
      }

    fn create_default_config(&self) -> Config
    {
        let default_config = Default::default();
        // TODO: don't override on read errors
        self.store.borrow_mut().set(&self.tag, &default_config);

        println!("Created config file at {:?}", self.file_path);

        default_config
    }
}

impl ConfigProvider for FileProvider {
    fn get(&self) -> Config {
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
    use crate::cache::{ByteFileCache, SerializedCache};

    #[test]
    fn test_provides_default_if_not_present() {
        let path = std::env::temp_dir()
            .join("rusty-pine-tests")
            .join("config")
            .join("connection.json");
        let _makeing_sure_file_doesnt_exist = std::fs::remove_file(&path);

        let provider = FileProvider::new(&path);

        provider.get();

        let default_file_was_created = path.exists();
        assert!(default_file_was_created);
    }
}
