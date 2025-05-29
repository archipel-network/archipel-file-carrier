use std::{collections::HashMap, ffi::{CString, OsString}, fmt::Debug, future::Future, path::PathBuf, task::Poll};
use std::pin::Pin;

use futures::{join, FutureExt, Stream, StreamExt};
use zbus::{fdo::{InterfacesAddedStream, ObjectManagerProxy}, zvariant::{self, OwnedObjectPath, Str}, Connection};

pub struct DiskManager<'a> {
    connection: &'a Connection,
    manager: udisks2::ManagerProxy<'a>,
    object_manager: ObjectManagerProxy<'a>
}

impl<'a> DiskManager<'a> {
    pub async fn new(connection: &'a Connection) -> Result<Self, zbus::Error> {
        let (manager, object_manager) = join!(

            udisks2::ManagerProxy::builder(&connection)
                .build(),

            ObjectManagerProxy::builder(&connection)
                .destination("org.freedesktop.UDisks2")?
                .path("/org/freedesktop/UDisks2")?
                .build()

        );
        
        Ok(Self { connection, manager: manager?, object_manager: object_manager? })
    }

    pub async fn block_devices(&self) -> Result<Vec<BlockDevice>, zbus::Error> {
        let block_devices = self.manager.get_block_devices(&HashMap::new()).await?;
        let mut result = Vec::new();
        for path in block_devices.into_iter() {
            result.push(BlockDevice::new(self.connection, path).await?);
        }
        Ok(result)
    }

    pub async fn devices_added(&self) -> Result<DeviceAddedStream, zbus::Error> {
        Ok(DeviceAddedStream{
            connection: &self.connection,
            inner_stream: self.object_manager.receive_interfaces_added().await?,
            ungoing_task: None
        })
    }
}

pub struct DeviceAddedStream<'a>{
    connection: &'a Connection,
    inner_stream: InterfacesAddedStream,
    ungoing_task: Option<Pin<Box<dyn Future<Output = Result<BlockDevice<'a>, zbus::Error>> + 'a>>>
}

impl<'a> Stream for DeviceAddedStream<'a> {
    type Item = BlockDevice<'a>;
    
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            match self.ungoing_task.take() {
                Some(mut f) => {
                    match f.poll_unpin(cx) {
                        Poll::Pending => {
                            self.ungoing_task = Some(f);
                            return Poll::Pending
                        },
                        Poll::Ready(Err(_)) => {
                            continue;
                        },
                        Poll::Ready(Ok(device)) => {
                            return Poll::Ready(Some(device))
                        }
                    }
                },
                None => {
                    let poll_result = self.inner_stream.poll_next_unpin(cx);

                    match poll_result {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(None) => return Poll::Ready(None),
                        Poll::Ready(Some(item)) => {
                            let Some(args) = item.args().ok() else {
                                continue;
                            };
                            
                            if args.interfaces_and_properties
                                .get("org.freedesktop.UDisks2.Block")
                                .is_none() {
                                continue;
                            }

                            self.ungoing_task = Some(Box::pin(BlockDevice::new(
                                self.connection,
                                args.object_path.clone().into()
                            )));
                        }
                    }
                }
            }
        }
    }
}

pub struct BlockDevice<'a> {
    connection: &'a Connection,
    path: OwnedObjectPath,
    proxy: udisks2::BlockProxy<'a>
}

impl<'a> Debug for BlockDevice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BlockDevice").field(&self.path.as_str()).finish()
    }
}

impl<'a> BlockDevice<'a> {
    pub async fn new(connection: &'a Connection, path: OwnedObjectPath) -> Result<Self, zbus::Error> {
        let proxy = udisks2::BlockProxy::builder(connection)
            .path(path.clone())?
            .build().await?;
        
        Ok(Self {
            connection,
            path,
            proxy
        })
    }

    pub async fn drive(&self) -> Result<Option<Drive>, zbus::Error> {
        let drive_path = self.proxy.drive().await?;
        if drive_path.as_str() == "/" {
            Ok(None)
        } else {
            Ok(Some(Drive::new(self.connection, drive_path).await?))
        }
    }

    pub async fn try_into_mountable(self) -> Result<MountableDevice<'a>, IntoMountableDeviceError> {
        MountableDevice::try_from_device(self).await
    }

    pub async fn id(&self) -> Result<String, zbus::Error> {
        self.proxy.id().await
    }
}

pub struct Drive<'a> {
    path: OwnedObjectPath,
    proxy: udisks2::DriveProxy<'a>
}

impl<'a> Debug for Drive<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Drive").field(&self.path.as_str()).finish()
    }
}

impl<'a> Drive<'a> {
    pub async fn new(connection: &'a Connection, path: OwnedObjectPath) -> Result<Self, zbus::Error> {
        let proxy = udisks2::DriveProxy::builder(connection)
            .path(path.clone())?
            .build().await?;
        
        Ok(Self {
            path,
            proxy
        })
    }

    pub async fn removable(&self) -> Result<bool, zbus::Error> {
        let Some(cached_removable) = self.proxy.cached_removable()? else {
            return self.proxy.removable().await
        };
        Ok(cached_removable)
    }
}

pub struct MountableDevice<'a>(BlockDevice<'a>, udisks2::FilesystemProxy<'a>);

impl<'a> MountableDevice<'a> {
    pub async fn try_from_device(drive: BlockDevice<'a>) -> Result<Self, IntoMountableDeviceError> {
        let filesystem = udisks2::FilesystemProxy::builder(drive.connection)
            .path(drive.path.clone())?
            .build().await?;

        match filesystem.mount_points().await {
            Ok(_) => Ok(Self(drive, filesystem)),
            Err(zbus::Error::InterfaceNotFound) => Err(IntoMountableDeviceError::NotMountable),
            Err(ref e @ zbus::Error::MethodError(ref name, _, _)) => {
                if name.as_str() == "org.freedesktop.DBus.Error.InvalidArgs" {
                    Err(IntoMountableDeviceError::NotMountable)
                } else {
                    Err(IntoMountableDeviceError::Dbus(e.clone()))
                }
            },
            Err(e) => Err(IntoMountableDeviceError::Dbus(e))
        }
    }

    pub async fn drive(&self) -> Result<Option<Drive>, zbus::Error> {
        self.0.drive().await
    }
    
    pub async fn mountpoints(&self) -> Result<Vec<PathBuf>, zbus::Error> {
        let mountpoints = self.1.mount_points().await?;
        let paths = mountpoints.into_iter()
            .map(|path| {
                let os_string = CString::from_vec_with_nul(path).unwrap();
                // BUG Maybe : I don't know if mountpoints are OsString or String encoded characters
                PathBuf::from(unsafe {
                    OsString::from_encoded_bytes_unchecked(os_string.to_bytes().to_vec())
                })
            }).collect();
        Ok(paths)
    }

    pub async fn mount(&self) -> Result<PathBuf, zbus::Error> {
        let mountpath = self.1.mount(&HashMap::new()).await?;
        Ok(PathBuf::from(mountpath))
    }

    pub async fn mount_as_user(&self, username: &str) -> Result<PathBuf, zbus::Error> {

        let mut options = HashMap::new();
        options.insert("as-user".to_owned(), zvariant::Value::Str(Str::from(username)));

        let mountpath = self.1.mount(&options).await?;
        Ok(PathBuf::from(mountpath))
    }

    pub async fn unmount(&self) -> Result<(), zbus::Error> {
        self.1.unmount(&HashMap::new()).await
    }

    pub async fn id(&self) -> Result<String, zbus::Error> {
        self.0.id().await
    }
}

impl<'a> Debug for MountableDevice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MountableDevice").field(&self.0.path.as_str()).finish()
    }
}

pub enum IntoMountableDeviceError {
    Dbus(zbus::Error),
    NotMountable
}

impl From<zbus::Error> for IntoMountableDeviceError {
    fn from(value: zbus::Error) -> Self {
        IntoMountableDeviceError::Dbus(value)
    }
}

mod udisks2 {
    use std::collections::HashMap;
    use zbus::{
        proxy,
        zvariant::{self, OwnedObjectPath},
        Result,
    };

    #[proxy(
        interface = "org.freedesktop.UDisks2.Manager",
        default_service = "org.freedesktop.UDisks2",
        default_path = "/org/freedesktop/UDisks2/Manager"
    )]
    pub trait Manager {
        async fn get_block_devices(
            &self,
            options: &HashMap<String, zvariant::Value<'_>>,
        ) -> Result<Vec<OwnedObjectPath>>;
    }

    #[proxy(
        interface = "org.freedesktop.UDisks2.Block",
        default_service = "org.freedesktop.UDisks2"
    )]
    pub trait Block {
        #[zbus(property)]
        fn drive(&self) -> Result<OwnedObjectPath>;

        #[zbus(property)]
        fn id(&self) -> Result<String>;
    }

    #[proxy(
        interface = "org.freedesktop.UDisks2.Filesystem",
        default_service = "org.freedesktop.UDisks2"
    )]
    pub trait Filesystem {
        
        async fn mount(&self, options: &HashMap<String, zvariant::Value<'_>>) -> Result<String>;
        async fn unmount(&self, options: &HashMap<String, zvariant::Value<'_>>) -> Result<()>;

        #[zbus(property)]
        fn mount_points(&self) -> Result<Vec<Vec<u8>>>;
    }

    #[proxy(
        interface = "org.freedesktop.UDisks2.Drive",
        default_service = "org.freedesktop.UDisks2"
    )]
    pub trait Drive {
        #[zbus(property)]
        fn removable(&self) -> Result<bool>;
    }
}