from os import path
from dataclasses import dataclass

@dataclass
class Hierarchy:
    fc: str
    root: str
    data: str
    reaches_file: str

def get_hierarchy(fc_folder):
    bundles_path = path.join(fc_folder, ".bundles")
    data_path = path.join(bundles_path, "data")
    reaches_file_path = path.join(bundles_path, "reaches")

    return Hierarchy(
        fc=fc_folder,
        root=bundles_path,
        data=data_path,
        reaches_file=reaches_file_path
    )