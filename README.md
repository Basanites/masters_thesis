# Using ACO to solve dynamic OP [![Build Status](https://travis-ci.com/Basanites/masters_thesis.svg?branch=master)](https://travis-ci.com/Basanites/masters_thesis)

## Table of contents
* [General info](#general-info)
* [Technologies](#technologies)
* [Setup](#setup)

## General info
This software simulates an Ant Colony Optimization Algorithm to solve specific types of dynamic Orienteering Problems.

The instances in question may dynamically change their node and edge weights to any nonzero value during the runtime of the algorithm.

Because of that the comparative algorithm used is also an anytime algorithm, meaning you can stop it at any time and get a result.
	
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
	
## Setup
To run this project locally, execute:

```
$ cargo run
```