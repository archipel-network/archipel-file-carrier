use std::{path::Path, time::{Duration, SystemTime}, io::{Read, BufReader}, io::{Write, BufRead}, fs::File};
use ud3tn_aap::{Agent, config::{Contact, ContactDataRate}};

use crate::{hierarchy::FileCarrierHierarchy, error::FileCarrierError};

/// Register a folder to a node
/// # Argument
///
/// * `aap_agent` - A [&mut Agent] to send bundle through
/// * `folder` - The folder [&Path] to register
/// * `duration` - The duration of the connection
pub fn register_folder<S: Read + Write>(aap_agent: &mut Agent<S>, folder: &Path, duration: Duration) -> Result<(), FileCarrierError> {
    let hierarchy = FileCarrierHierarchy::new(&folder);

    if !hierarchy.try_exists()? {
        println!("Folder {} is not a file-carrier", folder.display());
        return Err(FileCarrierError::NotAFileCarrier(folder.to_path_buf()));
    }

    let current_node = aap_agent.node_eid.clone();
    let mut reaches: Vec<String> = Vec::new();

    let file = File::open(hierarchy.reaches_file())?;
    let reader = BufReader::new(file);

    reaches.push(current_node.clone());
    
    for line in reader.lines() {
        let line = line?;
        if line != current_node {
            reaches.push(line);
        }
    }

    if reaches.len() == 1 {
        println!("You're the only one using this file-carrier");
        println!("Connect to another node to establish a connection");
        println!("or manually add node EID's in {}", hierarchy.reaches_file().display())
    }

    let msg = ud3tn_aap::config::ConfigBundle::AddContact {
        eid: reaches[1].clone(),
        reliability: None,
        cla_address: format!("file:{}", hierarchy.data().canonicalize()?.to_str().unwrap()),
        reaches_eid: reaches[1..reaches.len()].to_vec(),
        contacts: vec![Contact { start: SystemTime::now(), end: SystemTime::now() + duration, data_rate: ContactDataRate::Unlimited }],
    };

    let mut reaches_file = File::create(hierarchy.reaches_file())?;

    let reaches_concat = reaches.join("\n");

    reaches_file.write_all(reaches_concat.as_bytes())?;

    let mut connected_file = File::create(hierarchy.connected_file())?;
    connected_file.write_all(reaches[1].as_bytes())?;

    aap_agent.send_bundle(current_node.clone() + "/config", &msg.to_bytes())?;
    
    println!("Connected to node {} for {} seconds", current_node, duration.as_secs());

    Ok(())
}