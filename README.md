# Archipel File Carrier

*Use usb sticks to carry bundles.*

This is a daemon watching drive connections using dbus then connect folder to [Archipel Core](https://github.com/EpicKiwi/archipel-core) File CLA for bundle exchange.

## Development

Clone this repository

Use CLI

```sh
cargo run --bin file-carrier
```

Use daemon

```sh
cargo run --bin file-carrier-daemon
```

## Inspired by

* [Dead Drops](https://deaddrops.com/) : An anonymous, offline, peer to peer file-sharing network in public space
