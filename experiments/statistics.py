from scipy.stats import mannwhitneyu

from data import get_data


def a12(lst1, lst2, rev=True):
    "how often is x in lst1 more than y in lst2?"
    more = same = 0.0
    for x in lst1:
        for y in lst2:
            if x == y:
                same += 1
            elif rev and x > y:
                more += 1
            elif not rev and x < y:
                more += 1
    return (more + 0.5 * same) / (len(lst1) * len(lst2))


df = get_data()
rs = df[df["Algorithm"] == "Random Search"]
dynamosa = df[df["Algorithm"] == "DynaMOSA"]

for crate in rs["Crate"].unique():
    rs_values = rs[rs["Crate"] == crate]["Coverage"]
    dynamosa_values = dynamosa[dynamosa["Crate"] == crate]["Coverage"]
    print(f"{crate}: DynaMOSA is better in {a12(dynamosa_values, rs_values) * 100}% of cases")