use std::{path::{Path, PathBuf}, io, fs::{create_dir_all, File}};

pub struct FileCarrierHierarchy {
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
        Ok(self.data.try_exists()? && self.root.try_exists()?)
    }

    /// Returns a `Ok(true)` if the provided [&Path] points to an existing [FileCarrierHierarchy]
    /// # Argument
    ///
    /// * `path` - A [&Path] to tests the existence of a [FileCarrierHierarchy]
    pub fn folder_is_file_carrier(path: &Path) -> io::Result<bool> {
        let hierarchy = Self::new(path);
        hierarchy.try_exists()
    }

    /// Creates the hierarchy in the file system
    /// This function will create the hierarchy if it does not exist, and will truncate it if it does.
    pub fn create_hierarchy(&self) -> io::Result<()> {
        create_dir_all(self.data.to_owned())?;
        File::create(self.reaches_file.to_owned())?;
        Ok(())
    }

    /// Creates the hierarchy in the file system
    /// This function will create the hierarchy if it does not exist, and will truncate it if it does.
    pub fn create(path: &Path) -> io::Result<()> {
        let hierarchy = Self::new(path);
        create_dir_all(hierarchy.data.to_owned())?;
        File::create(hierarchy.reaches_file.to_owned())?;
        Ok(())
    }
}