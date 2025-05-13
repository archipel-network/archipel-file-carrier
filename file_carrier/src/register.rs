use std::{path::Path, time::{Duration, SystemTime}, io::BufReader, io::{Write, BufRead}, fs::File};
use ud3tn_aap::{Agent, config::{Contact, ContactDataRate}};

use crate::{hierarchy::FileCarrierHierarchy, error::FileCarrierError};

/// Register a folder to a node
/// # Argument
///
/// * `aap_agent` - A [&mut Agent] to send bundle through
/// * `folder` - The folder [&Path] to register
/// * `duration` - The duration of the connection
/// 
/// Returns Node Id of contact
pub fn register_folder(aap_agent: &mut Agent, folder: &Path, duration: Duration) -> Result<String, FileCarrierError> {
    let hierarchy = FileCarrierHierarchy::new(&folder);
    
    if !hierarchy.try_exists()? {
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

    if reaches.len() <= 1 {
        println!("You're the only one using this file-carrier");
        println!("Connect to another node to establish a connection");
        println!("or manually add node EID's in {}", hierarchy.reaches_file().display());
    }
    else {
        let msg = ud3tn_aap::config::ConfigBundle::AddContact {
            eid: reaches[1].clone(),
            reliability: None,
            cla_address: format!("file:{}", hierarchy.data().canonicalize()?.to_str().unwrap()),
            reaches_eid: reaches[1..reaches.len()].to_vec(),
            contacts: vec![Contact { start: SystemTime::now(), end: SystemTime::now() + duration, data_rate: ContactDataRate::Unlimited }],
        };

        aap_agent.send_config(msg)?;

        println!("Connected to node {} for {} seconds", reaches[1], duration.as_secs());
        println!("Reaches are: {}", reaches[1..].join(";"));

        let mut connected_file = File::create(hierarchy.connected_file())?;
        connected_file.write_all(reaches[1].as_bytes())?;
    }
    
    let mut reaches_file = File::create(hierarchy.reaches_file())?;
    
    let reaches_concat = reaches.join("\n");
    
    reaches_file.write_all(reaches_concat.as_bytes())?;
    
    match reaches.get(1) {
        Some(eid) => Ok(eid.clone()),
        None => Err(FileCarrierError::FirstUser(folder.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::Path, time::Duration};
    use crate::{init::initialize_file_carrier, hierarchy::FileCarrierHierarchy, register::register_folder};

    #[test]
    fn initialize_fc() {
        let current_dir = env::current_dir().unwrap();
        let fc = FileCarrierHierarchy::new(&current_dir);
        let _ = initialize_file_carrier(&current_dir);

        let mut agent = ud3tn_aap::Agent::connect_unix(
            Path::new("/run/archipel-core/archipel-core.socket"),
            "file-carrier/105b4168-a93e-459b-8672-09509db759cc".to_owned(),
        )
        .expect("u3dtn agent connection failure");

        let res = register_folder(&mut agent, &current_dir, Duration::from_secs(300));
        
        let _ = fs::remove_dir_all(fc.root());

        res.expect("Failed at registration");
    }
}