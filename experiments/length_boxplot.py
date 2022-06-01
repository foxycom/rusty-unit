import psycopg2 as psycopg2
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

sns.set_theme(style="whitegrid")
img_path = '/Users/tim/master-thesis/latex/img/experiments/'
algorithms_order = ['Seeded DynaMOSA', 'DynaMOSA', 'Random Search']

with psycopg2.connect("dbname=rustyunit user=rust password=Lz780231Ray") as conn:
    sql_random = "select * from experiments_random;"
    random_data = pd.read_sql_query(sql_random, conn)
    random_data['Algorithm'] = 'Random Search'
    random_data = random_data[random_data['crate'] != 'toycrate']

    sql_seeded_dynamosa = "select * from experiments_seeded_dynamosa;"
    seeded_dynamosa_data = pd.read_sql_query(sql_seeded_dynamosa, conn)
    seeded_dynamosa_data['Algorithm'] = 'Seeded DynaMOSA'
    seeded_dynamosa_data = seeded_dynamosa_data[seeded_dynamosa_data['crate'] != 'toycrate']

    sql_dynamosa = "select * from experiments_dynamosa;"
    dynamosa_data = pd.read_sql_query(sql_dynamosa, conn)
    dynamosa_data['Algorithm'] = 'DynaMOSA'
    dynamosa_data = dynamosa_data[dynamosa_data['crate'] != 'toycrate']

    data = pd.concat([random_data, seeded_dynamosa_data, dynamosa_data])
    data = data[data['gen'] == 99]
    fig = plt.figure(1)
    # mir_coverage, tests_length, tests, covered_targets

    coverage_plot = sns.boxplot(x="crate", y="tests_length",
                hue="Algorithm", #palette=["m", "g"],
                data=data, hue_order=algorithms_order)
    sns.despine(offset=10, trim=True)
    coverage_plot.figure.canvas.draw()
    coverage_plot.set_xticks(coverage_plot.get_xticks())
    coverage_plot.set_xticklabels(coverage_plot.get_xticklabels(), rotation=90)
    coverage_plot.get_legend().set_title(None)
    coverage_plot.set(xlabel = "Crate", ylabel = "Test case length")

    plt.tight_layout()
    plt.show()
    fig.savefig(img_path + 'length-boxplot-crates.png', dpi=300, format='png', bbox_inches='tight')