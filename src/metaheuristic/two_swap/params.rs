use crate::metaheuristic::Heuristic;

pub struct Params<'a, Nw, Ew> {
    pub heuristic: &'a Heuristic<Nw, Ew>,
}

impl<'a, Nw, Ew> Params<'a, Nw, Ew> {
    pub fn new(heuristic: &'a Heuristic<Nw, Ew>) -> Self {
        Params { heuristic }
    }
}
