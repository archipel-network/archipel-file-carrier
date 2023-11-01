use std::{path::Path, io::{self, Write}, fs::File};

use crate::hierarchy::FileCarrierHierarchy;

const README: &str = include_str!("templates/readme.txt");

/// Initialize a File Carrier hierarchy
/// # Arguments
///
/// * `path` - A [&Path] leading to the folder which will contains the `.bundles` directory
pub fn initialize_file_carrier(path: &Path) -> io::Result<()>{
    let hierarchy = FileCarrierHierarchy::new(path);

    if hierarchy.try_exists()? {
        println!("{:?} is already a file carrier", path);
        return Ok(());
    }

    hierarchy.create_hierarchy()?;

    File::create(hierarchy.root().join("readme.txt"))?.write_all(README.as_bytes())?;

    println!("File carrier initialized in {}", path.canonicalize()?.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use crate::init::initialize_file_carrier;

    #[test]
    fn initialize_fc() {
        let current_dir = env::current_dir().unwrap();
        let _ = initialize_file_carrier(&current_dir);

        let bundle_path = current_dir.join(".bundles");
        let data_path = current_dir.join(".bundles/data");
        let reaches_path = current_dir.join(".bundles/reaches");

        
        assert!(bundle_path.try_exists().unwrap() && data_path.try_exists().unwrap() && reaches_path.try_exists().unwrap());
        let _ = fs::remove_dir_all(bundle_path);
    }
}