import matplotlib.pyplot as plt
import seaborn as sns

from data import get_data

df = get_data()
sns.set_theme()
sns.set_style("white")
sns.color_palette("deep")

sns.boxplot(x="Crate", y="Coverage", hue="Algorithm",
            data=df)
plt.show()
