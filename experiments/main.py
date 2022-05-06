import seaborn as sns
import matplotlib.pyplot as plt
import pandas as pd

#df = pd.read_csv('data.csv')
df = pd.concat((pd.read_csv(f) for f in data_files))
sns.set_theme()
tips = sns.load_dataset("tips")

head = tips.head()
print(head)
#sns.boxplot(x="", y="total_bill", hue="smoker",
#                 data=tips, palette="Set3")
#plt.show()