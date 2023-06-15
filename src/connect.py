from hier import is_file_carrier,get_hierarchy
import os
from ud3tn_utils.config import ConfigMessage, make_contact
from helpers import get_config_eid

def connect_folder(aap_client, folder, duration=300):
    if not is_file_carrier(folder):
        print("Folder {} is not a file-carrier".format(folder))
        exit(1)

    hier = get_hierarchy(folder)
    current_node = aap_client.node_eid
    reaches = []

    with open(hier.reaches_file, "r") as f:
        reaches = list(
            filter(lambda it: it != "",
                filter(lambda it: it != current_node,
                    map(lambda it: it.rstrip(), 
                        f))))
        
    with open(hier.reaches_file, "w") as f:
        f.writelines(map(lambda it: it + os.linesep, [*reaches, current_node]))

    if len(reaches) == 0:
        print("You're the only one using this file-carrier")
        print("Connect to another node to establish a connection")
        print("or manually add node EIDs in {}".format(hier.reaches_file))
        exit(10)

    msg = bytes(ConfigMessage(
        reaches[-1],
        "file:{}".format(os.path.realpath(hier.data)),
        contacts=[make_contact(0, duration, 1000000)],
        reachable_eids=reaches[:-1],
    ))

    aap_client.send_bundle(get_config_eid(current_node), msg)

    print("Connected to node {} for {} seconds".format(current_node, duration))