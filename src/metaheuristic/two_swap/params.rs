use crate::metaheuristic::Heuristic;

pub struct Params<'a, IndexType, Nw, Ew> {
    pub heuristic: &'a Heuristic<IndexType, Nw, Ew>,
}

impl<'a, IndexType, Nw, Ew> Params<'a, IndexType, Nw, Ew> {
    pub fn new(heuristic: &'a Heuristic<IndexType, Nw, Ew>) -> Self {
        Params { heuristic }
    }
}
