use super::Generate;
use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::rng::preseeded_rng64;
use crate::util::Max;

use num_traits::Zero;
use oorandom::Rand64;
use std::fmt::Debug;
use std::ops::Add;

pub struct StochasticBlock<'a, Nw, Ew>
where
    Nw: Clone,
    Ew: Clone,
{
    probability_matrix: Vec<Vec<f64>>,
    community_size: usize,
    nw_generator: &'a dyn Fn(Rand64) -> Nw,
    ew_generator: &'a dyn Fn(Rand64) -> Ew,
    rng: &'a mut Rand64,
}

impl<'a, Nw: Clone, Ew: Clone> StochasticBlock<'a, Nw, Ew> {
    pub fn new(
        probability_matrix: Vec<Vec<f64>>,
        community_size: usize,
        nw_generator: &'a dyn Fn(Rand64) -> Nw,
        ew_generator: &'a dyn Fn(Rand64) -> Ew,
        rng: &'a mut Rand64,
    ) -> StochasticBlock<'a, Nw, Ew> {
        StochasticBlock {
            probability_matrix,
            community_size,
            nw_generator,
            ew_generator,
            rng,
        }
    }
}

impl<'a, Nw: 'static + Copy, Ew: 'static + Copy> Generate<Nw, Ew> for StochasticBlock<'a, Nw, Ew>
where
    Nw: 'static + Copy,
    Ew: 'static + Copy + Debug + Max + Zero + Add + Ord,
{
    fn generate(&mut self) -> MatrixGraph<usize, Nw, Ew> {
        let size = self.community_size * self.probability_matrix.len();
        let mut graph = MatrixGraph::<usize, Nw, Ew>::with_size(size);
        let mut rng = preseeded_rng64();

        // Populate nodes with random weights in range.
        for i in 0..size {
            // Unwrapping is fine, because the graph was just created, so we cant insert duplicates.
            graph.add_node(i, (self.nw_generator)(*self.rng)).unwrap();
        }

        // Populate edges with given probablity and weight in specified range.
        for i in 0..size {
            for j in 0..size {
                if rng.rand_float()
                    <= self.probability_matrix[i % self.community_size][j % self.community_size]
                {
                    // Unwrapping is fine, because all nodes in the range were just created.
                    graph
                        .add_edge((i, j), (self.ew_generator)(*self.rng))
                        .unwrap();
                }
            }
        }

        graph
    }
}
