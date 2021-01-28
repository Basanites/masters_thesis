# Using ACO to solve dynamic OP [![Build Status](https://travis-ci.com/Basanites/masters_thesis.svg?token=QsL71Bx4xNLGumpACMGx&branch=master)](https://travis-ci.com/Basanites/masters_thesis)

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
* [Rand](https://crates.io/crates/rand) for creation of random graph instances as well as modification of existing ones.
* [float-cmp](https://crates.io/crates/float-cmp) to compare float variables.
	
## Setup
To run this project locally, execute:

```
$ cargo run
```
