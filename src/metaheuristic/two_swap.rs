use crate::graph::{GenericWeightedGraph, Edge};

struct TwoSwap<'a, IndexType, Nw, Ew> {
    graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>,
    goal_point: IndexType,
    max_time: Ew,
    evaluator: &'a fn(Nw, Ew) -> f64,
    best_solution: Vec<Edge<IndexType>>,
    best_score: f64,
}

impl<'a, IndexType: Copy + std::cmp::PartialEq, Nw: Copy, Ew: Copy> TwoSwap<'a, IndexType, Nw, Ew> {
    pub fn new(graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>, goal_point: IndexType, max_time: Ew, evaluator: &'a fn(Nw, Ew) -> f64) -> Self {
        let mut swap = TwoSwap {
            graph,
            goal_point,
            max_time,
            evaluator,
            best_solution: Vec::new(),
            best_score: 0.0,
        };

        swap.initialize();
        swap
    }

    fn score(&self, node_weight: Nw, edge_weight: Ew) -> f64 {
        (self.evaluator)(node_weight, edge_weight)
    }

    fn score_edge(&self, from: IndexType, to: IndexType) -> f64 {
        self.score(*self.graph.node_weight(to).unwrap(), *self.graph.edge_weight((from, to)).unwrap())
    }

    fn score_with_known_edge(&self, to: IndexType, edge_weight: Ew) -> f64 {
        self.score(*self.graph.node_weight(to).unwrap(), edge_weight)
    }

    pub fn initialize(&mut self) {
        let max = self.graph.iter_neighbors(self.goal_point).unwrap()
            .map(|(id, weight)| (id, self.score_with_known_edge(id, *weight)))
            .max_by(|(_, ev_a), (_, ev_b)| ev_a.partial_cmp(ev_b).unwrap())
            .unwrap();
        let best_return = self.graph.iter_neighbors(max.0).unwrap()
            .map(|(id, weight)| (id, self.score_with_known_edge(id, *weight)))
            .max_by(|(_, ev_a), (_, ev_b)| ev_a.partial_cmp(ev_b).unwrap())
            .unwrap();

        self.best_solution.push((max.0, best_return.0));
        self.best_score = max.1 + best_return.1;
    }

    pub fn single_iteration(&mut self) -> Option<&Vec<Edge<IndexType>>> {
        let mut new_best = Vec::new();
        let mut max = 0.0;
        let mut score = 0.0;
        let mut temp_score = 0.0;
        for (from, to) in self.best_solution.iter() {
            let t_weight = self.graph.node_weight(*to).unwrap();
            max = self.score(*t_weight, *self.graph.edge_weight((*from, *to)).unwrap());
            let mut best_follow = *to;

            for (nid, weight) in self.graph.iter_neighbors(*from).unwrap() {
                temp_score = self.score_with_known_edge(nid, *weight);
                if let Ok(return_weight) = self.graph.edge_weight((nid, *to)) {
                    temp_score = temp_score + self.score(*t_weight, *return_weight);
                    if temp_score > max {
                        max = temp_score;
                        best_follow = nid;
                    }
                }
            }

            if best_follow != *to {
                new_best.push((*from, best_follow));
                new_best.push((best_follow, *to));
            } else {
                new_best.push((*from, *to));
            }
            score = score + max;
        }

        if score > self.best_score {
            self.best_solution = new_best;
            self.best_score = score;
            Some(&self.best_solution)
        } else {
            None
        }
    }
}

impl<'a, IndexType: Copy + std::cmp::PartialEq, Nw: Copy, Ew: Copy> Iterator for TwoSwap<'a, IndexType, Nw, Ew> {
    type Item = Vec<Edge<IndexType>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.single_iteration().cloned()
    }
}