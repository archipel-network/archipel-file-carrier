use clap::{Parser, Subcommand};
use file_carrier::{init::initialize_file_carrier, register::register_folder, unregister::unregister_folder};
use std::{
    os::unix::net::UnixStream,
    path::PathBuf,
    time::Duration,
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
    /// Unegisters a folder from a node through an AAP agent
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
            initialize_file_carrier(&folder).expect("Initialization failure")
        }
        Commands::Register {
            socket,
            folder,
            duration,
        } => {
            let mut agent = ud3tn_aap::Agent::connect(
                UnixStream::connect(socket).expect("Unix stream connection failure"),
                "file-carrier/".to_owned() + &Uuid::new_v4().to_string(),
            )
            .expect("u3dtn agent connection failure");

            register_folder(&mut agent, folder, Duration::from_secs(*duration))
                .expect("Folder registration failure")
        }
        Commands::Unregister { socket, folder } => {
            let mut agent = ud3tn_aap::Agent::connect(
                UnixStream::connect(socket).expect("Unix stream connection failure"),
                "file-carrier/".to_owned() + &Uuid::new_v4().to_string(),
            )
            .expect("u3dtn agent connection failure");

            unregister_folder(&mut agent, folder)
                .expect("Folder registration failure")
        }
    }
}
