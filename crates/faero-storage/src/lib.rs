use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use faero_types::{
    CommandEnvelope, EntityRecord, EventEnvelope, ExternalEndpoint, GraphEdge, PluginManifest,
    ProjectDocument, ProjectMetadata, TelemetryStream,
};
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

    write_project_metadata_yaml(root.join("project.yaml"), &document.metadata)?;

    for node in document.nodes.values() {
        let node_path = root.join("graph/nodes").join(format!("{}.json", node.id));
        write_entity_record(node_path, node)?;
    }
    write_graph_edges(root.join("graph/edges.jsonl"), &document.edges)?;

    for endpoint in document.endpoints.values() {
        let endpoint_path = root
            .join("integration/endpoints")
            .join(format!("{}.json", endpoint.id));
        write_external_endpoint(endpoint_path, endpoint)?;
    }
    for stream in document.streams.values() {
        let stream_path = root
            .join("integration/streams")
            .join(format!("{}.json", stream.id));
        write_telemetry_stream(stream_path, stream)?;
    }
    for manifest in document.plugin_manifests.values() {
        let manifest_path = root
            .join("plugins/manifests")
            .join(format!("{}.json", manifest.plugin_id));
        write_plugin_manifest_file(manifest_path, manifest)?;
    }
    write_plugin_states_file(
        root.join("plugins/state/plugins.json"),
        &document.plugin_states,
    )?;
    write_command_envelopes(root.join("events/commands.jsonl"), &document.commands)?;
    write_event_envelopes(root.join("events/events.jsonl"), &document.events)?;

    Ok(())
}

pub fn load_project(path: impl AsRef<Path>) -> Result<ProjectDocument, StorageError> {
    let root = path.as_ref();
    let metadata = read_project_metadata_yaml(root.join("project.yaml"))?;
    let nodes = read_entity_records_dir(root.join("graph/nodes"))?
        .into_iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let edges = read_graph_edges(root.join("graph/edges.jsonl"))?;
    let endpoints = read_external_endpoints_dir(root.join("integration/endpoints"))?
        .into_iter()
        .map(|endpoint| (endpoint.id.clone(), endpoint))
        .collect::<BTreeMap<_, _>>();
    let streams = read_telemetry_streams_dir(root.join("integration/streams"))?
        .into_iter()
        .map(|stream| (stream.id.clone(), stream))
        .collect::<BTreeMap<_, _>>();
    let plugin_manifests = read_plugin_manifests_dir(root.join("plugins/manifests"))?
        .into_iter()
        .map(|manifest| (manifest.plugin_id.clone(), manifest))
        .collect::<BTreeMap<_, _>>();
    let plugin_states =
        read_plugin_states_file(root.join("plugins/state/plugins.json")).unwrap_or_default();
    let commands = read_command_envelopes(root.join("events/commands.jsonl"))?;
    let events = read_event_envelopes(root.join("events/events.jsonl"))?;

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

fn ensure_parent_dir(path: &Path) -> Result<(), StorageError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    fs::create_dir_all(parent)?;
    Ok(())
}

fn write_text_file(path: PathBuf, payload: &str) -> Result<(), StorageError> {
    ensure_parent_dir(&path)?;
    fs::write(path, payload)?;
    Ok(())
}

fn write_jsonl_file(path: PathBuf, payloads: &[String]) -> Result<(), StorageError> {
    let mut joined_payloads = payloads.join("\n");
    if !joined_payloads.is_empty() {
        joined_payloads.push('\n');
    }
    write_text_file(path, &joined_payloads)
}

fn read_text_file(path: PathBuf) -> Result<String, StorageError> {
    Ok(fs::read_to_string(path)?)
}

fn read_jsonl_payloads(path: PathBuf) -> Result<Vec<String>, StorageError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let payload = read_text_file(path)?;
    Ok(payload
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect())
}

fn read_sorted_payload_files(path: PathBuf) -> Result<Vec<String>, StorageError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut entry_paths = Vec::new();
    for entry in fs::read_dir(path)?.flatten() {
        let entry_path = entry.path();
        if entry_path.is_file() {
            entry_paths.push(entry_path);
        }
    }
    entry_paths.sort();

    let mut payloads = Vec::with_capacity(entry_paths.len());
    for entry_path in entry_paths {
        payloads.push(fs::read_to_string(entry_path)?);
    }
    Ok(payloads)
}

fn write_project_metadata_yaml(
    path: PathBuf,
    metadata: &ProjectMetadata,
) -> Result<(), StorageError> {
    let payload = serde_yaml::to_string(metadata).expect("project metadata should serialize");
    write_text_file(path, &payload)
}

fn write_entity_record(path: PathBuf, record: &EntityRecord) -> Result<(), StorageError> {
    let payload = serde_json::to_string_pretty(record).expect("entity record should serialize");
    write_text_file(path, &payload)
}

fn write_graph_edges(path: PathBuf, edges: &[GraphEdge]) -> Result<(), StorageError> {
    let mut payloads = Vec::with_capacity(edges.len());
    for edge in edges {
        payloads.push(serde_json::to_string(edge).expect("graph edge should serialize"));
    }
    write_jsonl_file(path, &payloads)
}

fn write_external_endpoint(path: PathBuf, endpoint: &ExternalEndpoint) -> Result<(), StorageError> {
    let payload =
        serde_json::to_string_pretty(endpoint).expect("external endpoint should serialize");
    write_text_file(path, &payload)
}

fn write_telemetry_stream(path: PathBuf, stream: &TelemetryStream) -> Result<(), StorageError> {
    let payload = serde_json::to_string_pretty(stream).expect("telemetry stream should serialize");
    write_text_file(path, &payload)
}

fn write_plugin_manifest_file(
    path: PathBuf,
    manifest: &PluginManifest,
) -> Result<(), StorageError> {
    let payload = serde_json::to_string_pretty(manifest).expect("plugin manifest should serialize");
    write_text_file(path, &payload)
}

fn write_plugin_states_file(
    path: PathBuf,
    plugin_states: &BTreeMap<String, bool>,
) -> Result<(), StorageError> {
    let payload =
        serde_json::to_string_pretty(plugin_states).expect("plugin states should serialize");
    write_text_file(path, &payload)
}

fn write_command_envelopes(
    path: PathBuf,
    commands: &[CommandEnvelope],
) -> Result<(), StorageError> {
    let mut payloads = Vec::with_capacity(commands.len());
    for command in commands {
        payloads.push(serde_json::to_string(command).expect("command envelope should serialize"));
    }
    write_jsonl_file(path, &payloads)
}

fn write_event_envelopes(path: PathBuf, events: &[EventEnvelope]) -> Result<(), StorageError> {
    let mut payloads = Vec::with_capacity(events.len());
    for event in events {
        payloads.push(serde_json::to_string(event).expect("event envelope should serialize"));
    }
    write_jsonl_file(path, &payloads)
}

fn read_project_metadata_yaml(path: PathBuf) -> Result<ProjectMetadata, StorageError> {
    let payload = read_text_file(path)?;
    Ok(serde_yaml::from_str(&payload)?)
}

fn read_entity_records_dir(path: PathBuf) -> Result<Vec<EntityRecord>, StorageError> {
    let payloads = read_sorted_payload_files(path)?;
    let mut values = Vec::with_capacity(payloads.len());
    for payload in payloads {
        values.push(serde_json::from_str(&payload)?);
    }
    Ok(values)
}

fn read_graph_edges(path: PathBuf) -> Result<Vec<GraphEdge>, StorageError> {
    let payloads = read_jsonl_payloads(path)?;
    let mut values = Vec::with_capacity(payloads.len());
    for payload in payloads {
        values.push(serde_json::from_str(&payload)?);
    }
    Ok(values)
}

fn read_external_endpoints_dir(path: PathBuf) -> Result<Vec<ExternalEndpoint>, StorageError> {
    let payloads = read_sorted_payload_files(path)?;
    let mut values = Vec::with_capacity(payloads.len());
    for payload in payloads {
        values.push(serde_json::from_str(&payload)?);
    }
    Ok(values)
}

fn read_telemetry_streams_dir(path: PathBuf) -> Result<Vec<TelemetryStream>, StorageError> {
    let payloads = read_sorted_payload_files(path)?;
    let mut values = Vec::with_capacity(payloads.len());
    for payload in payloads {
        values.push(serde_json::from_str(&payload)?);
    }
    Ok(values)
}

fn read_plugin_manifests_dir(path: PathBuf) -> Result<Vec<PluginManifest>, StorageError> {
    let payloads = read_sorted_payload_files(path)?;
    let mut values = Vec::with_capacity(payloads.len());
    for payload in payloads {
        values.push(serde_json::from_str(&payload)?);
    }
    Ok(values)
}

fn read_plugin_states_file(path: PathBuf) -> Result<BTreeMap<String, bool>, StorageError> {
    let payload = read_text_file(path)?;
    Ok(serde_json::from_str(&payload)?)
}

fn read_command_envelopes(path: PathBuf) -> Result<Vec<CommandEnvelope>, StorageError> {
    let payloads = read_jsonl_payloads(path)?;
    let mut values = Vec::with_capacity(payloads.len());
    for payload in payloads {
        values.push(serde_json::from_str(&payload)?);
    }
    Ok(values)
}

fn read_event_envelopes(path: PathBuf) -> Result<Vec<EventEnvelope>, StorageError> {
    let payloads = read_jsonl_payloads(path)?;
    let mut values = Vec::with_capacity(payloads.len());
    for payload in payloads {
        values.push(serde_json::from_str(&payload)?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(windows)]
    use std::os::windows::fs::OpenOptionsExt;
    #[cfg(unix)]
    use std::path::PathBuf;

    use faero_types::{
        Addressing, ConnectionMode, DisplayUnits, EndpointType, LinkMetrics, PluginManifest,
        ProjectDocument, ProjectMetadata, QosProfile, StreamDirection, TelemetryStream,
        TimingProfile, TransportProfile,
    };
    use tempfile::tempdir;

    use super::*;

    #[cfg(unix)]
    struct UnreadableFileGuard {
        path: PathBuf,
        original_mode: u32,
    }

    #[cfg(unix)]
    impl Drop for UnreadableFileGuard {
        fn drop(&mut self) {
            let mut permissions = fs::metadata(&self.path)
                .expect("guarded file should still exist")
                .permissions();
            permissions.set_mode(self.original_mode);
            fs::set_permissions(&self.path, permissions)
                .expect("original permissions should be restorable");
        }
    }

    #[cfg(windows)]
    struct UnreadableFileGuard {
        _file: fs::File,
    }

    fn make_file_unreadable(path: &Path) -> UnreadableFileGuard {
        #[cfg(unix)]
        {
            let original_mode = fs::metadata(path)
                .expect("fixture should exist")
                .permissions()
                .mode();
            let mut permissions = fs::metadata(path)
                .expect("fixture should exist")
                .permissions();
            permissions.set_mode(0o000);
            fs::set_permissions(path, permissions).expect("permissions should update");
            UnreadableFileGuard {
                path: path.to_path_buf(),
                original_mode,
            }
        }

        #[cfg(windows)]
        {
            let file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(path)
                .expect("fixture should open without sharing");
            UnreadableFileGuard { _file: file }
        }
    }

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
        document.commands.push(CommandEnvelope {
            command_id: "cmd_001".to_string(),
            kind: "project.save".to_string(),
            project_id: "prj_9999".to_string(),
            target_id: Some("ent_cell_001".to_string()),
            actor_id: "user".to_string(),
            timestamp: "2026-04-06T00:00:00Z".to_string(),
            base_revision: Some("rev_0001".to_string()),
            payload: serde_json::json!({ "mode": "full" }),
        });
        document.events.push(EventEnvelope {
            event_id: "evt_001".to_string(),
            kind: "project.saved".to_string(),
            project_id: "prj_9999".to_string(),
            target_id: Some("ent_cell_001".to_string()),
            caused_by_command_id: "cmd_001".to_string(),
            timestamp: "2026-04-06T00:00:01Z".to_string(),
            revision: "rev_0002".to_string(),
            payload: serde_json::json!({ "artifactCount": 8 }),
        });
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
        assert_eq!(loaded.commands.len(), 1);
        assert_eq!(loaded.events.len(), 1);
        assert_eq!(
            loaded.plugin_states.get("plg.integration.viewer"),
            Some(&true)
        );
    }

    #[test]
    fn load_project_defaults_missing_optional_artifacts_to_empty() {
        let dir = tempdir().expect("tempdir should be available");
        let project_root = dir.path().join("minimal.faero");
        fs::create_dir_all(&project_root).expect("project root should exist");
        fs::write(
            project_root.join("project.yaml"),
            serde_yaml::to_string(&ProjectMetadata::scaffold("Minimal"))
                .expect("metadata should serialize"),
        )
        .expect("metadata file should be written");

        let loaded = load_project(&project_root).expect("minimal project should load");

        assert_eq!(loaded.metadata.name, "Minimal");
        assert!(loaded.nodes.is_empty());
        assert!(loaded.edges.is_empty());
        assert!(loaded.endpoints.is_empty());
        assert!(loaded.streams.is_empty());
        assert!(loaded.plugin_manifests.is_empty());
        assert!(loaded.plugin_states.is_empty());
        assert!(loaded.commands.is_empty());
        assert!(loaded.events.is_empty());
    }

    #[test]
    fn read_jsonl_skips_blank_lines_and_returns_empty_for_missing_file() {
        let dir = tempdir().expect("tempdir should be available");
        let missing = dir.path().join("missing.jsonl");
        let values = read_graph_edges(missing).expect("missing file should return empty");
        assert!(values.is_empty());

        let jsonl = dir.path().join("edges.jsonl");
        fs::write(
            &jsonl,
            "{\"edgeId\":\"edg_1\",\"from\":\"a\",\"to\":\"b\",\"type\":\"contains\",\"createdAt\":\"2026-04-06T00:00:00Z\"}\n\n",
        )
        .expect("jsonl file should be written");

        let values = read_graph_edges(jsonl).expect("jsonl should load");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].edge_id, "edg_1");
    }

    #[test]
    fn write_jsonl_file_supports_empty_payload_sets() {
        let dir = tempdir().expect("tempdir should be available");
        let path = dir.path().join("events/empty.jsonl");

        write_jsonl_file(path.clone(), &[]).expect("empty jsonl payloads should still write");
        assert_eq!(
            fs::read_to_string(path).expect("empty jsonl file should be readable"),
            ""
        );
    }

    #[test]
    fn read_json_dir_ignores_missing_directory_and_non_file_entries() {
        let dir = tempdir().expect("tempdir should be available");
        let missing = dir.path().join("missing");
        let missing_values =
            read_entity_records_dir(missing).expect("missing directory should return empty");
        assert!(missing_values.is_empty());

        let nodes_dir = dir.path().join("nodes");
        fs::create_dir_all(nodes_dir.join("nested")).expect("nested directory should exist");
        fs::write(
            nodes_dir.join("b.json"),
            serde_json::to_string(&EntityRecord {
                id: "ent_b".to_string(),
                entity_type: "Part".to_string(),
                name: "B".to_string(),
                revision: "rev_0002".to_string(),
                status: "active".to_string(),
                data: serde_json::json!({}),
            })
            .expect("entity should serialize"),
        )
        .expect("entity file should be written");
        fs::write(
            nodes_dir.join("a.json"),
            serde_json::to_string(&EntityRecord {
                id: "ent_a".to_string(),
                entity_type: "Part".to_string(),
                name: "A".to_string(),
                revision: "rev_0001".to_string(),
                status: "active".to_string(),
                data: serde_json::json!({}),
            })
            .expect("entity should serialize"),
        )
        .expect("entity file should be written");

        let values = read_entity_records_dir(nodes_dir).expect("directory should load");
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].id, "ent_a");
        assert_eq!(values[1].id, "ent_b");
    }

    #[test]
    fn invalid_json_and_yaml_surfaces_structured_errors() {
        let dir = tempdir().expect("tempdir should be available");
        let invalid_json = dir.path().join("invalid.json");
        fs::write(&invalid_json, "{not valid json").expect("json fixture should write");
        let _json_error =
            read_plugin_states_file(invalid_json).expect_err("invalid json should fail");

        let invalid_yaml = dir.path().join("invalid.yaml");
        fs::write(&invalid_yaml, "name: [broken").expect("yaml fixture should write");
        let _yaml_error =
            read_project_metadata_yaml(invalid_yaml).expect_err("invalid yaml should fail");
    }

    #[test]
    fn helper_readers_and_writers_cover_all_supported_artifact_types() {
        let dir = tempdir().expect("tempdir should be available");

        let metadata_path = dir.path().join("project.yaml");
        let metadata = ProjectMetadata::scaffold("Helpers");
        write_project_metadata_yaml(metadata_path.clone(), &metadata).expect("yaml should write");
        let loaded_metadata =
            read_project_metadata_yaml(metadata_path).expect("yaml should round trip");
        assert_eq!(loaded_metadata.name, "Helpers");

        let endpoint_dir = dir.path().join("integration/endpoints");
        fs::create_dir_all(&endpoint_dir).expect("endpoint dir should exist");
        write_external_endpoint(
            endpoint_dir.join("endpoint.json"),
            &ExternalEndpoint {
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
                    latency_ms: Some(8),
                    jitter_ms: Some(1),
                    drop_rate: Some(0.0),
                    rssi_dbm: Some(-42),
                    bandwidth_kbps: Some(9000),
                },
                status: "connected".to_string(),
            },
        )
        .expect("endpoint should write");
        let endpoints = read_external_endpoints_dir(endpoint_dir).expect("endpoints should load");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].id, "ext_wifi_001");

        let stream_dir = dir.path().join("integration/streams");
        fs::create_dir_all(&stream_dir).expect("stream dir should exist");
        write_telemetry_stream(
            stream_dir.join("stream.json"),
            &TelemetryStream {
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
        )
        .expect("stream should write");
        let streams = read_telemetry_streams_dir(stream_dir).expect("streams should load");
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].id, "str_001");

        let manifest_dir = dir.path().join("plugins/manifests");
        fs::create_dir_all(&manifest_dir).expect("manifest dir should exist");
        write_plugin_manifest_file(
            manifest_dir.join("plugin.json"),
            &PluginManifest {
                id: "ent_plugin_001".to_string(),
                plugin_id: "plg.integration.viewer".to_string(),
                version: "0.1.0".to_string(),
                capabilities: vec!["panel".to_string()],
                permissions: vec!["project.read".to_string()],
                entrypoints: vec!["plugins/integration-viewer/index.js".to_string()],
                compatibility: vec!["faero-core@0.1".to_string()],
                status: "installed".to_string(),
            },
        )
        .expect("manifest should write");
        let manifests = read_plugin_manifests_dir(manifest_dir).expect("manifests should load");
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].plugin_id, "plg.integration.viewer");

        let plugin_states_path = dir.path().join("plugins/state/plugins.json");
        let plugin_states = BTreeMap::from([(String::from("plg.integration.viewer"), true)]);
        write_plugin_states_file(plugin_states_path.clone(), &plugin_states)
            .expect("states should write");
        let loaded_states =
            read_plugin_states_file(plugin_states_path).expect("states should load");
        assert_eq!(loaded_states.get("plg.integration.viewer"), Some(&true));

        let command_path = dir.path().join("events/commands.jsonl");
        let commands = vec![CommandEnvelope {
            command_id: "cmd_001".to_string(),
            kind: "project.save".to_string(),
            project_id: "prj_0001".to_string(),
            target_id: None,
            actor_id: "user".to_string(),
            timestamp: "2026-04-06T00:00:00Z".to_string(),
            base_revision: None,
            payload: serde_json::json!({}),
        }];
        write_command_envelopes(command_path.clone(), &commands).expect("commands should write");
        let loaded_commands = read_command_envelopes(command_path).expect("commands should load");
        assert_eq!(loaded_commands.len(), 1);
        assert_eq!(loaded_commands[0].command_id, "cmd_001");

        let event_path = dir.path().join("events/events.jsonl");
        let events = vec![EventEnvelope {
            event_id: "evt_001".to_string(),
            kind: "project.saved".to_string(),
            project_id: "prj_0001".to_string(),
            target_id: Some("ent_cell_001".to_string()),
            caused_by_command_id: "cmd_001".to_string(),
            timestamp: "2026-04-06T00:00:01Z".to_string(),
            revision: "rev_0002".to_string(),
            payload: serde_json::json!({ "saved": true }),
        }];
        write_event_envelopes(event_path.clone(), &events).expect("events should write");
        let loaded_events = read_event_envelopes(event_path).expect("events should load");
        assert_eq!(loaded_events.len(), 1);
        assert_eq!(loaded_events[0].event_id, "evt_001");
    }

    #[test]
    fn ensure_parent_dir_accepts_paths_without_parent_components() {
        ensure_parent_dir(Path::new("artifact.json")).expect("relative file should be accepted");
        ensure_parent_dir(Path::new("")).expect("empty path should be accepted");
    }

    #[test]
    fn low_level_helpers_surface_io_errors() {
        let dir = tempdir().expect("tempdir should be available");
        let blocked_parent = dir.path().join("blocked-parent");
        fs::write(&blocked_parent, "occupied").expect("blocking parent should write");

        let ensure_parent_error = ensure_parent_dir(&blocked_parent.join("child.json"))
            .expect_err("file parent should fail directory creation");
        let _ = ensure_parent_error;

        let write_text_error = write_text_file(dir.path().to_path_buf(), "payload")
            .expect_err("writing into a directory should fail");
        let _ = write_text_error;

        let write_text_parent_error = write_text_file(blocked_parent.join("child.txt"), "payload")
            .expect_err("writing under a file parent should fail");
        let _ = write_text_parent_error;

        let write_jsonl_error = write_jsonl_file(dir.path().to_path_buf(), &[String::from("{}")])
            .expect_err("creating a jsonl file over a directory should fail");
        let _ = write_jsonl_error;

        let write_jsonl_parent_error =
            write_jsonl_file(blocked_parent.join("child.jsonl"), &[String::from("{}")])
                .expect_err("writing jsonl under a file parent should fail");
        let _ = write_jsonl_parent_error;

        let read_text_error = read_text_file(dir.path().join("missing.txt"))
            .expect_err("missing text file should fail");
        let _ = read_text_error;

        let read_jsonl_error = read_jsonl_payloads(dir.path().to_path_buf())
            .expect_err("opening a directory as a jsonl file should fail");
        let _ = read_jsonl_error;

        let read_dir_error = read_sorted_payload_files(dir.path().join("single-file"))
            .expect("missing directory should return empty");
        assert!(read_dir_error.is_empty());

        fs::write(dir.path().join("single-file"), "payload").expect("fixture file should write");
        let read_dir_error = read_sorted_payload_files(dir.path().join("single-file"))
            .expect_err("reading a file as a directory should fail");
        let _ = read_dir_error;
    }

    #[test]
    fn read_sorted_payload_files_surfaces_unreadable_file_errors() {
        let dir = tempdir().expect("tempdir should be available");
        let unreadable = dir.path().join("locked.json");
        fs::write(&unreadable, "{\"id\":\"blocked\"}").expect("fixture file should write");

        let guard = make_file_unreadable(&unreadable);
        let error = read_sorted_payload_files(dir.path().to_path_buf())
            .expect_err("unreadable file should bubble an io error");
        let _ = error;
        drop(guard);
    }

    #[test]
    fn load_helpers_surface_json_errors_for_every_artifact_family() {
        let dir = tempdir().expect("tempdir should be available");

        let cases = [
            ("nodes", dir.path().join("nodes"), "broken.json"),
            ("endpoints", dir.path().join("endpoints"), "broken.json"),
            ("streams", dir.path().join("streams"), "broken.json"),
            ("manifests", dir.path().join("manifests"), "broken.json"),
        ];

        for (_, root, file_name) in cases {
            fs::create_dir_all(&root).expect("artifact directory should exist");
            fs::write(root.join(file_name), "{broken").expect("broken artifact should write");
        }

        fs::write(dir.path().join("edges.jsonl"), "{broken\n").expect("broken edges should write");
        fs::write(dir.path().join("commands.jsonl"), "{broken\n")
            .expect("broken commands should write");
        fs::write(dir.path().join("events.jsonl"), "{broken\n")
            .expect("broken events should write");

        let _ = read_entity_records_dir(dir.path().join("nodes")).expect_err("nodes should fail");
        let _ = read_graph_edges(dir.path().join("edges.jsonl")).expect_err("edges should fail");
        let _ = read_external_endpoints_dir(dir.path().join("endpoints"))
            .expect_err("endpoints should fail");
        let _ = read_telemetry_streams_dir(dir.path().join("streams"))
            .expect_err("streams should fail");
        let _ = read_plugin_manifests_dir(dir.path().join("manifests"))
            .expect_err("manifests should fail");
        let _ = read_command_envelopes(dir.path().join("commands.jsonl"))
            .expect_err("commands should fail");
        let _ =
            read_event_envelopes(dir.path().join("events.jsonl")).expect_err("events should fail");
    }

    #[test]
    fn wrapper_helpers_surface_io_errors_from_underlying_sources() {
        let dir = tempdir().expect("tempdir should be available");
        let file_path = dir.path().join("artifact.file");
        fs::write(&file_path, "occupied").expect("fixture file should write");

        let _ = read_project_metadata_yaml(dir.path().to_path_buf())
            .expect_err("directories should fail as metadata files");
        let _ = read_entity_records_dir(file_path.clone())
            .expect_err("files should fail as entity directories");
        let _ = read_external_endpoints_dir(file_path.clone())
            .expect_err("files should fail as endpoint directories");
        let _ = read_telemetry_streams_dir(file_path.clone())
            .expect_err("files should fail as stream directories");
        let _ = read_plugin_manifests_dir(file_path)
            .expect_err("files should fail as manifest directories");
        let _ = read_graph_edges(dir.path().to_path_buf())
            .expect_err("directories should fail as edge jsonl files");
        let _ = read_command_envelopes(dir.path().to_path_buf())
            .expect_err("directories should fail as command jsonl files");
        let _ = read_event_envelopes(dir.path().to_path_buf())
            .expect_err("directories should fail as event jsonl files");
    }

    #[test]
    fn load_project_surfaces_errors_for_each_serialized_artifact_group() {
        let cases = [
            ("project.yaml", "project.yaml", "name: [broken"),
            ("graph/nodes/ent_cell_001.json", "project.yaml", ""),
            ("graph/edges.jsonl", "project.yaml", ""),
            (
                "integration/endpoints/ext_wifi_001.json",
                "project.yaml",
                "",
            ),
            ("integration/streams/str_001.json", "project.yaml", ""),
            (
                "plugins/manifests/plg.integration.viewer.json",
                "project.yaml",
                "",
            ),
            ("events/commands.jsonl", "project.yaml", ""),
            ("events/events.jsonl", "project.yaml", ""),
        ];

        for (artifact, metadata_file, invalid_yaml) in cases {
            let dir = tempdir().expect("tempdir should be available");
            let root = dir.path().join("fixture.faero");
            let document = sample_document();

            save_project(&root, &document).expect("baseline project should save");
            if metadata_file == "project.yaml" && !invalid_yaml.is_empty() {
                fs::write(root.join("project.yaml"), invalid_yaml)
                    .expect("invalid yaml should write");
            } else {
                fs::write(root.join(artifact), "{broken\n").expect("invalid artifact should write");
            }

            let _error = load_project(&root).expect_err("invalid artifact should fail");
        }
    }

    #[test]
    fn save_project_surfaces_io_errors_for_each_output_artifact_group() {
        let cases = [
            "project.yaml",
            "graph/nodes/ent_cell_001.json",
            "graph/edges.jsonl",
            "integration/endpoints/ext_wifi_001.json",
            "integration/streams/str_001.json",
            "plugins/manifests/plg.integration.viewer.json",
            "plugins/state/plugins.json",
            "events/commands.jsonl",
            "events/events.jsonl",
        ];

        for artifact in cases {
            let dir = tempdir().expect("tempdir should be available");
            let root = dir.path().join("fixture.faero");
            let document = sample_document();

            fs::create_dir_all(root.join(Path::new(artifact)))
                .expect("conflicting artifact directory should exist");

            let _error =
                save_project(&root, &document).expect_err("conflicting artifact should fail");
        }

        let dir = tempdir().expect("tempdir should be available");
        let root_file = dir.path().join("fixture.faero");
        fs::write(&root_file, "occupied").expect("root file should exist");
        let _error =
            save_project(&root_file, &sample_document()).expect_err("file root should fail");

        let layout_conflicts = [
            "integration/endpoints",
            "integration/streams",
            "plugins/manifests",
            "plugins/state",
            "events",
        ];

        for blocked_path in layout_conflicts {
            let dir = tempdir().expect("tempdir should be available");
            let root = dir.path().join("fixture.faero");
            let blocked = root.join(blocked_path);
            let parent = blocked.parent().expect("blocked path should have a parent");
            fs::create_dir_all(parent).expect("blocked parent should exist");
            fs::write(&blocked, "occupied").expect("blocking file should write");

            let _error = save_project(&root, &sample_document())
                .expect_err("conflicting layout should fail");
        }
    }
}
