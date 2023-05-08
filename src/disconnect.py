from hier import is_file_carrier,get_hierarchy
import os
from ud3tn_utils.config import ConfigMessage, RouterCommand
from helpers import get_config_eid

def disconnect_folder(aap_client, folder):
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

    if len(reaches) == 0:
        return

    msg = bytes(ConfigMessage(
        reaches[-1],
        "file:{}".format(os.path.realpath(hier.data)),
        contacts=[],
        reachable_eids=[],
        type=RouterCommand.DELETE
    ))

    aap_client.send_bundle(get_config_eid(current_node), msg)

    print("Disconnected {}".format(folder))