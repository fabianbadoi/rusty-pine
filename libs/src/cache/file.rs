use super::Cache;
use std::convert::TryInto;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct ByteFileCache {
    base_dir: OsString,
}

impl Cache<Vec<u8>> for ByteFileCache {
    fn get(&self, tag: &str) -> Option<Vec<u8>> {
        let file = File::open(self.get_path(tag));

        if file.is_err() {
            return None;
        }
        let mut file = file.unwrap();

        let mut buffer = Vec::with_capacity(file.metadata().unwrap().len().try_into().unwrap());
        let read_result = file.read_to_end(&mut buffer);

        match read_result {
            Ok(_) => Some(buffer),
            Err(_) => None,
        }
    }

    fn set(&mut self, tag: &str, data: &Vec<u8>) {
        Self::ensure_dir_exists(&self.base_dir);

        let path = self.get_path(tag);
        let mut file =
            File::create(path.clone()).expect(&format!("could not open file: {:?}", path.clone()));

        let _ = file
            .write_all(data)
            .expect(&format!("could not write to file: {:?}", path));
    }

    fn clear(&mut self) {
        let _result = std::fs::remove_dir_all(self.base_dir.clone());
    }
}

impl ByteFileCache {
    pub fn new(dir_path: OsString) -> ByteFileCache {
        Self::ensure_dir_exists(&dir_path);

        ByteFileCache { base_dir: dir_path }
    }

    fn get_path(&self, tag: &str) -> PathBuf {
        if tag.contains('/') {
            panic!("Tags should not contains '/'");
        }

        Path::new(&self.base_dir).to_path_buf().join(tag)
    }

    fn ensure_dir_exists(path: &OsString) {
        std::fs::create_dir_all(path.clone())
            .expect(&format!("Could not write to dir: {:?}", path));
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn can_write_to_files() {
        let base_dir = std::env::temp_dir()
            .join("rusty-pine-tests")
            .join("file-cache")
            .into_os_string();
        let mut cache = ByteFileCache::new(base_dir);

        cache.set("test", &b"some data".to_vec());
        let read_data = cache.get("test");

        assert!(read_data.is_some());
        assert_eq!(b"some data", read_data.unwrap().as_slice());
    }

    #[test]
    #[should_panic]
    fn panics_on_bad_tags() {
        let base_dir = std::env::temp_dir()
            .join("rusty-pine-tests")
            .join("file-cache")
            .into_os_string();
        let mut cache = ByteFileCache::new(base_dir);

        cache.set("../test", &b"some data".to_vec());
    }
}
