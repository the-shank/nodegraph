use nodegraph::{NodeGraph, NodeGraphBuilder, NodeGraphError};
use serde_json::Value;

trait MachineModule {
    fn name(&self) -> String;
    fn details(&self) -> String;
}

struct LightModule {
    id: u32,
    brightness: u8,
    color: String,
}

impl MachineModule for LightModule {
    fn name(&self) -> String {
        format!("LightModule_{}", self.id)
    }

    fn details(&self) -> String {
        format!("Brightness: {}, Color: {}", self.brightness, self.color)
    }
}

struct HvacModule {
    id: u32,
    temperature: f32,
    fan_speed: u8,
}

impl MachineModule for HvacModule {
    fn name(&self) -> String {
        format!("HvacModule_{}", self.id)
    }

    fn details(&self) -> String {
        format!(
            "Temperature: {}°C, Fan Speed: {}",
            self.temperature, self.fan_speed
        )
    }
}

macro_rules! machine_description {
    (
        groups: [$($group:ident),*],
        devices: {$($device:ident: [$($modules:expr),* $(,)?]),* $(,)?},
        connections: {$($conn_group:ident: [$($conn_device:ident),* $(,)?]),* $(,)?}
    ) => {{
        (|| -> Result<NodeGraph<String, String>, NodeGraphError> {
            let mut builder = NodeGraphBuilder::<String, String>::default();
            let mut device_mapping: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

            // Adding nodes for groups
            $(
                builder.add_node(stringify!($group).to_string(), [("type".to_string(), Value::String("Group".to_string()))].iter().cloned().collect())?;
            )*

            // Adding nodes for devices and maintaining a mapping of device identifier to module names
            $(
                let mut module_names = Vec::new();
                $(
                    let name = $modules.name();
                    module_names.push(name.clone());
                    builder.add_node(name, [("details".to_string(), Value::String($modules.details()))].iter().cloned().collect())?;
                )*
                device_mapping.insert(stringify!($device).to_string(), module_names);
            )*

            // Adding edges using the correct node names
            $(
                let source_name = stringify!($conn_group).to_string();
                $(
                    let targets = device_mapping.get(&stringify!($conn_device).to_string()).unwrap();
                    for target_name in targets {
                        builder.add_edge(source_name.clone(), target_name.clone())?;
                    }
                )*
            )*

            Ok(builder.build())
        })()
    }};
}

fn main() {
    let light_module_1 = LightModule {
        id: 1,
        brightness: 80,
        color: "White".to_string(),
    };
    let hvac_module_1 = HvacModule {
        id: 1,
        temperature: 22.5,
        fan_speed: 3,
    };
    let hvac_module_2 = HvacModule {
        id: 2,
        temperature: 20.0,
        fan_speed: 2,
    };

    let graph_result = machine_description! {
        groups: [group_1, group_2],
        devices: {
            device_1: [light_module_1, hvac_module_1],
            device_2: [hvac_module_2]
        },
        connections: {
            group_1: [device_1],
            group_2: [device_1, device_2]
        }
    };

    match graph_result {
        Ok(graph) => {
            println!("{graph}\n");

            // Now the nodegraph can be used for use cases like code generation,
            // where you might traverse the graph and use the node IDs to construct a JSON object
            // or fill out an ecs library's world, etc.

            let traversal_order = graph.traverse_dfs("group_1".to_string());

            // Extracting entities and their attributes
            let entities_json: serde_json::Value = graph
                .nodes
                .iter()
                .map(|(id, data)| (id.clone(), serde_json::json!(data)))
                .collect();

            // Extracting relationships
            let relationships_json: serde_json::Value = graph
                .edges
                .iter()
                .map(|adj_list| {
                    adj_list
                        .edges
                        .iter()
                        .map(|(source, targets)| (source.clone(), serde_json::json!(targets)))
                        .collect::<serde_json::Map<String, serde_json::Value>>()
                })
                .collect();

            // Constructing the final JSON object
            let json_output = serde_json::json!({
                "dfs_order": traversal_order,
                "entities": entities_json,
                "relationships": relationships_json
            });

            println!("{json_output:#?}");
        }
        Err(error) => {
            eprintln!("Error: {error:?}");
        }
    }
}
