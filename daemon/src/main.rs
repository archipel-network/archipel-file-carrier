use std::{
    collections::HashMap,
    env,
    ops::DerefMut,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use async_std::{
    stream::StreamExt,
    sync::Mutex,
    task::{self},
};
use file_carrier::{default_path, register::register_folder};
use ud3tn_aap::{AapStream, RegisteredAgent};
use zbus::{
    fdo::{InterfacesAddedStream, InterfacesRemovedStream, ObjectManagerProxy},
    zvariant::{self, OwnedObjectPath},
    Connection, Result,
};

const SECONDS_IN_YEAR: u64 = 31556952;

mod udisks2 {
    use std::collections::HashMap;
    use zbus::{
        dbus_proxy,
        zvariant::{self, OwnedObjectPath},
        Result,
    };

    #[dbus_proxy(
        interface = "org.freedesktop.UDisks2.Manager",
        default_service = "org.freedesktop.UDisks2",
        default_path = "/org/freedesktop/UDisks2/Manager"
    )]
    trait Manager {
        async fn get_block_devices(
            &self,
            options: &HashMap<String, zvariant::Value<'_>>,
        ) -> Result<Vec<OwnedObjectPath>>;
    }

    #[dbus_proxy(
        interface = "org.freedesktop.UDisks2.Block",
        default_service = "org.freedesktop.UDisks2"
    )]
    trait Block {
        #[dbus_proxy(property)]
        fn drive(&self) -> Result<OwnedObjectPath>;
    }

    #[dbus_proxy(
        interface = "org.freedesktop.UDisks2.Filesystem",
        default_service = "org.freedesktop.UDisks2"
    )]
    trait Filesystem {
        async fn mount(&self, options: &HashMap<String, zvariant::Value<'_>>) -> Result<String>;

        #[dbus_proxy(property)]
        fn mount_points(&self) -> Result<Vec<Vec<u8>>>;
    }

    #[dbus_proxy(
        interface = "org.freedesktop.UDisks2.Drive",
        default_service = "org.freedesktop.UDisks2"
    )]
    trait Drive {
        #[dbus_proxy(property)]
        fn removable(&self) -> Result<bool>;
    }
}

async fn get_mount_path(connection: &Connection, path: OwnedObjectPath) -> Result<String> {
    let block_proxy = udisks2::BlockProxy::builder(&connection)
        .path(path.clone())?
        .build()
        .await?;
    let drive_path = block_proxy.drive().await?;
    if *drive_path == "/" {
        return Err(zbus::Error::Failure(
            "No drive associated to this block device".into(),
        ));
    }

    let filesystem_proxy = udisks2::FilesystemProxy::builder(&connection)
        .path(path.clone())?
        .build()
        .await?;
    let drive_proxy = udisks2::DriveProxy::builder(&connection)
        .path(drive_path.clone())?
        .build()
        .await?;

    if drive_proxy.removable().await? {
        let mount_points = filesystem_proxy.mount_points().await?;
        let mount_path = if mount_points.is_empty() {
            filesystem_proxy
                .mount(&HashMap::<String, zvariant::Value>::new())
                .await?
        } else {
            String::from_utf8(mount_points[0].clone()).unwrap()
        };

        Ok(mount_path)
    } else {
        return Err(zbus::Error::Failure("This drive is not removable".into()));
    }
}

async fn startup_check(connection: &Connection) -> Result<Vec<(OwnedObjectPath, String)>> {
    // `dbus_proxy` macro creates `MyGreeterProxy` based on `Notifications` trait.
    let manager_proxy = udisks2::ManagerProxy::new(connection).await?;
    let reply = manager_proxy
        .get_block_devices(&HashMap::<String, zvariant::Value>::new())
        .await?;

    let mut paths = Vec::new();
    for object in reply {
        match get_mount_path(connection, object.clone()).await {
            Ok(path) => paths.push((object, path[..path.len()-1].to_owned())),
            Err(_) => {}
        }
    }

    Ok(paths)
}

type SharedInterfaceMap = Arc<Mutex<HashMap<OwnedObjectPath, String>>>;

async fn interface_added_watcher<S: AapStream>(
    mut added_stream: InterfacesAddedStream<'_>,
    connection: Connection,
    agent: Arc<Mutex<RegisteredAgent<S>>>,
    interfaces: SharedInterfaceMap,
) -> Result<()> {
    loop {
        if let Some(signal) = added_stream.next().await {
            let args = signal.args()?;

            if args
                .interfaces_and_properties
                .get("org.freedesktop.UDisks2.Filesystem")
                .is_some()
            {
                let object_path = args.object_path.clone().into();
                let mount_path = get_mount_path(&connection, args.object_path.into()).await?;
                let path = Path::new(&mount_path).to_owned();

                println!("Register folder: {mount_path}");

                match register_folder(
                    &mut agent.lock().await.deref_mut(),
                    &path,
                    Duration::from_secs(SECONDS_IN_YEAR),
                ) {
                    Ok(eid) => {
                        interfaces.lock().await.insert(object_path, eid);
                    }
                    Err(err) => {
                        eprintln!("{err}")
                    }
                }
            }
        };
    }
}

async fn interface_removed_watcher<S: AapStream>(
    mut removed_stream: InterfacesRemovedStream<'_>,
    agent: Arc<Mutex<RegisteredAgent<S>>>,
    interfaces: SharedInterfaceMap,
) -> Result<()> {
    loop {
        if let Some(signal) = removed_stream.next().await {
            let args = signal.args()?;
            if let Some(eid) = interfaces
                .lock()
                .await
                .remove(&args.object_path().clone().into())
            {
                let msg = ud3tn_aap::config::ConfigBundle::DeleteContact(eid.clone());
                match agent.lock().await.send_config(msg) {
                    Ok(_) => {println!("Disconnected from node {eid}")}
                    Err(err) => {
                        eprintln!("{err}")
                    }
                }
            }
        };
    }
}

async fn async_main<S: AapStream + 'static>(mut agent: RegisteredAgent<S>) -> Result<()> {
    let interfaces = Arc::new(Mutex::new(HashMap::new()));
    let connection = Connection::system().await?;

    for (object_path, path) in startup_check(&connection).await? {
        match register_folder(
            &mut agent,
            Path::new(&path),
            Duration::from_secs(SECONDS_IN_YEAR),
        ) {
            Ok(eid) => {
                interfaces.lock().await.insert(object_path, eid);
            }
            Err(err) => {
                eprintln!("{err} {path}");
            }
        }
    }

    let agent = Arc::new(Mutex::new(agent));

    let object_manager_proxy = ObjectManagerProxy::builder(&connection)
        .destination("org.freedesktop.UDisks2")?
        .path("/org/freedesktop/UDisks2")?
        .build()
        .await?;
    let removed_stream = object_manager_proxy.receive_interfaces_removed().await?;
    let added_stream = object_manager_proxy.receive_interfaces_added().await?;

    task::spawn(interface_added_watcher(
        added_stream,
        connection,
        agent.clone(),
        interfaces.clone(),
    ));
    interface_removed_watcher(removed_stream, agent, interfaces).await

    // println!("{reply:?}");
}

fn main() {
    let socket = match env::args().skip(1).next() {
        Some(value) => PathBuf::from(value),
        None => default_path(),
    };

    println!("Communicate through socket: {}", socket.display());

    let agent = ud3tn_aap::Agent::connect_unix(
        &socket,
    )
        .expect("u3dtn agent connection failure")
    .register("file-carrier-daemon".to_owned())
        .expect("Failed to register agent");

    task::block_on(async_main(agent)).expect("Ca a foiré lol");
}
