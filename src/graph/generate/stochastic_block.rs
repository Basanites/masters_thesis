use super::Generate;
use super::super::{WeightedGraph, MatrixGraph};

use rand::{thread_rng, Rng};

pub struct StochasticBlock<'a, Nw, Ew> where
    Nw: Clone, 
    Ew: Clone {
        probability_matrix: Vec<Vec<f64>>,
        community_size: usize,
        nw_generator: &'a dyn Fn() -> Nw,
        ew_generator: &'a dyn Fn() -> Ew,
}

impl<'a, Nw: Clone, Ew: Clone> StochasticBlock<'a, Nw, Ew> {
    pub fn new(probability_matrix: Vec<Vec<f64>>,
        community_size: usize, 
        nw_generator: &'a dyn Fn() -> Nw, 
        ew_generator: &'a dyn Fn() -> Ew)
        -> StochasticBlock<'a, Nw, Ew> {
        StochasticBlock {
            probability_matrix,
            community_size,
            nw_generator,
            ew_generator,
        }
    }
}

impl<'a, Nw: 'static + Clone, Ew: 'static + Clone> Generate<Nw, Ew> for StochasticBlock<'a, Nw, Ew> {
    fn generate(&self) -> Box<dyn WeightedGraph<Nw, Ew>> {
        let size = self.community_size * self.probability_matrix.len();
        let mut graph = MatrixGraph::<Nw, Ew>::with_size(size);
        let mut rng = thread_rng();

        // Populate nodes with random weights in range.
        for i in 0..size {
            // Unwrapping is fine, because the graph was just created, so we cant insert duplicates.
            graph.add_node(i, (self.nw_generator)()).unwrap();
        }

        // Populate edges with given probablity and weight in specified range.
        for i in 0..size {
            for j in 0..size {
                if rng.gen_range(0.0, 1.0) <= self.probability_matrix[i % self.community_size][j % self.community_size] {
                    // Unwrapping is fine, because all nodes in the range were just created.
                    graph.add_edge((i, j), (self.ew_generator)()).unwrap();
                }
            }
        }

        Box::new(graph)

    }
}