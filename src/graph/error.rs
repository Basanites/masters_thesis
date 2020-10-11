#[derive(Debug, PartialEq)]
pub enum GraphError {
    MissingEdge((usize, usize)),
    DuplicateEdge((usize, usize)),
    MissingNode(usize),
    DuplicateNode(usize),
}
