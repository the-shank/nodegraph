use std::error::Error;

#[derive(Debug)]
pub enum NodeGraphError {
    NodeAlreadyExists,
    NodeNotFound,
    EdgeError,
    SerializationError(String),
    DeserializationError(String),
}

// impl std::fmt::Display for NodeGraphError {
impl core::fmt::Display for NodeGraphError {
    // fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            NodeGraphError::NodeAlreadyExists => {
                write!(f, "Node with this ID already exists")
            }
            NodeGraphError::NodeNotFound => write!(f, "Node with this ID does not exist"),
            NodeGraphError::EdgeError => write!(f, "One of the node IDs does not exist"),
            NodeGraphError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            NodeGraphError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
        }
    }
}

impl Error for NodeGraphError {}
