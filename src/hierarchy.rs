use std::{path::{Path, PathBuf}, io, fs::{create_dir_all, File}};

pub struct FileCarrierHierarchy {
    fc: PathBuf,
    root: PathBuf,
    data: PathBuf,
    reaches_file: PathBuf
}

impl FileCarrierHierarchy {
    pub fn new(path: &Path) -> Self {
        let root = path.join(".bundles");
        let data = root.join("data");
        let reaches_file = root.join("reaches");
        Self {
            fc: path.into(),
            root,
            data,
            reaches_file,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn data(&self) -> &Path {
        &self.data
    }

    pub fn reaches_file(&self) -> &Path {
        &self.reaches_file
    }

    pub fn is_file_carrier(&self) -> io::Result<bool> {
        Ok(self.data.try_exists()? && self.root.try_exists()? && self.reaches_file.try_exists()?)
    }

    pub fn make_hierarchy(&self) -> io::Result<()> {
        create_dir_all(self.data.to_owned())?;
        File::create(self.reaches_file.to_owned())?;
        Ok(())
    }

    pub fn folder_is_file_carrier(path: &Path) -> io::Result<bool> {
        let hierarchy = Self::new(path);
        hierarchy.is_file_carrier()
    }
}