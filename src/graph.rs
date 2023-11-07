extern crate alloc;

use crate::NodeGraphError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
// use std::any::Any;
use core::any::Any;
use std::collections::{HashMap, VecDeque};
use std::error::Error;

pub type ComponentMap<K> = HashMap<K, serde_json::Value>;
pub type NodeMap<ID, K> = HashMap<ID, ComponentMap<K>>;
pub type EdgeMap<ID> = AdjacencyList<ID>;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
// pub struct NodeGraph<ID: Eq + std::hash::Hash + Clone, K: Eq + std::hash::Hash + Clone> {
pub struct NodeGraph<ID: Eq + core::hash::Hash + Clone, K: Eq + core::hash::Hash + Clone> {
    pub nodes: NodeMap<ID, K>,
    pub edges: Vec<EdgeMap<ID>>,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
// pub struct AdjacencyList<ID: Eq + std::hash::Hash + Clone> {
pub struct AdjacencyList<ID: Eq + core::hash::Hash + Clone> {
    pub edges: HashMap<ID, Vec<ID>>,
}

impl<ID, K> NodeGraph<ID, K>
where
    // ID: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + std::fmt::Display,
    // K: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + std::fmt::Display,
    ID: Eq + core::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + core::fmt::Display,
    K: Eq + core::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + core::fmt::Display,
{
    pub fn to_dot(&self) -> String {
        let mut dot_str = String::from("digraph G {\n");
        for adj_list in &self.edges {
            for (source_id, targets) in &adj_list.edges {
                for target_id in targets {
                    dot_str.push_str(&format!("\t\"{source_id}\" -> \"{target_id}\";\n"));
                }
            }
        }
        dot_str.push('}');
        dot_str
    }

    pub fn add_node(
        &mut self,
        id: ID,
        components: HashMap<K, serde_json::Value>,
    ) -> Result<(), NodeGraphError> {
        if self.nodes.contains_key(&id) {
            return Err(NodeGraphError::NodeAlreadyExists);
        }
        self.nodes.insert(id, components);
        Ok(())
    }

    pub fn remove_node(&mut self, id: &ID) {
        self.nodes.remove(id);
        for relation in &mut self.edges {
            relation.edges.remove(id);
        }
    }

    pub fn add_edge(&mut self, from: ID, to: ID) -> Result<(), NodeGraphError> {
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return Err(NodeGraphError::EdgeError);
        }

        let relationship_exists = self.edges.last_mut().is_some();
        if !relationship_exists {
            self.edges.push(AdjacencyList {
                edges: HashMap::new(),
            });
        }

        let relationship = self.edges.last_mut().unwrap();
        relationship.edges.entry(from).or_default().push(to);
        Ok(())
    }

    pub fn remove_edge(&mut self, from: ID, to: ID) -> Result<(), NodeGraphError> {
        // Check if both entities exist
        if !self.nodes.contains_key(&from) || !self.nodes.contains_key(&to) {
            return Err(NodeGraphError::EdgeError);
        }

        // Check if there's a relationship to remove from
        if let Some(relationship) = self.edges.last_mut() {
            if let Some(neighbors) = relationship.edges.get_mut(&from) {
                // Remove the target node from the list of neighbors
                if let Some(index) = neighbors.iter().position(|x| x == &to) {
                    neighbors.remove(index);
                }

                // If the node has no more neighbors, remove its entry
                if neighbors.is_empty() {
                    relationship.edges.remove(&from);
                }
            }
        }

        Ok(())
    }

    pub fn serialize(&self) -> Result<String, Box<dyn Error>> {
        serde_json::to_string(&self).map_err(Into::into)
    }

    pub fn deserialize_with_registry(
        data: &str,
        registry: &TypeRegistry,
    ) -> Result<Self, NodeGraphError> {
        let mut graph: Self = serde_json::from_str(data).map_err(|e| {
            NodeGraphError::DeserializationError(format!("Failed to deserialize graph: {}", e))
        })?;

        // Deserialize components
        for (_id, component_map) in graph.nodes.iter_mut() {
            for (type_name, value) in component_map.iter_mut() {
                match registry.deserialize_value(&type_name.to_string(), value) {
                    Ok(new_value) => *value = new_value,
                    Err(e) => {
                        return Err(NodeGraphError::DeserializationError(format!(
                            "Failed to deserialize component: {}",
                            e
                        )))
                    }
                }
            }
        }

        Ok(graph)
    }

    pub fn traverse_dfs(&self, start: ID) -> Option<Vec<ID>> {
        let mut visited = HashMap::new();
        let mut stack = vec![start];
        let mut result = Vec::new();

        while let Some(current) = stack.pop() {
            if !visited.contains_key(&current) {
                visited.insert(current.clone(), true);
                result.push(current.clone());

                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        if !visited.contains_key(neighbor) {
                            stack.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn traverse_bfs(&self, start: ID) -> Option<Vec<ID>> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(start.clone());
        visited.insert(start.clone(), true);

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            if let Some(neighbors) = self.get_neighbors(&current) {
                for neighbor in neighbors {
                    if !visited.contains_key(neighbor) {
                        visited.insert(neighbor.clone(), true);
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    pub fn get_neighbors(&self, node_id: &ID) -> Option<&Vec<ID>> {
        for relationship in &self.edges {
            if let Some(neighbors) = relationship.edges.get(node_id) {
                return Some(neighbors);
            }
        }
        None
    }

    pub fn get_component(&self, node_id: &ID, component_key: &K) -> Option<&serde_json::Value> {
        self.nodes
            .get(node_id)
            .and_then(|components| components.get(component_key))
    }

    pub fn get_component_as<T: DeserializeOwned>(
        &self,
        node_id: &ID,
        component_key: &K,
    ) -> Option<T> {
        self.get_component(node_id, component_key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    pub fn get_parent(&self, id: &ID) -> Option<ID> {
        for relationship in &self.edges {
            for (source_id, targets) in &relationship.edges {
                if targets.contains(id) {
                    return Some(source_id.clone());
                }
            }
        }
        None
    }
}

// impl<ID, K> std::fmt::Display for NodeGraph<ID, K>
impl<ID, K> core::fmt::Display for NodeGraph<ID, K>
where
    // ID: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + std::fmt::Display,
    // K: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + std::fmt::Display,
    ID: Eq + core::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + core::fmt::Display,
    K: Eq + core::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + core::fmt::Display,
{
    // fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.to_dot())
    }
}

pub type SerializationFunction =
    Box<dyn Fn(&serde_json::Value) -> Result<Box<dyn Any + Send>, String>>;

pub type DeserializationFunction = Box<dyn Fn(&(dyn Any + Send)) -> Option<serde_json::Value>>;

#[derive(Default)]
pub struct TypeRegistry {
    deserialize_fn_map: HashMap<String, SerializationFunction>,
    serialize_map: HashMap<String, DeserializationFunction>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // Register a type with its serialization function
    pub fn register<T: 'static + Send + Serialize + DeserializeOwned>(&mut self, type_name: &str) {
        self.serialize_map.insert(
            type_name.to_string(),
            Box::new(move |any: &(dyn Any + Send)| {
                any.downcast_ref::<T>()
                    .and_then(|typed_ref| serde_json::to_value(typed_ref).ok())
            }),
        );

        self.deserialize_fn_map.insert(
            type_name.to_string(),
            Box::new(move |value: &serde_json::Value| {
                serde_json::from_value::<T>(value.clone())
                    .map(|value| Box::new(value) as Box<dyn Any + Send>)
                    .map_err(|e| e.to_string())
            }),
        );
    }

    pub fn deserialize_value(
        &self,
        type_name: &str,
        value: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Deserialize using the appropriate function from the map
        if let Some(deserialize_fn) = self.deserialize_fn_map.get(type_name) {
            let deserialized_value = deserialize_fn(value);

            // Attempt to re-serialize the deserialized value
            if let Some(serialize_fn) = self.serialize_map.get(type_name) {
                serialize_fn(&*deserialized_value?)
                    .ok_or_else(|| format!("Failed to re-serialize for: {}", type_name))
            } else {
                Err(format!(
                    "No serialization function found for type: {}",
                    type_name
                ))
            }
        } else {
            Err(format!(
                "No deserialization function found for type: {}",
                type_name
            ))
        }
    }
}

#[derive(Default)]
// pub struct NodeGraphBuilder<ID: Eq + std::hash::Hash + Clone, K: Eq + std::hash::Hash + Clone> {
pub struct NodeGraphBuilder<ID: Eq + core::hash::Hash + Clone, K: Eq + core::hash::Hash + Clone> {
    graph: NodeGraph<ID, K>,
}

impl<ID, K> NodeGraphBuilder<ID, K>
where
    // ID: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + std::fmt::Display,
    // K: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + std::fmt::Display,
    ID: Eq + core::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + core::fmt::Display,
    K: Eq + core::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de> + core::fmt::Display,
{
    pub fn add_node(
        &mut self,
        id: ID,
        components: HashMap<K, serde_json::Value>,
    ) -> Result<&mut Self, NodeGraphError> {
        self.graph.add_node(id, components)?;
        Ok(self)
    }

    pub fn add_edge(&mut self, from: ID, to: ID) -> Result<&mut Self, NodeGraphError> {
        self.graph.add_edge(from, to)?;
        Ok(self)
    }

    pub fn remove_node(&mut self, id: &ID) -> &mut Self {
        self.graph.remove_node(id);
        self
    }

    pub fn build(self) -> NodeGraph<ID, K> {
        self.graph
    }
}

#[macro_export]
macro_rules! register_types {
    ($registry:expr, $(($t:ty, $s:expr)),* ) => {
        $(
            $registry.register::<$t>($s);
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_add_remove_node() {
        let mut graph = NodeGraph::<String, String>::default();
        assert!(graph
            .add_node(
                "node1".to_string(),
                vec![
                    ("component_name1".to_string(), Value::from("component1")),
                    ("component_name2".to_string(), Value::from("component2"))
                ]
                .into_iter()
                .collect()
            )
            .is_ok());
        assert!(graph
            .add_node(
                "node1".to_string(),
                vec![("component_name3".to_string(), Value::from("component3"))]
                    .into_iter()
                    .collect()
            )
            .is_err());

        graph.remove_node(&"node1".to_string());
        assert!(!graph.nodes.contains_key(&"node1".to_string()));
    }

    #[test]
    fn test_add_edge() {
        let mut graph = NodeGraph::<String, String>::default();
        graph
            .add_node(
                "node1".to_string(),
                vec![("component_name1".to_string(), Value::from("component1"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph
            .add_node(
                "node2".to_string(),
                vec![("component_name2".to_string(), Value::from("component2"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();

        assert!(graph
            .add_edge("node1".to_string(), "node2".to_string())
            .is_ok());
        assert!(graph
            .add_edge("node1".to_string(), "node3".to_string())
            .is_err());
    }

    #[test]
    fn test_remove_edge() {
        let mut graph = NodeGraph::<String, String>::default();

        // Add entities
        graph.add_node("node1".to_string(), HashMap::new()).unwrap();
        graph.add_node("node2".to_string(), HashMap::new()).unwrap();

        // Add an edge
        graph
            .add_edge("node1".to_string(), "node2".to_string())
            .unwrap();

        // Assert edge exists
        assert!(graph.get_neighbors(&"node1".to_string()).is_some());

        // Remove the edge
        graph
            .remove_edge("node1".to_string(), "node2".to_string())
            .unwrap();

        // Assert edge is removed
        assert!(graph.get_neighbors(&"node1".to_string()).is_none());
    }

    // Mock ECS setup
    mod mock_ecs {
        use serde_json::Value;
        use std::collections::HashMap;

        #[derive(Default)]
        pub struct World {
            pub entities: Vec<Node>,
        }

        #[derive(Default)]
        pub struct Node {
            pub components: HashMap<String, Value>,
        }

        impl World {
            pub fn new() -> Self {
                World {
                    entities: Vec::new(),
                }
            }

            pub fn create_node(&mut self) -> &mut Node {
                self.entities.push(Node::default());
                self.entities.last_mut().unwrap()
            }
        }

        impl Node {
            pub fn add_component(&mut self, key: &str, component: Value) {
                self.components.insert(key.to_string(), component);
            }
        }
    }

    #[test]
    fn test_populate_mock_ecs_with_node_graph() {
        let mut graph = NodeGraph::<String, String>::default();
        graph
            .add_node(
                "node1".to_string(),
                vec![
                    ("position".to_string(), Value::from("x:10, y:20")),
                    ("velocity".to_string(), Value::from("dx:5, dy:-5")),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let mut world = mock_ecs::World::new();

        for components in graph.nodes.values() {
            let node = world.create_node();
            for (component_name, component_data) in components {
                node.add_component(component_name, component_data.clone());
            }
        }

        assert_eq!(world.entities.len(), 1);
        let mock_node = &world.entities[0];
        assert_eq!(
            mock_node.components.get("position").unwrap(),
            &Value::from("x:10, y:20")
        );
        assert_eq!(
            mock_node.components.get("velocity").unwrap(),
            &Value::from("dx:5, dy:-5")
        );
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    pub struct Component5 {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_serialization_and_deserialization() {
        let mut graph = NodeGraph::<String, String>::default();
        graph
            .add_node(
                "node1".to_string(),
                vec![
                    ("component_name1".to_string(), Value::from("component1")),
                    ("component_name2".to_string(), Value::from(1234)),
                    ("component_name3".to_string(), Value::from(true)),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        graph
            .add_node(
                "node2".to_string(),
                vec![("component_name4".to_string(), Value::from(5.67))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph
            .add_edge("node1".to_string(), "node2".to_string())
            .unwrap();
        // Create an instance of Component5 and serialize it as a component for an node
        let comp5 = Component5 {
            field1: "some_data".to_string(),
            field2: 42,
        };
        graph
            .add_node(
                "node3".to_string(),
                vec![(
                    "component_name5".to_string(),
                    serde_json::to_value(comp5).unwrap(),
                )]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let serialized = graph.serialize().unwrap();

        // Here we set up the type registry for deserialization
        let mut registry = TypeRegistry::new();
        register_types!(
            registry,
            (String, "component_name1"),
            (i32, "component_name2"),
            (bool, "component_name3"),
            (f64, "component_name4"),
            (Component5, "component_name5")
        );

        let deserialized =
            NodeGraph::<String, String>::deserialize_with_registry(&serialized, &registry).unwrap();

        assert_eq!(graph, deserialized);
    }

    #[test]
    fn test_dfs_traversal() {
        let mut graph = NodeGraph::<String, String>::default();

        // Adding entities
        graph.add_node("A".to_string(), HashMap::new()).unwrap();
        graph.add_node("B".to_string(), HashMap::new()).unwrap();
        graph.add_node("C".to_string(), HashMap::new()).unwrap();
        graph.add_node("D".to_string(), HashMap::new()).unwrap();

        // Adding edges
        graph.add_edge("A".to_string(), "B".to_string()).unwrap();
        graph.add_edge("A".to_string(), "C".to_string()).unwrap();
        graph.add_edge("B".to_string(), "D".to_string()).unwrap();

        let traversal_result = graph.traverse_dfs("A".to_string()).unwrap();
        let expected_traversal = vec![
            "A".to_string(),
            "C".to_string(),
            "B".to_string(),
            "D".to_string(),
        ];

        assert_eq!(traversal_result, expected_traversal);
    }

    #[test]
    fn test_bfs_traversal() {
        let mut graph = NodeGraph::<String, String>::default();

        // Adding entities
        graph.add_node("A".to_string(), HashMap::new()).unwrap();
        graph.add_node("B".to_string(), HashMap::new()).unwrap();
        graph.add_node("C".to_string(), HashMap::new()).unwrap();
        graph.add_node("D".to_string(), HashMap::new()).unwrap();

        // Adding edges
        graph.add_edge("A".to_string(), "B".to_string()).unwrap();
        graph.add_edge("A".to_string(), "C".to_string()).unwrap();
        graph.add_edge("B".to_string(), "D".to_string()).unwrap();

        let traversal_result = graph.traverse_bfs("A".to_string()).unwrap();
        let expected_traversal = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];

        assert_eq!(traversal_result, expected_traversal);
    }

    #[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
    enum ComponentKey {
        #[default]
        Position,
        Velocity,
    }

    // impl std::fmt::Display for ComponentKey {
    impl core::fmt::Display for ComponentKey {
        // fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[test]
    fn test_enum_key() {
        let mut graph = NodeGraph::<String, ComponentKey>::default();
        graph
            .add_node(
                "node1".to_string(),
                vec![
                    (
                        ComponentKey::Position,
                        serde_json::Value::from("x:10, y:20"),
                    ),
                    (
                        ComponentKey::Velocity,
                        serde_json::Value::from("dx:5, dy:-5"),
                    ),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();

        let position = graph.get_component(&"node1".to_string(), &ComponentKey::Position);
        assert_eq!(position, Some(&serde_json::Value::from("x:10, y:20")));
    }

    #[test]
    fn test_dfs_print_components() {
        // Create an node graph and add entities with components
        let mut graph = NodeGraph::<String, String>::default();
        graph
            .add_node(
                "A".to_string(),
                vec![
                    ("type1".to_string(), serde_json::Value::from("data1")),
                    ("type2".to_string(), serde_json::Value::from(123)),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap();
        graph
            .add_node(
                "B".to_string(),
                vec![("type1".to_string(), serde_json::Value::from("data2"))]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
        graph.add_node("C".to_string(), HashMap::new()).unwrap();

        // Add edges for traversal
        graph.add_edge("A".to_string(), "B".to_string()).unwrap();
        graph.add_edge("A".to_string(), "C".to_string()).unwrap();

        // Set up the type registry
        let mut registry = TypeRegistry::new();
        registry.register::<String>("type1");
        registry.register::<i32>("type2");

        // Perform DFS traversal and print components
        let traversal_result = graph.traverse_dfs("A".to_string()).unwrap();

        // Expected traversal order and component count
        let expected_order = vec!["A", "C", "B"];
        let expected_components = [2, 0, 1];

        // Check traversal order
        assert_eq!(traversal_result, expected_order);

        for (index, node_id) in traversal_result.iter().enumerate() {
            if let Some(components) = graph.nodes.get(node_id) {
                // Assert the number of components
                assert_eq!(components.len(), expected_components[index]);

                // Bonus: Check component types
                for (type_name, value) in components {
                    assert!(
                        registry.deserialize_value(type_name, value).is_ok(),
                        "Component of type {} is NOT of expected type.",
                        type_name
                    );
                }
            }
        }
    }

    #[test]
    fn test_builder_pattern() {
        let mut builder = NodeGraphBuilder::<String, String>::default();

        builder
            .add_node(
                "node1".to_string(),
                vec![
                    (
                        "component_name1".to_string(),
                        serde_json::Value::from("component1"),
                    ),
                    (
                        "component_name2".to_string(),
                        serde_json::Value::from("component2"),
                    ),
                ]
                .into_iter()
                .collect(),
            )
            .unwrap()
            .add_node(
                "node2".to_string(),
                vec![(
                    "component_name3".to_string(),
                    serde_json::Value::from("component3"),
                )]
                .into_iter()
                .collect(),
            )
            .unwrap()
            .add_edge("node1".to_string(), "node2".to_string())
            .unwrap();

        let graph = builder.build();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_get_component_as() -> Result<(), NodeGraphError> {
        let mut graph = NodeGraph::<String, String>::default();
        graph.add_node(
            "node1".to_string(),
            vec![
                (
                    "component_str".to_string(),
                    serde_json::Value::from("value1"),
                ),
                ("component_int".to_string(), serde_json::Value::from(451)),
            ]
            .into_iter()
            .collect(),
        )?;

        assert_eq!(
            graph.get_component_as::<String>(&"node1".to_string(), &"component_str".to_string()),
            Some("value1".to_string())
        );
        assert_eq!(
            graph.get_component_as::<i32>(&"node1".to_string(), &"component_int".to_string()),
            Some(451)
        );

        // Test for a type mismatch
        assert!(graph
            .get_component_as::<i32>(&"node1".to_string(), &"component_str".to_string())
            .is_none());

        Ok(())
    }

    #[test]
    fn test_get_parent() {
        let mut graph = NodeGraph::<String, String>::default();

        // Add nodes
        graph.add_node("Root".to_string(), HashMap::new()).unwrap();
        graph
            .add_node("Child1".to_string(), HashMap::new())
            .unwrap();
        graph
            .add_node("Child2".to_string(), HashMap::new())
            .unwrap();

        // Add edges (representing parent-child relationships)
        graph
            .add_edge("Root".to_string(), "Child1".to_string())
            .unwrap();
        graph
            .add_edge("Root".to_string(), "Child2".to_string())
            .unwrap();

        // Test get_parent method
        assert_eq!(
            graph.get_parent(&"Child1".to_string()),
            Some("Root".to_string())
        );
        assert_eq!(
            graph.get_parent(&"Child2".to_string()),
            Some("Root".to_string())
        );
        assert_eq!(graph.get_parent(&"Root".to_string()), None); // Root has no parent
    }
}
