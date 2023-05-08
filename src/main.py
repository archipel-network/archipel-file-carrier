#!/bin/env python3
import argparse
from init import initialize_fc

parser = argparse.ArgumentParser("archipel-fc")
subparsers = parser.add_subparsers(required=True, metavar="command", dest="command")

def add_socket_argument(parser: argparse.ArgumentParser):
    parser.add_argument("-s", "--socket", help="Archipel Core socket to connect to")

# Init subcommand
init_parser = subparsers.add_parser("init", help="Initialize a new file carrier")
init_parser.add_argument("folder", default=".", nargs='?', help="File carrier folder to initialize")
init_parser.add_argument("--force", help="Force intitialization event if .bundles folder exists", action="store_true")

# Script execution
args = parser.parse_args()
com = args.command

if com == "init":
    initialize_fc(args.folder, force=args.force)
