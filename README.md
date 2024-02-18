# Using ACO to solve dynamic OP

## Table of contents
* [General info](#general-info)
* [Technologies](#technologies)
* [Setup](#setup)

## General info
This software provides a simulation framework for analyzing metaheuristics on the Dynamic Orienteering Problem on non-fully connected graphs for my Masters Thesis on the topic.
A Dynamic Orienteering Problem is similar to a regular Orienteering Problem, but the edge weights of the graph change during the runtime of the algorithm. 

There are multiple algorithms provided for benchmarking.
A naive random search is implemented as well as different versions of ant colony optimization algorithms (ACO, MM-ACO, ACS).
All of these algorithms are anytime algorithms meaning they can be stopped after any number of iterations and still produce a valid result.
	
## Technologies
Project is created with:
* [Rust](https://github.com/rust-lang/rust)
* [osmpbfreader](https://crates.io/crates/osmpbfreader) to parse OpenStreetMaps data.
* [Tera](https://crates.io/crates/tera) for export to SVG format.
* [Serde](https://crates.io/crates/serde) to serialize graphdata.
* [oorandom](https://crates.io/crates/oorandom) for random decision making in ACO, as well as for random graph generation.
* [getrandom](https://crates.io/crates/getrandom) for seeding of oorandom rngs.
* [num-traits](https://crates.io/crates/num-traits) specifically for specification of types with additive zero elements.
* [float-cmp](https://crates.io/crates/float-cmp) to compare float variables.

* [OpenStreetMap](https://www.openstreetmap.org) data used for testing on real world instances.
  All bundled data in the instances directory is provided by OpenStreetMap under the Open Data Commons Open Database License and as such is free to use and redistribute when including this disclaimer.
  For further information abour the usage of OpenStreetMap data visit https://www.openstreetmap.org/copyright  
	
## Setup
To setup an experiment create a configuration file in the `experiments` directory. 
Multiple examples are provided, showing the different configuration options.
Once that is done you can start your experiments with the following command (you may need to switch to the rust nightly toolchain first).

```
$ cargo run
```

This will then run all your supplied experiment configurations and log the results.

If you need to create a lot of experiments you can also use the supplied `experiment_gen.py` with your own parametersets.
