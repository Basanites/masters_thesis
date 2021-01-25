#[derive(Debug, PartialEq)]
pub enum GraphError<IndexType> {
    MissingEdge((IndexType, IndexType)),
    DuplicateEdge((IndexType, IndexType)),
    MissingNode(IndexType),
    DuplicateNode(IndexType),
}
