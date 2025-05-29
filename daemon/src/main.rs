use std::{path::{Path, PathBuf}, time::Duration};

use async_std::{channel::{self, Receiver, RecvError, Sender}, task};
use disks::{DiskManager, IntoMountableDeviceError, MountableDevice};
use file_carrier::{hierarchy::FileCarrierHierarchy, register::register_folder};
use futures::{future::try_join_all, StreamExt};
use ud3tn_aap::{AapStream, Agent, RegisteredAgent};
use zbus::Connection;
use clap::Parser;

mod disks;

async fn async_main(cli: Cli) {

    let config_agent = Agent::connect_unix(
        Path::new("/run/archipel-core/archipel-core.socket")
    )
        .expect("Failed to connect to Archipel Core")
        .register("file-carrier/config-daemon".to_owned())
        .expect("Failed to register config agent");

    let (agent_message_sender, agent_task) = {
        let (sender, receiver) = channel::unbounded::<AgentMessage>();
        let task = task::spawn(agent_task(receiver, config_agent));
        (sender, task)
    };

    let dbus = Connection::system().await
        .expect("Failed to connect to dbus");

    let manager = DiskManager::new(&dbus).await
        .expect("Failed to connect to udisks2");

    init_all_devices(&manager, agent_message_sender.clone(), cli.as_user.clone()).await
        .expect("Failed to init all devices");
    
    let mut added_devices = manager.devices_added().await
        .expect("Failed to watch added devices");

    loop {
        let Some(added_device) = added_devices.next().await else {
            break
        };

        let device = match added_device.try_into_mountable().await {
            Err(IntoMountableDeviceError::NotMountable) => continue,
            Err(IntoMountableDeviceError::Dbus(e)) => panic!("Failed to get mountable device: {e}"),
            Ok(d) => d
        };

        let Some(drive) = device.drive().await
            .expect("Failed to get drive") else {
            continue;
        };

        if !drive.removable().await.expect("Failed to check for removable drive") {
            continue;
        }

        let device_id = device.id().await.unwrap();

        match mount_and_register(device, agent_message_sender.clone(), cli.as_user.clone()).await {
            Err(e) => {
                eprintln!("Failed to mount and register device {}: {e}", device_id);
                continue;
            },
            Ok(()) => {

            }
        };


    }

    agent_message_sender.send(AgentMessage::Shutdown).await.unwrap();
    agent_task.await;
}

async fn init_all_devices<'a>(manager: &'a DiskManager<'a>, agent_message_sender: Sender<AgentMessage>, mount_as_user: Option<String>) -> Result<(), zbus::Error> {

    let devices = manager.block_devices().await?;

    let mountable_removable_drive = try_join_all(
        devices.into_iter()
            .map(async |device| {
                let Some(drive) = device.drive().await? else {
                    return Ok(None)
                };

                if !drive.removable().await? {
                    return Ok(None)
                }

                match device.try_into_mountable().await {
                    Ok(d) => Ok(Some(d)),
                    Err(IntoMountableDeviceError::NotMountable) => Ok(None),
                    Err(IntoMountableDeviceError::Dbus(e)) => Err(e)
                }
            })
        ).await?.into_iter().filter_map(|it| it);

    try_join_all(
        mountable_removable_drive.map(async |device| {

            let (mounted_by_us, mountpoint) = match device.mountpoints().await?.into_iter().next() {
                Some(m) => (false, m),
                None => (true, device.mount().await?)
            };

            match FileCarrierHierarchy::folder_is_file_carrier(&mountpoint) {
                Ok(is_fc) => {
                    if is_fc {
                        device.unmount().await?;
                        mount_and_register(device, agent_message_sender.clone(), mount_as_user.clone()).await?;
                        return Ok(());
                    }
                },
                Err(e) => eprintln!("Failed to open folder {}: {}", mountpoint.display(), e)
            }
            
            println!("Device {} is not a file carrier", mountpoint.display());
                
            if mounted_by_us {
                device.unmount().await?;
            }

            Ok::<_, zbus::Error>(())
        })
    ).await?;

    Ok(())
}

async fn mount_and_register<'a>(device: MountableDevice<'a>, agent_message_sender: Sender<AgentMessage>, mount_as_user: Option<String>) -> Result<(), zbus::Error> {
    println!("Should mount and register filecarrier in {:?}", device);

    let mountpoint = match mount_as_user {
        Some(username) => device.mount_as_user(&username).await?,
        None => device.mount().await?
    };

    match FileCarrierHierarchy::folder_is_file_carrier(&mountpoint) {
        Ok(is_fc) => {
            if !is_fc {
                device.unmount().await?;
                println!("Device {} is not a file carrier", device.id().await.unwrap());
                return Ok(());
            }
        },
        Err(e) => eprintln!("Failed to open folder {}: {}", mountpoint.display(), e)
    }

    agent_message_sender.send(AgentMessage::Register(mountpoint)).await
        .expect("Failed to send agent message");

    Ok(())
}

enum AgentMessage {
    Register(PathBuf),
    Shutdown
}

async fn agent_task<T:AapStream>(receiver: Receiver<AgentMessage>, mut agent: RegisteredAgent<T>) {
    loop {
        match receiver.recv().await {
            Ok(AgentMessage::Register(path)) => {
                if let Err(e) = register_folder(
                    &mut agent, &path, Duration::from_secs(10 * 25 * 3600)) {
                    eprintln!("Failed to register folder {}: {}", path.display(), e)
                } else {
                    println!("Registered folder {} as file carrier", path.display());
                }
            },
            Ok(AgentMessage::Shutdown) => return,
            Err(RecvError) => return // Empty and closed channel (end of task)
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    /// Mount file carrier for the provided user instead of user currently running daemon
    /// Useful if Archipel Core is not running as current user
    #[arg(long)]
    as_user: Option<String>
}

fn main() {
    let cli = Cli::parse();

    task::block_on(async_main(cli))
}
