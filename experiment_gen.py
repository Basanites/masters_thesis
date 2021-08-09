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


def acs_cfg(alpha, beta, rho, q_0, t_0, seed, ant_count, constructions):
    iterations = math.ceil(constructions / ant_count)
    return f"""    alpha: {alpha}
    beta: {beta}
    rho: {rho}
    q_0: {q_0}
    t_0: {t_0}
    seed: {seed}
    ant_count: {ant_count}
    iterations: {iterations}"""


def mm_aco_cfg(alpha, beta, rho, p_best, seed, ant_count, constructions):
    iterations = math.ceil(constructions / ant_count)
    return f"""    alpha: {alpha}
    beta: {beta}
    rho: {rho}
    p_best: {p_best}
    seed: {seed}
    ant_count: {ant_count}
    iterations: {iterations}"""


def random_cfg(seed, constructions):
    return f"""    seed: {seed}
    iterations: {constructions}"""


def file_cfg(filename, nw_spread, nw_chance):
    return f"""    filename: \"{filename}\"
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
        - 5.0"""


def run_for_all_files(restart, algo, algo_cfg, nw_spread, nw_chance, param_name, prefix, cfg_folder='cfgs'):
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

        filename = f"{cfg_folder}/{subfolder}/{algo}_{instance}\
_{nw_spread[0]}-{nw_chance[0]}_{param_name}_r{run}s{restart}.yaml"
        with open(filename, 'w') as f:
            f.write(template.substitute(exp_seed=restart, algo_cfg=algo_cfg,
                                        creation_cfg=file_cfg(f'{prefix}/instances/{instance}.osm.pbf', nw_spread[1], nw_chance[1])))


def run_for_all_generated(restart, algo, algo_cfg, nw_spread, nw_chance, param_name, cfg_folder='cfgs'):
    for dimension in [10, 20, 50, 100]:
        subfolder = "uncat"
        if dimension in [10, 20, 50]:
            subfolder = "small"
        elif dimension in [100]:
            subfolder = "medium"

        filename = f"{cfg_folder}/{subfolder}/{algo}_{dimension}x{dimension}\
_{nw_spread[0]}-{nw_chance[0]}_{param_name}_r{run}s{restart}.yaml"
        with open(filename, 'w') as f:
            f.write(template.substitute(exp_seed=restart, algo_cfg=algo_cfg,
                    creation_cfg=generation_cfg(dimension, nw_spread[1], nw_chance[1])))


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


def run_as(alpha, beta, rho, ants, nw_chance, nw_spread, run, restart, prefix, constructions=10000, folder='cfgs'):
    algo_cfg = aco_cfg(alpha, beta, rho, 0.0, run, ants, constructions)
    param_name = f'a{alpha}b{beta}r{rho}n{ants}c{constructions}'

    run_for_all_files(restart, 'as', algo_cfg,
                      nw_spread, nw_chance, param_name, prefix, folder)
    run_for_all_generated(
        restart, 'as', algo_cfg, nw_spread,  nw_chance, param_name, folder)


def run_acs(alpha, beta, rho, q_0, t_0, ants, nw_chance, nw_spread, run, restart, prefix, constructions=10000, folder='cfgs'):
    algo_cfg = acs_cfg(alpha, beta, rho, q_0, t_0, run, ants, constructions)
    param_name = f'a{alpha}b{beta}r{rho}q{q_0}t{t_0}n{ants}c{constructions}'

    run_for_all_files(restart, 'acs', algo_cfg,
                      nw_spread, nw_chance, param_name, prefix, folder)
    run_for_all_generated(
        restart, 'acs', algo_cfg, nw_spread, nw_chance, param_name, folder)


def run_mmas(alpha, beta, rho, p_best, ants, nw_chance, nw_spread, run, restart, prefix, constructions=10000, folder='cfgs'):
    algo_cfg = mm_aco_cfg(
        alpha, beta, rho, p_best, run, ants, constructions)
    param_name = f'a{alpha}b{beta}r{rho}p{p_best}n{ants}c{constructions}'

    run_for_all_files(restart, 'mm-as', algo_cfg,
                      nw_spread, nw_chance, param_name, prefix, folder)
    run_for_all_generated(
        restart, 'mm-as', algo_cfg, nw_spread, nw_chance, param_name, folder)


def run_random(nw_chance, nw_spread, run, restart, prefix, constructions=10000, folder='cfgs'):
    algo_cfg = random_cfg(run, constructions)
    param_name = f'c{constructions}'
    run_for_all_files(restart, 'random', algo_cfg,
                      nw_spread, nw_chance, param_name, prefix, folder)
    run_for_all_generated(
        restart, 'random', algo_cfg, nw_spread, nw_chance, param_name, folder)


def run_as_default(nw_chance, nw_spread, run, restart, prefix):
    run_as(1.0, 1.0, 0.5, 100, nw_chance, nw_spread,
           run, restart, prefix, constructions=10000)


def run_acs_default(nw_chance, nw_spread, run, restart, prefix):
    run_acs(1.0, 2.0, 0.9, 0.9, 1.0/10000.0, 30, nw_chance,
            nw_spread, run, restart, prefix, constructions=10000)


def run_mmas_default(nw_chance, nw_spread, run, restart, prefix):
    run_mmas(1.0, 2.0, 0.8, 0.05, 25, nw_chance, nw_spread,
             run, restart, prefix, constructions=10000)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        prefix = '.'
    else:
        prefix = sys.argv[1]
    for directory in [
        "./cfgs/small",
        "./cfgs/medium",
        "./cfgs/large",
        "./cfgs/uncat",
        "./extended_cfgs/small",
        "./extended_cfgs/medium",
        "./extended_cfgs/large",
        "./extended_cfgs/uncat",
    ]:
        if not os.path.exists(directory):
            os.makedirs(directory)

    # standard parameter runs
    for (run, restart) in itertools.product(range(0, 10), range(0, 3)):
        for (nw_spread, nw_chance) in itertools.product(enumerate([(10.0, 20.0), (10.0, 200.0), (10.0, 2000.0)]),
                                                        enumerate([0.2, 0.5, 0.8])):
            run_as_default(nw_chance, nw_spread, run, restart, prefix)
            run_acs_default(nw_chance, nw_spread, run, restart, prefix)
            run_mmas_default(nw_chance, nw_spread, run, restart, prefix)
            run_random(nw_chance, nw_spread, run, restart,
                       prefix, constructions=10000)

    # exploration in parameter space
    for (run, restart) in itertools.product(range(0, 10), range(0, 1)):
        nw_spread = (1, (10.0, 200))
        nw_chance = (1, 0.5)
        alpha = 1.0

        # as parameter exploration
        for (beta, rho, ants) in [x for x in itertools.product([1.0, 2.0, 8.0, 0.0], [0.5, 0.8, 0.7], [100, 10, 25]) if x != (1.0, 0.5, 100)]:
            run_as(alpha, beta, rho, ants, nw_chance, nw_spread,
                   run, restart, prefix, folder='extended_cfgs')

        # acs parameter exploration
        for (beta, rho, q_0, ants) in [x for x in itertools.product([5.0, 2.0, 8.0, 0.0], [0.9, 0.8, 0.7], [0.8, 0.9, 0.7], [30, 10, 25]) if x != (2.0, 0.9, 0.9, 30)]:
            run_acs(alpha, beta, rho, q_0, 1.0/10000.0, ants, nw_chance, nw_spread,
                    run, restart, prefix, folder='extended_cfgs')

        # mmas parameter exploration
        for (beta, rho, p_best, ants) in [x for x in itertools.product([2.0, 5.0, 8.0, 0.0], [0.8, 0.9, 0.7], [0.05, 0.005, 0.1], [25, 10, 30]) if x != (2.0, 0.8, 0.05, 25)]:
            run_mmas(alpha, beta, rho, p_best, ants, nw_chance,
                     nw_spread, run, restart, prefix, folder='extended_cfgs')

    # confirmation of found best parameters as well as standard parameters
    for (run, restart) in itertools.product(range(0, 20), range(0, 5)):
        nw_spread = (1, (10.0, 200))
        nw_chance = (1, 0.5)

        # default params
        run_as_default(nw_chance, nw_spread, run, restart, prefix)
        run_acs_default(nw_chance, nw_spread, run, restart, prefix)
        run_mmas_default(nw_chance, nw_spread, run, restart, prefix)
        run_random(nw_chance, nw_spread, run, restart,
                   prefix, constructions=10000)

        # new best params
        run_as(1.0, 1.0, 0.5, 25, nw_chance, nw_spread, run, restart, prefix)
        run_acs(1.0, 2.0, 0.7, 0.8, 1.0/10000.0, 10, nw_chance,
                nw_spread, run, restart, prefix)
        run_mmas(1.0, 2.0, 0.8, 0.05, 25, nw_chance,
                 nw_spread, run, restart, prefix)
