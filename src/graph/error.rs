use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum GraphError<IndexType: fmt::Debug + fmt::Display> {
    MissingEdge((IndexType, IndexType)),
    DuplicateEdge((IndexType, IndexType)),
    MissingNode(IndexType),
    DuplicateNode(IndexType),
}

impl<IndexType: fmt::Debug + fmt::Display> fmt::Display for GraphError<IndexType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEdge(edge) => write!(f, "Edge {:?} not in graph.", edge),
            Self::DuplicateEdge(edge) => write!(f, "Edge {:?} already in graph.", edge),
            Self::MissingNode(node) => write!(f, "Node {} not in graph.", node),
            Self::DuplicateNode(node) => write!(f, "Node {} already in graph.", node),
        }
    }
}

impl<IndexType: fmt::Debug + fmt::Display> Error for GraphError<IndexType> {}
