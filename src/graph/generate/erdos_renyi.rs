use super::Generate;
use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::rng::preseeded_rng64;

pub struct ErdosRenyi<'a, Nw, Ew>
where
    Nw: Clone,
    Ew: Clone,
{
    size: usize,
    connection_probability: f64,
    nw_generator: &'a mut dyn FnMut() -> Nw,
    ew_generator: &'a mut dyn FnMut() -> Ew,
}

impl<'a, Nw: Clone, Ew: Clone> ErdosRenyi<'a, Nw, Ew> {
    pub fn new(
        size: usize,
        connection_probability: f64,
        nw_generator: &'a mut dyn FnMut() -> Nw,
        ew_generator: &'a mut dyn FnMut() -> Ew,
    ) -> ErdosRenyi<'a, Nw, Ew> {
        ErdosRenyi {
            size,
            connection_probability,
            nw_generator,
            ew_generator,
        }
    }
}

impl<'a, Nw: 'static + Copy, Ew: 'static + Copy> Generate<Nw, Ew> for ErdosRenyi<'a, Nw, Ew> {
    fn generate(&mut self) -> MatrixGraph<usize, Nw, Ew> {
        let mut rng = preseeded_rng64();
        let mut graph = MatrixGraph::<usize, Nw, Ew>::with_size(self.size);

        // Populate nodes with random weights in range.
        for i in 0..self.size {
            // Unwrapping is fine, because the graph was just created, so we cant insert duplicates.
            graph.add_node(i, (self.nw_generator)()).unwrap();
        }

        // Populate edges with given probablity and weight in specified range.
        for i in 0..self.size {
            for j in 0..self.size {
                if rng.rand_float() <= self.connection_probability {
                    // Unwrapping is fine, because all nodes in the range were just created.
                    graph.add_edge((i, j), (self.ew_generator)()).unwrap();
                }
            }
        }

        graph
    }
}
