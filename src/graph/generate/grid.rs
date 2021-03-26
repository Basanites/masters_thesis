use super::Generate;
use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::util::Max;

use decorum::R64;
use num_traits::Zero;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;

pub struct Grid<'a, Nw, Ew>
where
    Nw: Clone,
    Ew: Clone,
{
    size: (usize, usize),
    nw_generator: &'a mut dyn FnMut() -> Nw,
    ew_generator: &'a mut dyn FnMut() -> Ew,
    phantom: PhantomData<(Nw, Ew)>,
}

impl<'a, Nw: Clone, Ew: Clone> Grid<'a, Nw, Ew> {
    pub fn new(
        size: (usize, usize),
        nw_generator: &'a mut dyn FnMut() -> Nw,
        ew_generator: &'a mut dyn FnMut() -> Ew,
    ) -> Grid<'a, Nw, Ew> {
        Grid {
            size,
            nw_generator,
            ew_generator,
            phantom: PhantomData,
        }
    }
}

/// 'static lifetime needed here. See https://stackoverflow.com/questions/32625583/parameter-type-may-not-live-long-enough for explanation.
/// tldr: Any type without stored references satisfies any lifetime. Thus e.g. all primitives satisfy 'static.
impl<'a, Nw, Ew> Generate<Nw, Ew> for Grid<'a, Nw, Ew>
where
    Nw: 'static + Copy + Debug,
    Ew: 'static + Copy + Ord + Zero + Debug + Add + Max,
{
    fn generate(&mut self) -> MatrixGraph<usize, Nw, Ew> {
        let mut graph = MatrixGraph::<usize, Nw, Ew>::with_size(self.size.0 * self.size.1);

        // count is used to generate consecutive numbered ids.
        // This means we need to remember which id an abstract (i, j) edge corresponds to.
        // This is done via the id_map.
        let mut id_map = HashMap::new();
        let mut count = 0;
        for i in 0..self.size.0 {
            for j in 0..self.size.1 {
                id_map.insert((i, j), count);
                graph.add_node(count, (self.nw_generator)()).unwrap();
                count += 1;
            }
        }

        for i in 0..self.size.0 {
            for j in 0..self.size.1 {
                // add edge to right neighbor
                if i < self.size.0 - 1 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i + 1, j)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                // add edge to left neighbor
                if i > 0 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i - 1, j)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                // add edge to below neighbor
                if j < self.size.1 - 1 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i, j + 1)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                // add edge to above neighbor
                if j > 0 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i, j - 1)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                // add edge to right below neighbor
                if i < self.size.0 - 1 && j < self.size.1 - 1 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i + 1, j + 1)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                // add edge to above left neighbor
                if i > 0 && j > 0 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i - 1, j - 1)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GenericWeightedGraph;
    use crate::rng::preseeded_rng64;
    use std::cell::RefCell;

    #[test]
    fn constant_weighted_works() {
        let mut node_gen = || R64::from_inner(1.0);
        let mut edge_gen = || R64::from_inner(2.0);
        let mut gen = Grid::new((5, 5), &mut node_gen, &mut edge_gen);
        let graph = gen.generate();
        let nodes: Vec<(usize, &R64)> = graph.iter_nodes().collect();
        let edges: Vec<((usize, usize), &R64)> = graph.iter_edges().collect();

        assert_eq!(nodes.len(), 25, "A 5x5 grid graph should have 25 nodes.");
        assert_eq!(
            edges.len(),
            112,
            "A 5x5 triangular grid graph should have 112 edges."
        );
        for (_, weight) in edges.iter() {
            assert_eq!(
                **weight, 2.0,
                "All weights should have been initialized with the value 2.0."
            )
        }
        for (_, weight) in nodes.iter() {
            assert_eq!(
                **weight, 1.0,
                "All weights should have been initialized with the value 1.0"
            )
        }
    }

    #[test]
    fn random_weighted_works() {
        let mut node_rng = preseeded_rng64();
        let mut edge_rng = preseeded_rng64();
        let mut node_gen = || R64::from_inner(node_rng.rand_float());
        let mut edge_gen = || R64::from_inner(edge_rng.rand_float());
        let mut gen = Grid::new((5, 5), &mut node_gen, &mut edge_gen);
        let graph = gen.generate();
        let nodes: Vec<(usize, &R64)> = graph.iter_nodes().collect();
        let edges: Vec<((usize, usize), &R64)> = graph.iter_edges().collect();

        assert_eq!(nodes.len(), 25, "A 5x5 grid graph should have 25 nodes.");
        assert_eq!(
            edges.len(),
            112,
            "A 5x5 triangular grid graph should have 112 edges."
        );
    }
    #[test]
    fn random_weighted_same_rng_works() {
        let rc = RefCell::new(preseeded_rng64());
        let mut node_gen = || R64::from_inner(rc.borrow_mut().rand_float());
        let mut edge_gen = || R64::from_inner(rc.borrow_mut().rand_float());
        let mut gen = Grid::new((5, 5), &mut node_gen, &mut edge_gen);
        let graph = gen.generate();
        let nodes: Vec<(usize, &R64)> = graph.iter_nodes().collect();
        let edges: Vec<((usize, usize), &R64)> = graph.iter_edges().collect();

        assert_eq!(nodes.len(), 25, "A 5x5 grid graph should have 25 nodes.");
        assert_eq!(
            edges.len(),
            112,
            "A 5x5 triangular grid graph should have 112 edges."
        );
    }
}
