use clap::{Parser, Subcommand};
use file_carrier::{error::FileCarrierError, init::initialize_file_carrier, register::register_folder, unregister::unregister_folder};
use std::{
    path::PathBuf, process, time::Duration
};
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a file carrier hierarchy into the provided folder
    Init {
        #[arg(default_value = ".")]
        folder: PathBuf,
    },
    /// Registers a folder to a node through an AAP agent connection. Duration is in seconds
    Register {
        #[arg(short, long, default_value = "/run/archipel-core/archipel-core.socket")]
        socket: PathBuf,
        #[arg(default_value = ".")]
        folder: PathBuf,
        #[arg(short, long, default_value_t = 300)]
        duration: u64,
    },
    /// Unregister a folder from a node through an AAP agent
    Unregister {
        #[arg(short, long, default_value = "/run/archipel-core/archipel-core.socket")]
        socket: PathBuf,
        #[arg(default_value = ".")]
        folder: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { folder } => {
            if let Err(e) = initialize_file_carrier(&folder) {
                eprintln!("Failed to initialize folder: {e}");
                process::exit(11);
            }
        }
        Commands::Register {
            socket,
            folder,
            duration,
        } => {
            let mut agent = match ud3tn_aap::Agent::connect_unix(
                &socket, format!("file-carrier/{}", &Uuid::new_v4().to_string())
            ) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Failed to connect to node: {e}");
                    process::exit(10);
                }
            };

            match register_folder(&mut agent, folder, Duration::from_secs(*duration)) {
                Ok(_) => {},
                Err(FileCarrierError::FirstUser(_)) => { process::exit(1) }
                Err(FileCarrierError::NotAFileCarrier(folder)) => { 
                    eprintln!("Folder {} is not a file carrier (it does not contains a .bundles folder)", folder.to_string_lossy());
                    process::exit(2)
                }
                Err(e) => {
                    eprintln!("Failed to register folder: {e}");
                    process::exit(11);
                }
            };
        }
        Commands::Unregister { socket, folder } => {

            let mut agent = match ud3tn_aap::Agent::connect_unix(
                &socket, format!("file-carrier/{}", &Uuid::new_v4().to_string())
            ) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Failed to connect to node: {e}");
                    process::exit(10);
                }
            };

            match unregister_folder(&mut agent, folder) {
                Ok(_) => {}
                Err(FileCarrierError::FirstUser(_)) => { process::exit(1) }
                Err(FileCarrierError::NotAFileCarrier(folder)) => { 
                    eprintln!("Folder {} is not a file carrier (it does not contains a .bundles folder)", folder.to_string_lossy());
                    process::exit(2)
                }
                Err(e) => {
                    eprintln!("Failed to unregister folder: {e}");
                    process::exit(11);
                }
            }
        }
    }
}
