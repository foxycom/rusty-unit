import seaborn as sns
import matplotlib.pyplot as plt
import pandas as pd
from os import listdir
from os.path import isfile, join

def get_data_files(path):
    data_files = [join(path, f) for f in listdir(path) if isfile(join(path, f))]
    return data_files

data_files = get_data_files("/Users/tim/master-thesis/experiments/data")
df = pd.concat((pd.read_csv(f) for f in data_files))
sns.set_theme()
tips = sns.load_dataset("tips")

print(df.head())
sns.boxplot(x="Crate", y="Coverage", hue="Algorithm",
                 data=df, palette="Set3")
plt.show()



