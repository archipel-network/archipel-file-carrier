#!/bin/env python3
import argparse
from init import initialize_fc
from connect import connect_folder
from disconnect import disconnect_folder
from ud3tn_utils.aap import AAPUnixClient

parser = argparse.ArgumentParser("archipel-fc")
subparsers = parser.add_subparsers(required=True, metavar="command", dest="command")

def add_socket_argument(parser: argparse.ArgumentParser):
    parser.add_argument("-s", "--socket", help="Archipel Core socket to connect to", default="ud3tn.socket")

# Init subcommand
init_parser = subparsers.add_parser("init", help="Initialize a new file carrier")
init_parser.add_argument("folder", default=".", nargs='?', help="File carrier folder to initialize")
init_parser.add_argument("--force", help="Force intitialization event if .bundles folder exists", action="store_true")

# Connect subcommand
connect_parser = subparsers.add_parser("connect", help="Connect a file carrier to Archipel Core")
connect_parser.add_argument("folder", default=".", nargs='?')
connect_parser.add_argument("-d", "--duration", help="Duration of the connection (in seconds)", default=300, type=int)
add_socket_argument(connect_parser)

# Disconnect subcommand
disconnect_parser = subparsers.add_parser("disconnect", help="Remove a file carrier from Archipel Core")
disconnect_parser.add_argument("folder", default=".", nargs='?')
add_socket_argument(disconnect_parser)

# Script execution
args = parser.parse_args()
com = args.command

if com == "init":
    initialize_fc(args.folder, force=args.force)

elif com == "connect":
    with AAPUnixClient(address=args.socket) as aap_client:
        aap_client.register()
        connect_folder(aap_client, args.folder, duration=args.duration)

elif com == "disconnect":
    with AAPUnixClient(address=args.socket) as aap_client:
        aap_client.register()
        disconnect_folder(aap_client, args.folder)
