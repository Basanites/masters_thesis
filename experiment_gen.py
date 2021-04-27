import glob
import os
import math
import itertools
import sys
from string import Template


def aco_cfg(alpha, beta, rho, q_0, seed, ant_count, constructions):
    iterations = math.ceil(constructions / ant_count)
    return f"""    alpha: {alpha}
    beta: {beta}
    rho: {rho}
    q_0: {q_0}
    seed: {seed}
    ant_count: {ant_count}
    iterations: {iterations}"""


def mm_aco_cfg(alpha, beta, rho, q_0, p_best, seed, ant_count, constructions):
    iterations = math.ceil(constructions / ant_count)
    return f"""    alpha: {alpha}
    beta: {beta}
    rho: {rho}
    q_0: {q_0}
    p_best: {p_best}
    seed: {seed}
    ant_count: {ant_count}
    iterations: {iterations}"""


def random_cfg(seed, constructions):
    return f"""    seed: {seed}
    iterations: {constructions}"""


def file_cfg(filename, nw_spread, nw_chance):
    return f"""    filename: {filename}
    nw_range:
        - {nw_spread[0]}
        - {nw_spread[1]}
    node_weight_probability: {nw_chance}"""


def generation_cfg(dimension, nw_spread, nw_chance):
    return f"""    size:
            - {dimension}
            - {dimension}
        nw_range:
            - {nw_spread[0]}
            - {nw_spread[1]}
        node_weight_probability: {nw_chance}
        ew_range:
            - 2.0
            - 5.0
    """


template = Template("""---
experiment:
    finished: false
    seed: $exp_seed
    aggregation_rate: 1
    max_time: 480.
algorithm:
$algo_cfg
graph_creation:
    seed: 0
$creation_cfg
""")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        prefix = '.'
    else:
        prefix = sys.argv[1]
    for directory in [
        "./cfgs/small",
        "./cfgs/medium",
        "./cfgs/large",
        "./cfgs/uncat"
    ]:
        if not os.path.exists(directory):
            os.makedirs(directory)

    for (algo, run, restart) in itertools.product(["random", "mm-aco", "aco"], range(0, 10), range(0, 10)):
        # This is done for standard parameters
        for (nw_spread, nw_chance) in itertools.product(enumerate([(10.0, 20.0), (10.0, 200.0), (10.0, 2000.0)]),
                                                        enumerate([0.2, 0.5, 0.8])):
            param_name = ''
            if algo == 'aco':
                algo_cfg = aco_cfg(1.0, 5.0, 0.9, 0.2, run, 30, 10000)
                param_name = 'a1.0b5.0r0.9q0.2c10000'
            elif algo == 'mm-aco':
                algo_cfg = mm_aco_cfg(
                    1.0, 5.0, 0.9, 0.2, 0.05, run, 30, 10000)
                param_name = 'a1.0b5.0r0.9q0.2p0.05c10000'
            elif algo == 'random':
                algo_cfg = random_cfg(run, 10000)
                param_name = 'c10000'

            for instance_path in glob.glob("./instances/*"):
                instance = instance_path.split(
                    "\\")[-1].split(".")[0]
                subfolder = "uncat"
                if instance.__contains__("Berlin") or instance.__contains__("Hamburg"):
                    subfolder = "large"
                elif instance.__contains__("Frankfurt"):
                    subfolder = "medium"
                elif instance.__contains__("Sydney") or instance.__contains__("Marburg") or instance.__contains__("Leipzig"):
                    subfolder = "small"

                filename = f"cfgs/{subfolder}/{algo}_{instance}\
_{nw_spread[0]}-{nw_chance[0]}_{param_name}_{run}s{restart}.yaml"
                with open(filename, 'w') as f:
                    f.write(template.substitute(exp_seed=restart, algo_cfg=algo_cfg,
                            creation_cfg=file_cfg(f'{prefix}/{filename}', nw_spread[1], nw_chance[1])))

            for dimension in [10, 20, 50, 100]:
                subfolder = "uncat"
                if dimension in [10, 20, 50]:
                    subfolder = "small"
                elif dimension in [100]:
                    subfolder = "medium"

                filename = f"cfgs/{subfolder}/{algo}_{dimension}x{dimension}\
_{nw_spread[0]}-{nw_chance[0]}_{param_name}_r{run}s{restart}.yaml"
                with open(filename, 'w') as f:
                    f.write(template.substitute(exp_seed=restart, algo_cfg=algo_cfg,
                            creation_cfg=generation_cfg(dimension, nw_spread[1], nw_chance[1])))
