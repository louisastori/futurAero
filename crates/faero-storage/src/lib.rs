use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use faero_types::{
    CommandEnvelope, EntityRecord, EventEnvelope, ExternalEndpoint, GraphEdge, PluginManifest,
    ProjectDocument, ProjectMetadata, TelemetryStream,
};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub fn save_project(
    path: impl AsRef<Path>,
    document: &ProjectDocument,
) -> Result<(), StorageError> {
    let root = path.as_ref();
    fs::create_dir_all(root.join("graph/nodes"))?;
    fs::create_dir_all(root.join("integration/endpoints"))?;
    fs::create_dir_all(root.join("integration/streams"))?;
    fs::create_dir_all(root.join("plugins/manifests"))?;
    fs::create_dir_all(root.join("plugins/state"))?;
    fs::create_dir_all(root.join("events"))?;

    write_yaml(root.join("project.yaml"), &document.metadata)?;

    for node in document.nodes.values() {
        write_json(
            root.join("graph/nodes").join(format!("{}.json", node.id)),
            node,
        )?;
    }
    write_jsonl(root.join("graph/edges.jsonl"), &document.edges)?;

    for endpoint in document.endpoints.values() {
        write_json(
            root.join("integration/endpoints")
                .join(format!("{}.json", endpoint.id)),
            endpoint,
        )?;
    }
    for stream in document.streams.values() {
        write_json(
            root.join("integration/streams")
                .join(format!("{}.json", stream.id)),
            stream,
        )?;
    }
    for manifest in document.plugin_manifests.values() {
        write_json(
            root.join("plugins/manifests")
                .join(format!("{}.json", manifest.plugin_id)),
            manifest,
        )?;
    }
    write_json(
        root.join("plugins/state/plugins.json"),
        &document.plugin_states,
    )?;
    write_jsonl(root.join("events/commands.jsonl"), &document.commands)?;
    write_jsonl(root.join("events/events.jsonl"), &document.events)?;

    Ok(())
}

pub fn load_project(path: impl AsRef<Path>) -> Result<ProjectDocument, StorageError> {
    let root = path.as_ref();
    let metadata: ProjectMetadata = read_yaml(root.join("project.yaml"))?;
    let nodes = read_json_dir::<EntityRecord>(root.join("graph/nodes"))?
        .into_iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let edges = read_jsonl::<GraphEdge>(root.join("graph/edges.jsonl"))?;
    let endpoints = read_json_dir::<ExternalEndpoint>(root.join("integration/endpoints"))?
        .into_iter()
        .map(|endpoint| (endpoint.id.clone(), endpoint))
        .collect::<BTreeMap<_, _>>();
    let streams = read_json_dir::<TelemetryStream>(root.join("integration/streams"))?
        .into_iter()
        .map(|stream| (stream.id.clone(), stream))
        .collect::<BTreeMap<_, _>>();
    let plugin_manifests = read_json_dir::<PluginManifest>(root.join("plugins/manifests"))?
        .into_iter()
        .map(|manifest| (manifest.plugin_id.clone(), manifest))
        .collect::<BTreeMap<_, _>>();
    let plugin_states =
        read_json::<BTreeMap<String, bool>>(root.join("plugins/state/plugins.json"))
            .unwrap_or_default();
    let commands = read_jsonl::<CommandEnvelope>(root.join("events/commands.jsonl"))?;
    let events = read_jsonl::<EventEnvelope>(root.join("events/events.jsonl"))?;

    Ok(ProjectDocument {
        metadata,
        nodes,
        edges,
        endpoints,
        streams,
        plugin_manifests,
        plugin_states,
        commands,
        events,
    })
}

fn write_json<T: serde::Serialize>(path: PathBuf, value: &T) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)?;
    fs::write(path, payload)?;
    Ok(())
}

fn write_yaml<T: serde::Serialize>(path: PathBuf, value: &T) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_yaml::to_string(value)?;
    fs::write(path, payload)?;
    Ok(())
}

fn write_jsonl<T: serde::Serialize>(path: PathBuf, values: &[T]) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    for value in values {
        serde_json::to_writer(&mut file, value)?;
        writeln!(&mut file)?;
    }
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: PathBuf) -> Result<T, StorageError> {
    let payload = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&payload)?)
}

fn read_yaml<T: DeserializeOwned>(path: PathBuf) -> Result<T, StorageError> {
    let payload = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&payload)?)
}

fn read_jsonl<T: DeserializeOwned>(path: PathBuf) -> Result<Vec<T>, StorageError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut values = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        values.push(serde_json::from_str::<T>(&line)?);
    }
    Ok(values)
}

fn read_json_dir<T: DeserializeOwned>(path: PathBuf) -> Result<Vec<T>, StorageError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    let mut values = Vec::new();
    for entry in entries {
        if entry.file_type()?.is_file() {
            let payload = fs::read_to_string(entry.path())?;
            values.push(serde_json::from_str::<T>(&payload)?);
        }
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use faero_types::{
        Addressing, ConnectionMode, DisplayUnits, EndpointType, LinkMetrics, PluginManifest,
        ProjectDocument, ProjectMetadata, QosProfile, StreamDirection, TelemetryStream,
        TimingProfile, TransportProfile,
    };
    use tempfile::tempdir;

    use super::*;

    fn sample_document() -> ProjectDocument {
        let mut document = ProjectDocument {
            metadata: ProjectMetadata {
                project_id: "prj_9999".to_string(),
                name: "Fixture".to_string(),
                format_version: "0.1.0".to_string(),
                created_at: "2026-04-06T00:00:00Z".to_string(),
                updated_at: "2026-04-06T00:00:00Z".to_string(),
                app_version: "0.1.0-alpha".to_string(),
                display_units: DisplayUnits::default(),
                default_frame: "world".to_string(),
                root_scene_id: Some("ent_cell_001".to_string()),
                active_configuration_id: "cfg_default".to_string(),
            },
            ..ProjectDocument::default()
        };
        document.nodes.insert(
            "ent_cell_001".to_string(),
            EntityRecord {
                id: "ent_cell_001".to_string(),
                entity_type: "RobotCell".to_string(),
                name: "Cellule Demo".to_string(),
                revision: "rev_0001".to_string(),
                status: "active".to_string(),
                data: serde_json::json!({ "robotIds": ["ent_robot_001"] }),
            },
        );
        document.edges.push(GraphEdge {
            edge_id: "edg_0001".to_string(),
            from: "ent_cell_001".to_string(),
            to: "ent_robot_001".to_string(),
            relation_type: "contains".to_string(),
            created_at: "2026-04-06T00:00:00Z".to_string(),
        });
        document.endpoints.insert(
            "ext_wifi_001".to_string(),
            ExternalEndpoint {
                id: "ext_wifi_001".to_string(),
                name: "WiFi Edge".to_string(),
                endpoint_type: EndpointType::WifiDevice,
                transport_profile: TransportProfile {
                    transport_kind: "wifi".to_string(),
                    adapter_id: Some("wlan0".to_string()),
                    discovery_mode: Some("mdns".to_string()),
                    credential_policy: Some("runtime_prompt".to_string()),
                    security_mode: Some("wpa3".to_string()),
                },
                connection_profile: serde_json::json!({ "reconnect": true }),
                addressing: Addressing {
                    host: Some("edge.local".to_string()),
                    port: Some(9001),
                    path: Some("/telemetry".to_string()),
                    device_id: None,
                },
                signal_map_ids: vec!["sig_001".to_string()],
                mode: ConnectionMode::Live,
                link_metrics: LinkMetrics {
                    latency_ms: Some(10),
                    jitter_ms: Some(2),
                    drop_rate: Some(0.0),
                    rssi_dbm: Some(-44),
                    bandwidth_kbps: Some(9000),
                },
                status: "connected".to_string(),
            },
        );
        document.streams.insert(
            "str_001".to_string(),
            TelemetryStream {
                id: "str_001".to_string(),
                name: "BumperStatus".to_string(),
                endpoint_id: "ext_wifi_001".to_string(),
                stream_type: "mqtt_topic".to_string(),
                direction: StreamDirection::Inbound,
                codec_profile: serde_json::json!({ "encoding": "json" }),
                schema_ref: "schemas/telemetry/bumper-status.schema.json".to_string(),
                timing_profile: TimingProfile {
                    expected_rate_hz: 20,
                    max_latency_ms: 80,
                },
                qos_profile: QosProfile {
                    delivery: "at_least_once".to_string(),
                    ordering: "best_effort".to_string(),
                },
                status: "ready".to_string(),
            },
        );
        document.plugin_manifests.insert(
            "plg.integration.viewer".to_string(),
            PluginManifest {
                id: "ent_plugin_001".to_string(),
                plugin_id: "plg.integration.viewer".to_string(),
                version: "0.1.0".to_string(),
                capabilities: vec!["panel".to_string()],
                permissions: vec!["project.read".to_string()],
                entrypoints: vec!["plugins/integration-viewer/index.js".to_string()],
                compatibility: vec!["faero-core@0.1".to_string()],
                status: "installed".to_string(),
            },
        );
        document
            .plugin_states
            .insert("plg.integration.viewer".to_string(), true);
        document
    }

    #[test]
    fn saves_and_loads_project_document_round_trip() {
        let dir = tempdir().expect("tempdir should be available");
        let project_root = dir.path().join("fixture.faero");
        let document = sample_document();

        save_project(&project_root, &document).expect("project should save");
        let loaded = load_project(&project_root).expect("project should load");

        assert_eq!(loaded.metadata.project_id, "prj_9999");
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.endpoints.len(), 1);
        assert_eq!(loaded.streams.len(), 1);
        assert_eq!(
            loaded.plugin_states.get("plg.integration.viewer"),
            Some(&true)
        );
    }
}
