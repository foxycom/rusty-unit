import matplotlib.pyplot as plt
import seaborn as sns

from data import get_data

df = get_data()
sns.set_theme()
sns.set_style("white")
sns.color_palette("deep")

fig = plt.figure(1)
ax = sns.boxplot(x="Crate", y="Length", hue="Algorithm",
            data=df)
ax.set_xticks(ax.get_xticks())
ax.set_xticklabels(ax.get_xticklabels(), rotation=90, )
plt.show()


fig.savefig('length.png', dpi=300, format='png', bbox_inches='tight')