#!/bin/bash

set -e

if [ ! -e target/release/file-carrier ]; then
    echo "Please build first with : cargo build --release"
    exit 1
fi

echo ""
echo "  Install Archipel File Carrier on your system"
echo ""

mkdir -p /usr/share/archipel-fc

# CLI
cp -v target/release/file-carrier /usr/share/archipel-fc/archipel-fc
ln -f -s /usr/share/archipel-fc/archipel-fc /usr/bin/archipel-fc

# Damon
cp -v target/release/file-carrier-daemon /usr/share/archipel-fc/archipel-fcd
cp -v daemon/user-archipel-fcd.service /etc/systemd/user/archipel-fcd.service
cp -v daemon/archipel-fcd.service /etc/systemd/system/archipel-fcd.service

echo ""
echo "  Archipel File Carrier is now installed"
echo ""
echo "  CLI interface is available as archipel-fc"
echo ""
echo "  Enable and Start removable drive daemon with the following"
echo ""
echo "      systemctl start --user archipel-fcd"
echo "      systemctl enable --user archipel-fcd"
echo ""
echo "  For system-wide service just remove --user in the previous commands"
echo ""