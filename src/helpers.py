DEFAULT_CONFIG_AGENT_ID_DTN = "config"
DEFAULT_CONFIG_AGENT_ID_IPN = "9000"

def get_node_eid_prefix(eid):
    if eid[0:6] == "dtn://":
        return "dtn://" + eid.split("/")[2] + "/"
    elif eid[0:4] == "ipn:":
        return eid.split(".")[0] + "."
    else:
        raise ValueError("Cannot determine the node prefix for the given EID.")

def get_config_eid(eid):
    return get_node_eid_prefix(eid) + (
        DEFAULT_CONFIG_AGENT_ID_DTN
        if eid[0] == "d"  # get_node_eid_prefix already checks everything else
        else DEFAULT_CONFIG_AGENT_ID_IPN
    )