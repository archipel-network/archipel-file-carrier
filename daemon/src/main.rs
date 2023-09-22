use std::{collections::HashMap, os::unix::net::UnixStream, env, path::PathBuf};

use async_std::{task, stream::StreamExt};
use file_carrier::default_path;
use zbus::{
    zvariant::{self, OwnedObjectPath},
    Connection, Result, fdo::ObjectManagerProxy,
};

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
        async fn mount(&self,
            options: &HashMap<String, zvariant::Value<'_>>
        ) -> Result<String>;

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

async fn get_mount_path(connection: &Connection, path: OwnedObjectPath) -> Result<(String)>{
    let block_proxy = udisks2::BlockProxy::builder(&connection).path(path.clone())?.build().await?;
    let drive_path = block_proxy.drive().await?;
    if *drive_path == "/" {
        return Err(zbus::Error::Failure("No drive associated to this block device".into()));
    }

    // println!("{drive_path}");

    
    let filesystem_proxy = udisks2::FilesystemProxy::builder(&connection).path(path.clone())?.build().await?;
    let drive_proxy = udisks2::DriveProxy::builder(&connection).path(drive_path.clone())?.build().await?;
    if drive_proxy.removable().await? {
        let mount_points = filesystem_proxy.mount_points().await?;
        let mount_path = if mount_points.is_empty() {
            filesystem_proxy.mount(&HashMap::<String,zvariant::Value>::new()).await?
        }
        else {
            String::from_utf8(mount_points.into_iter().nth(0).unwrap()).unwrap()
        };
        
        println!("drive path: {mount_path}");
        Ok(mount_path)
    }
    else {
        return Err(zbus::Error::Failure("This drive is not removable".into()));
    }
}

async fn startup_check(connection: &Connection) -> Result<(Vec<String>)> {
    // `dbus_proxy` macro creates `MyGreeterProxy` based on `Notifications` trait.
    let manager_proxy = udisks2::ManagerProxy::new(connection).await?;
    let reply = manager_proxy
        .get_block_devices(&HashMap::<String, zvariant::Value>::new())
        .await?;

    let mut paths = Vec::new();
    for object in reply {
        match get_mount_path(connection, object).await {
            Ok(path) => paths.push(path),
            Err(_) => {},
        }
    }

    Ok(paths)
}

async fn async_main() -> Result<()> {
    let connection = Connection::system().await?;

    startup_check(&connection).await?;
    
    let object_manager_proxy = ObjectManagerProxy::builder(&connection).destination("org.freedesktop.UDisks2")?.path("/org/freedesktop/UDisks2")?.build().await?;

    let mut stream = object_manager_proxy.receive_interfaces_added().await?;

    loop {
        let Some(signal) = stream.next().await else {
            continue;
        };

        let args = signal.args()?;
        
        if args.interfaces_and_properties.get("org.freedesktop.UDisks2.Filesystem").is_some() {
            get_mount_path(&connection, args.object_path.into()).await?;
        }
    }

    // println!("{reply:?}");
}

fn main() {
    let socket = match env::args().skip(1).next() {
        Some(value) => PathBuf::from(value),
        None => default_path(),
    };

    println!("socket: {}", socket.display());

    let mut agent = ud3tn_aap::Agent::connect(
        UnixStream::connect(socket).expect("Unix stream connection failure"),
        "file-carrier-daemon".to_owned(),
    )
    .expect("u3dtn agent connection failure");

    task::block_on(async_main()).expect("Ca a foiré lol");
}
