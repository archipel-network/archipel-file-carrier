use std::{path::{Path, PathBuf}, io};

pub struct FileCarrierHierarchy {
    fc: PathBuf,
    root: PathBuf,
    data: PathBuf,
    reaches_file: PathBuf
}

impl FileCarrierHierarchy {
    pub fn new(path: &Path) -> Self {
        Self {
            fc: path.into(),
            root: path.join(".bundles"),
            data: path.join("data"),
            reaches_file: path.join("reaches"),
        }
    }

    pub fn is_file_carrier(path: &Path) -> io::Result<bool> {
        let hierarchy = Self::new(path);
        hierarchy.data.try_exists()
    }
}