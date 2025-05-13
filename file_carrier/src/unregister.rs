use std::{path::Path, io::Read, fs::{File, self}};
use ud3tn_aap::Agent;

use crate::{hierarchy::FileCarrierHierarchy, error::FileCarrierError};

/// Unregister a folder from a node
/// # Argument
///
/// * `aap_agent` - A [&mut Agent] to send bundle through
/// * `folder` - The folder [&Path] to unregister
pub fn unregister_folder(aap_agent: &mut Agent, folder: &Path) -> Result<(), FileCarrierError> {
    let hierarchy = FileCarrierHierarchy::new(&folder);

    if !hierarchy.try_exists()? {
        println!("Folder {} is not a file-carrier", folder.display());
        return Err(FileCarrierError::NotAFileCarrier(folder.to_path_buf()));
    }

    let mut connected_eid = String::new();
    File::open(hierarchy.connected_file())?.read_to_string(&mut connected_eid)?;
    
    let msg = ud3tn_aap::config::ConfigBundle::DeleteContact(connected_eid);
    aap_agent.send_config(msg)?;

    fs::remove_file(hierarchy.connected_file())?;
    
    println!("Unregistered {}", folder.display());

    Ok(())
}