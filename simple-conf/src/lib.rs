use std::path::{PathBuf, Path};

pub trait SimpleConf {
    fn from_serialized(serialized: &str) -> Self;
    fn from_path(path: &Path) -> Self;
    fn from_path_str(path: &str) -> Self
    where
        Self: Sized
    {
        Self::from_path(Path::new(path))
    }

    fn to_serialized(&self) -> &str;
    fn to_path(&self, path: &Path);
    fn to_path_str(&self, path: &str) {
        self.to_path(Path::new(path));
    }
}