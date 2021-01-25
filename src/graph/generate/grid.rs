use super::Generate;
use crate::graph::{regular::MatrixGraph, GenericWeightedGraph, WeightedGraph};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct Grid<'a, Nw, Ew>
where
    Nw: Clone,
    Ew: Clone,
{
    size: (usize, usize),
    nw_generator: &'a dyn Fn() -> Nw,
    ew_generator: &'a dyn Fn() -> Ew,
    phantom: PhantomData<(Nw, Ew)>,
}

impl<'a, Nw: Clone, Ew: Clone> Grid<'a, Nw, Ew> {
    pub fn new(
        size: (usize, usize),
        nw_generator: &'a dyn Fn() -> Nw,
        ew_generator: &'a dyn Fn() -> Ew,
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
impl<'a, Nw: 'static + Clone, Ew: 'static + Clone> Generate<Nw, Ew> for Grid<'a, Nw, Ew> {
    fn generate(&self) -> Box<dyn WeightedGraph<Nw, Ew>> {
        let mut graph = MatrixGraph::<Nw, Ew>::with_size(self.size.0 * self.size.1);

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
                if i < self.size.0 - 1 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i + 1, j)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                if i > 0 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i - 1, j)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                if j < self.size.1 - 1 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i, j + 1)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
                if j > 0 {
                    graph
                        .add_edge(
                            (id_map[&(i, j)], id_map[&(i, j - 1)]),
                            (self.ew_generator)(),
                        )
                        .unwrap();
                }
            }
        }

        Box::new(graph)
    }
}
