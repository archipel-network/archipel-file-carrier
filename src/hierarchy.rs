use std::{path::{Path, PathBuf}, io, fs::{create_dir_all, File}};

pub struct FileCarrierHierarchy {
    fc: PathBuf,
    root: PathBuf,
    data: PathBuf,
    reaches_file: PathBuf,
    connected_file: PathBuf
}

impl FileCarrierHierarchy {
    /// Creates a new [FileCarrierHierarchy]
    /// # Argument
    ///
    /// * `path` - A [&Path] leading to the folder which will contains the `.bundles` directory
    pub fn new(path: &Path) -> Self {
        let root = path.join(".bundles");
        let data = root.join("data");
        let reaches_file = root.join("reaches");
        let connected_file = root.join(".connected");
        Self {
            fc: path.into(),
            root,
            data,
            reaches_file,
            connected_file
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

    pub fn connected_file(&self) -> &Path {
        &self.connected_file
    }

    /// Returns a `Ok(true)` if the [FileCarrierHierarchy] already exists
    pub fn try_exists(&self) -> io::Result<bool> {
        Ok(self.data.try_exists()? && self.root.try_exists()? && self.reaches_file.try_exists()? && self.connected_file.try_exists()?)
    }

    /// Returns a `Ok(true)` if the provided [&Path] points to an existing [FileCarrierHierarchy]
    /// # Argument
    ///
    /// * `path` - A [&Path] to tests the existence of a [FileCarrierHierarchy]
    pub fn folder_is_file_carrier(path: &Path) -> io::Result<bool> {
        let hierarchy = Self::new(path);
        hierarchy.try_exists()
    }

    /// Actually creates the hierarchy in the file system
    pub fn make_hierarchy(&self) -> io::Result<()> {
        create_dir_all(self.data.to_owned())?;
        File::create(self.reaches_file.to_owned())?;
        Ok(())
    }
}