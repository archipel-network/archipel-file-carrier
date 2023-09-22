use std::collections::HashMap;

use async_std::task;
use zbus::{
    zvariant::{self},
    Connection, Result,
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

async fn async_main() -> Result<()> {
    let connection = Connection::system().await?;

    // `dbus_proxy` macro creates `MyGreeterProxy` based on `Notifications` trait.
    let proxy = udisks2::ManagerProxy::new(&connection).await?;
    let reply = proxy
        .get_block_devices(&HashMap::<String, zvariant::Value>::new())
        .await?;

    // println!("{reply:?}");

    for object in reply {
        let block_proxy = udisks2::BlockProxy::builder(&connection).path(object.clone())?.build().await?;
        let drive_path = block_proxy.drive().await?;
        if *drive_path == "/" {
            continue;
        }
        
        let filesystem_proxy = udisks2::FilesystemProxy::builder(&connection).path(object.clone())?.build().await?;
        let drive_proxy = udisks2::DriveProxy::builder(&connection).path(drive_path.clone())?.build().await?;
        if drive_proxy.removable().await? {
            if let Ok(mount_points) = filesystem_proxy.mount_points().await {
                let mount_path = if mount_points.is_empty() {
                    filesystem_proxy.mount(&HashMap::<String,zvariant::Value>::new()).await?
                }
                else {
                    String::from_utf8(mount_points.into_iter().nth(0).unwrap()).unwrap()
                };

                println!("drive path: {mount_path}");
            }
        }
    }

    Ok(())
}

fn main() {
    task::block_on(async_main()).expect("Ca a foiré lol");
}
