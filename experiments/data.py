from os import listdir
from os.path import join, isfile

import pandas as pd

DATA_PATH = "/Users/tim/master-thesis/experiments/data"

def get_data_files(path):
    data_files = [join(path, f) for f in listdir(path) if isfile(join(path, f))]
    return data_files

def get_data():
    return pd.concat((pd.read_csv(f) for f in get_data_files(DATA_PATH)))