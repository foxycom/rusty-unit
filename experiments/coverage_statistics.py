import pandas as pd
import psycopg2
from scipy.stats import mannwhitneyu


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


with psycopg2.connect("dbname=rustyunit user=rust password=Lz780231Ray") as conn:
    sql_random = "select * from experiments_random;"
    random_data = pd.read_sql_query(sql_random, conn)
    random_data['Algorithm'] = 'Random Search'
    random_data = random_data[random_data['crate'] != 'trying']
    random_data = random_data[random_data['crate'] != 'toycrate']
    random_data = random_data[random_data['gen'] == 99]

    sql_seeded_dynamosa = "select * from experiments_seeded_dynamosa;"
    seeded_dynamosa_data = pd.read_sql_query(sql_seeded_dynamosa, conn)
    seeded_dynamosa_data['Algorithm'] = 'Seeded DynaMOSA'
    seeded_dynamosa_data = seeded_dynamosa_data[seeded_dynamosa_data['crate'] != 'toycrate']
    seeded_dynamosa_data = seeded_dynamosa_data[seeded_dynamosa_data['gen'] == 99]

    sql_dynamosa = "select * from experiments_dynamosa;"
    dynamosa_data = pd.read_sql_query(sql_dynamosa, conn)
    dynamosa_data['Algorithm'] = 'DynaMOSA'
    dynamosa_data = dynamosa_data[dynamosa_data['gen'] == 99]
    dynamosa_data = dynamosa_data[dynamosa_data['crate'] != 'toycrate']

    data = pd.concat([random_data, seeded_dynamosa_data])

    seeded_a12_sum = 0
    a12_sum = 0
    for crate in random_data["crate"].unique():
        rs_values = random_data[random_data["crate"] == crate]["mir_coverage"]
        seeded_dynamosa_values = seeded_dynamosa_data[seeded_dynamosa_data["crate"] == crate]["mir_coverage"]
        dynamosa_values = dynamosa_data[dynamosa_data["crate"] == crate]["mir_coverage"]

        seeded_mw = mannwhitneyu(seeded_dynamosa_values, rs_values)
        mw = mannwhitneyu(dynamosa_values, rs_values)

        seeded_a12_sum += a12(seeded_dynamosa_values, rs_values)
        a12_sum += a12(dynamosa_values, rs_values)

        print(f"{crate}: Seeded DynaMOSA is better in {a12(seeded_dynamosa_values, rs_values) * 100}% of cases")
        print(f"{crate}: DynaMOSA is better in {a12(dynamosa_values, rs_values)}")
        print(f"Seeded MWU Test p-value = {seeded_mw.pvalue} => {'Reject H0' if seeded_mw.pvalue < 0.05 else 'Accept H0'}")
        print(f"MWU Test p-value = {mw.pvalue} => {'Reject H0' if mw.pvalue < 0.05 else 'Accept H0'}")
        print(f"RS average: {rs_values.mean()}")
        print(f"Seeded DynaMOSA average: {seeded_dynamosa_values.mean()}")
        print(f"DynaMOSA average: {dynamosa_values.mean()}\n")


    print(f"RS average: {random_data['mir_coverage'].mean()}")
    print(f"Seeded DynaMOSA average: {seeded_dynamosa_data['mir_coverage'].mean()}")
    print(f"DynaMOSA average: {dynamosa_data['mir_coverage'].mean()}")
    print(f"Seeded A12 average: {seeded_a12_sum / len(random_data['crate'].unique())}")
    print(f"A12 average: {a12_sum / len(random_data['crate'].unique())}")
