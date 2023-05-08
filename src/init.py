import os
from hier import get_hierarchy
import shutil
from os import path

def initialize_fc(folder, force=False):
    hier = get_hierarchy(folder)
    
    try:
        os.mkdir(hier.root)
    except FileExistsError:
        print("Folder {} is already a file carrier".format(path.realpath(folder)))
        if not force:
            return

    os.makedirs(hier.data, exist_ok=True)

    with open(hier.reaches_file, "a") as f:
        f.flush()
    
    shutil.copyfile(path.join(path.dirname(__file__), "templates/readme.txt"), path.join(hier.root, "readme.txt"))

    print("File carrier initialized in {}".format(path.realpath(folder)))
    
    