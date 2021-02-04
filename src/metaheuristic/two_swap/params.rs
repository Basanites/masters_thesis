use crate::metaheuristic::Heuristic;

pub struct Params<IndexType, Nw, Ew> {
    pub heuristic: Heuristic<IndexType, Nw, Ew>,
}

impl<IndexType, Nw, Ew> Params<IndexType, Nw, Ew> {
    pub fn new(heuristic: Heuristic<IndexType, Nw, Ew>) -> Self {
        Params { heuristic }
    }
}
