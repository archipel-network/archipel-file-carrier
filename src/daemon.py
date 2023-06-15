#!/bin/env python3
from dbus_next.aio import MessageBus, ProxyInterface
from dbus_next import BusType, InterfaceNotFoundError
from dataclasses import dataclass
import asyncio

BUSN = 'org.freedesktop.UDisks2'

loop = asyncio.get_event_loop()

async def get_manager(bus):
    introspection = await bus.introspect(BUSN, '/org/freedesktop/UDisks2/Manager')
    obj = bus.get_proxy_object(BUSN, '/org/freedesktop/UDisks2/Manager', introspection)
    return obj.get_interface('org.freedesktop.UDisks2.Manager')

@dataclass
class BlockDevice:
    path: str
    filesystem: ProxyInterface

    drive_path: str
    drive: ProxyInterface

async def get_block_device(bus, path):
    introspection = await bus.introspect(BUSN, '/org/freedesktop/UDisks2/Manager')
    obj = bus.get_proxy_object(BUSN, path, introspection)
    properties = obj.get_interface('org.freedesktop.DBus.Properties')

    try:
        filesystem = obj.get_interface('org.freedesktop.UDisks2.Filesystem')
    except InterfaceNotFoundError:
        filesystem = None

    drive_path = await properties.call_get("org.freedesktop.UDisks2.Block", "Drive")
    if drive_path.value != "/":
        drive_path = drive_path.value
        drive_obj = None #await bus.get_proxy_object(BUSN, drive_path, introspection)
        drive = None #drive_obj.get_interface('org.freedesktop.UDisks2.Drive')
    else:
        drive = None
        drive_path = None

    return BlockDevice(
        path=path,
        filesystem=filesystem,
        drive_path=drive_path,
        drive=drive
    )

async def main():
    bus = await MessageBus(bus_type=BusType.SYSTEM).connect()
    mngr = await get_manager(bus)

    for block_path in (await mngr.call_get_block_devices(dict())):
        device = await get_block_device(bus, block_path)
        print(device)

loop.run_until_complete(main())
