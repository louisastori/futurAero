use std::collections::BTreeMap;

use faero_types::{
    CommandEnvelope, EntityRecord, EventEnvelope, ExternalEndpoint, PluginManifest,
    ProjectDocument, TelemetryStream,
};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("entity `{0}` already exists")]
    EntityAlreadyExists(String),
    #[error("entity `{0}` does not exist")]
    EntityNotFound(String),
    #[error("endpoint `{0}` already exists")]
    EndpointAlreadyExists(String),
    #[error("stream `{0}` already exists")]
    StreamAlreadyExists(String),
    #[error("stream references unknown endpoint `{0}`")]
    UnknownEndpoint(String),
    #[error("plugin `{0}` already installed")]
    PluginAlreadyInstalled(String),
    #[error("plugin `{0}` is not installed")]
    PluginNotInstalled(String),
}

#[derive(Debug, Clone)]
pub enum CoreCommand {
    CreateEntity(EntityRecord),
    ReplaceEntity(EntityRecord),
    RegisterEndpoint(ExternalEndpoint),
    RegisterStream(TelemetryStream),
    InstallPlugin(PluginManifest),
    SetPluginEnabled { plugin_id: String, enabled: bool },
}

#[derive(Debug, Clone)]
pub struct ProjectGraph {
    document: ProjectDocument,
    revision_counter: usize,
    command_counter: usize,
    event_counter: usize,
}

impl ProjectGraph {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            document: ProjectDocument::empty(name.into()),
            revision_counter: 1,
            command_counter: 1,
            event_counter: 1,
        }
    }

    pub fn from_document(document: ProjectDocument) -> Self {
        Self {
            revision_counter: next_counter(
                document
                    .nodes
                    .values()
                    .map(|entity| entity.revision.as_str())
                    .chain(document.events.iter().map(|event| event.revision.as_str())),
                "rev_",
            ),
            command_counter: next_counter(
                document
                    .commands
                    .iter()
                    .map(|command| command.command_id.as_str()),
                "cmd_",
            ),
            event_counter: next_counter(
                document.events.iter().map(|event| event.event_id.as_str()),
                "evt_",
            ),
            document,
        }
    }

    pub fn document(&self) -> &ProjectDocument {
        &self.document
    }

    pub fn into_document(self) -> ProjectDocument {
        self.document
    }

    pub fn project_name(&self) -> &str {
        &self.document.metadata.name
    }

    pub fn entity_count(&self) -> usize {
        self.document.nodes.len()
    }

    pub fn endpoint_count(&self) -> usize {
        self.document.endpoints.len()
    }

    pub fn stream_count(&self) -> usize {
        self.document.streams.len()
    }

    pub fn plugin_count(&self) -> usize {
        self.document.plugin_manifests.len()
    }

    pub fn plugin_state(&self) -> &BTreeMap<String, bool> {
        &self.document.plugin_states
    }

    pub fn apply_command(&mut self, command: CoreCommand) -> Result<EventEnvelope, CoreError> {
        match command {
            CoreCommand::CreateEntity(entity) => {
                if self.document.nodes.contains_key(&entity.id) {
                    return Err(CoreError::EntityAlreadyExists(entity.id));
                }

                let payload = serialize_payload(&entity);
                let command_id = self.record_command(
                    "entity.create",
                    Some(entity.id.clone()),
                    None,
                    payload.clone(),
                );
                self.document
                    .nodes
                    .insert(entity.id.clone(), entity.clone());
                let revision = self.next_revision();

                self.record_event(
                    "entity.created",
                    Some(entity.id),
                    command_id,
                    revision,
                    payload,
                )
            }
            CoreCommand::ReplaceEntity(entity) => {
                if !self.document.nodes.contains_key(&entity.id) {
                    return Err(CoreError::EntityNotFound(entity.id));
                }

                let base_revision = self
                    .document
                    .nodes
                    .get(&entity.id)
                    .map(|existing| existing.revision.clone());
                let payload = serialize_payload(&entity);
                let command_id = self.record_command(
                    "entity.update",
                    Some(entity.id.clone()),
                    base_revision,
                    payload,
                );
                let revision = self.next_revision();
                let mut next_entity = entity.clone();
                next_entity.revision = revision.clone();
                let event_payload = serialize_payload(&next_entity);
                self.document
                    .nodes
                    .insert(next_entity.id.clone(), next_entity.clone());

                self.record_event(
                    "entity.updated",
                    Some(next_entity.id),
                    command_id,
                    revision,
                    event_payload,
                )
            }
            CoreCommand::RegisterEndpoint(endpoint) => {
                if self.document.endpoints.contains_key(&endpoint.id) {
                    return Err(CoreError::EndpointAlreadyExists(endpoint.id));
                }

                let payload = serialize_payload(&endpoint);
                let command_id = self.record_command(
                    "wireless.endpoint.create",
                    Some(endpoint.id.clone()),
                    None,
                    payload.clone(),
                );
                self.document
                    .endpoints
                    .insert(endpoint.id.clone(), endpoint.clone());
                let revision = self.next_revision();

                self.record_event(
                    "integration.connected",
                    Some(endpoint.id),
                    command_id,
                    revision,
                    payload,
                )
            }
            CoreCommand::RegisterStream(stream) => {
                if self.document.streams.contains_key(&stream.id) {
                    return Err(CoreError::StreamAlreadyExists(stream.id));
                }
                if !self.document.endpoints.contains_key(&stream.endpoint_id) {
                    return Err(CoreError::UnknownEndpoint(stream.endpoint_id));
                }

                let payload = serialize_payload(&stream);
                let command_id = self.record_command(
                    "telemetry.stream.create",
                    Some(stream.id.clone()),
                    None,
                    payload.clone(),
                );
                self.document
                    .streams
                    .insert(stream.id.clone(), stream.clone());
                let revision = self.next_revision();

                self.record_event(
                    "entity.created",
                    Some(stream.id),
                    command_id,
                    revision,
                    payload,
                )
            }
            CoreCommand::InstallPlugin(manifest) => {
                if self
                    .document
                    .plugin_manifests
                    .contains_key(&manifest.plugin_id)
                {
                    return Err(CoreError::PluginAlreadyInstalled(manifest.plugin_id));
                }

                let payload = serialize_payload(&manifest);
                let command_id = self.record_command(
                    "plugin.install",
                    Some(manifest.plugin_id.clone()),
                    None,
                    payload.clone(),
                );
                self.document
                    .plugin_manifests
                    .insert(manifest.plugin_id.clone(), manifest.clone());
                self.document
                    .plugin_states
                    .insert(manifest.plugin_id.clone(), false);
                let revision = self.next_revision();

                self.record_event(
                    "plugin.installed",
                    Some(manifest.plugin_id),
                    command_id,
                    revision,
                    payload,
                )
            }
            CoreCommand::SetPluginEnabled { plugin_id, enabled } => {
                if !self.document.plugin_manifests.contains_key(&plugin_id) {
                    return Err(CoreError::PluginNotInstalled(plugin_id));
                }

                let payload = serde_json::json!({
                    "pluginId": plugin_id,
                    "enabled": enabled
                });
                let command_id = self.record_command(
                    "plugin.state.set",
                    Some(plugin_id.clone()),
                    None,
                    payload.clone(),
                );
                self.document
                    .plugin_states
                    .insert(plugin_id.clone(), enabled);
                let revision = self.next_revision();

                self.record_event(
                    if enabled {
                        "plugin.enabled"
                    } else {
                        "plugin.disabled"
                    },
                    Some(plugin_id),
                    command_id,
                    revision,
                    payload,
                )
            }
        }
    }

    fn record_command(
        &mut self,
        kind: &str,
        target_id: Option<String>,
        base_revision: Option<String>,
        payload: Value,
    ) -> String {
        let command_id = format!("cmd_{:04}", self.command_counter);
        self.command_counter += 1;

        self.document.commands.push(CommandEnvelope {
            command_id: command_id.clone(),
            kind: kind.to_string(),
            project_id: self.document.metadata.project_id.clone(),
            target_id,
            actor_id: "system.scaffold".to_string(),
            timestamp: "2026-04-06T00:00:00Z".to_string(),
            base_revision,
            payload,
        });

        command_id
    }

    fn record_event(
        &mut self,
        kind: &str,
        target_id: Option<String>,
        caused_by_command_id: String,
        revision: String,
        payload: Value,
    ) -> Result<EventEnvelope, CoreError> {
        let event = EventEnvelope {
            event_id: format!("evt_{:04}", self.event_counter),
            kind: kind.to_string(),
            project_id: self.document.metadata.project_id.clone(),
            target_id,
            caused_by_command_id,
            timestamp: "2026-04-06T00:00:01Z".to_string(),
            revision,
            payload,
        };
        self.event_counter += 1;
        self.document.events.push(event.clone());
        Ok(event)
    }

    fn next_revision(&mut self) -> String {
        let revision = format!("rev_{:04}", self.revision_counter);
        self.revision_counter += 1;
        self.document.metadata.updated_at = "2026-04-06T00:00:01Z".to_string();
        revision
    }
}

fn next_counter<'a>(ids: impl Iterator<Item = &'a str>, prefix: &str) -> usize {
    ids.filter_map(|id| {
        id.strip_prefix(prefix)
            .and_then(|raw| raw.parse::<usize>().ok())
    })
    .max()
    .map(|value| value + 1)
    .unwrap_or(1)
}

fn serialize_payload<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("core payload should always serialize")
}

#[cfg(test)]
mod tests {
    use faero_types::{
        Addressing, ConnectionMode, EndpointType, LinkMetrics, QosProfile, StreamDirection,
        TimingProfile, TransportProfile,
    };

    use super::*;

    fn sample_entity() -> EntityRecord {
        EntityRecord {
            id: "ent_part_001".to_string(),
            entity_type: "Part".to_string(),
            name: "Bracket-A".to_string(),
            revision: "rev_seed".to_string(),
            status: "active".to_string(),
            data: serde_json::json!({
                "geometrySource": "parametric",
                "parameterSet": { "width": 120 }
            }),
        }
    }

    fn sample_endpoint() -> ExternalEndpoint {
        ExternalEndpoint {
            id: "ext_wifi_001".to_string(),
            name: "AMR-Lidar-WiFi-01".to_string(),
            endpoint_type: EndpointType::WifiDevice,
            transport_profile: TransportProfile {
                transport_kind: "wifi".to_string(),
                adapter_id: Some("wlan0".to_string()),
                discovery_mode: Some("mdns".to_string()),
                credential_policy: Some("runtime_prompt".to_string()),
                security_mode: Some("wpa3".to_string()),
            },
            connection_profile: serde_json::json!({ "retryBackoffMs": 250 }),
            addressing: Addressing {
                host: Some("amr-lidar-01.local".to_string()),
                port: Some(9001),
                path: Some("/telemetry".to_string()),
                device_id: None,
            },
            signal_map_ids: vec!["sig_lidar_001".to_string()],
            mode: ConnectionMode::Live,
            link_metrics: LinkMetrics {
                latency_ms: Some(14),
                jitter_ms: Some(2),
                drop_rate: Some(0.0),
                rssi_dbm: Some(-51),
                bandwidth_kbps: Some(12000),
            },
            status: "connected".to_string(),
        }
    }

    fn sample_stream(endpoint_id: &str) -> TelemetryStream {
        TelemetryStream {
            id: "str_bumper_001".to_string(),
            name: "BumperStatus".to_string(),
            endpoint_id: endpoint_id.to_string(),
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
        }
    }

    fn sample_plugin() -> PluginManifest {
        PluginManifest {
            id: "ent_plugin_001".to_string(),
            plugin_id: "plg.integration.viewer".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec!["panel".to_string()],
            permissions: vec![
                "project.read".to_string(),
                "integration.observe".to_string(),
            ],
            entrypoints: vec!["plugins/integration-viewer/index.js".to_string()],
            compatibility: vec!["faero-core@0.1".to_string()],
            status: "installed".to_string(),
        }
    }

    #[test]
    fn creates_entity_and_records_command_event() {
        let mut graph = ProjectGraph::new("Demo");
        let event = graph
            .apply_command(CoreCommand::CreateEntity(sample_entity()))
            .expect("entity should be created");

        assert_eq!(graph.entity_count(), 1);
        assert_eq!(event.kind, "entity.created");
        assert_eq!(graph.document().commands.len(), 1);
        assert_eq!(graph.document().events.len(), 1);
    }

    #[test]
    fn replaces_existing_entity_and_advances_revision() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity()))
            .expect("entity should be created");

        let mut updated = sample_entity();
        updated.name = "Bracket-B".to_string();
        updated.data = serde_json::json!({
            "geometrySource": "parametric",
            "parameterSet": { "width": 140 }
        });

        let event = graph
            .apply_command(CoreCommand::ReplaceEntity(updated))
            .expect("entity should update");

        assert_eq!(event.kind, "entity.updated");
        assert_eq!(graph.document().nodes["ent_part_001"].name, "Bracket-B");
        assert_eq!(graph.document().nodes["ent_part_001"].revision, "rev_0002");
        assert_eq!(graph.document().commands.len(), 2);
    }

    #[test]
    fn stream_requires_existing_endpoint() {
        let mut graph = ProjectGraph::new("Demo");

        let error = graph
            .apply_command(CoreCommand::RegisterStream(sample_stream("missing")))
            .expect_err("stream without endpoint should fail");

        assert_eq!(error, CoreError::UnknownEndpoint("missing".to_string()));
    }

    #[test]
    fn registers_endpoint_stream_and_plugin() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::RegisterEndpoint(sample_endpoint()))
            .expect("endpoint should register");
        graph
            .apply_command(CoreCommand::RegisterStream(sample_stream("ext_wifi_001")))
            .expect("stream should register");
        graph
            .apply_command(CoreCommand::InstallPlugin(sample_plugin()))
            .expect("plugin should install");

        assert_eq!(graph.endpoint_count(), 1);
        assert_eq!(graph.document().streams.len(), 1);
        assert_eq!(
            graph.plugin_state().get("plg.integration.viewer"),
            Some(&false)
        );
    }

    #[test]
    fn reconstructs_graph_from_document_and_keeps_counters_moving_forward() {
        let mut seed = ProjectGraph::new("Demo");
        seed.apply_command(CoreCommand::CreateEntity(sample_entity()))
            .expect("entity should seed");
        seed.apply_command(CoreCommand::InstallPlugin(sample_plugin()))
            .expect("plugin should seed");

        let mut graph = ProjectGraph::from_document(seed.into_document());
        graph
            .apply_command(CoreCommand::SetPluginEnabled {
                plugin_id: "plg.integration.viewer".to_string(),
                enabled: true,
            })
            .expect("plugin should enable");

        assert_eq!(
            graph.plugin_state().get("plg.integration.viewer"),
            Some(&true)
        );
        assert_eq!(
            graph
                .document()
                .commands
                .last()
                .map(|command| command.command_id.as_str()),
            Some("cmd_0003")
        );
    }

    #[test]
    fn accessors_reflect_current_document_state() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::RegisterEndpoint(sample_endpoint()))
            .expect("endpoint should register");
        graph
            .apply_command(CoreCommand::RegisterStream(sample_stream("ext_wifi_001")))
            .expect("stream should register");
        graph
            .apply_command(CoreCommand::InstallPlugin(sample_plugin()))
            .expect("plugin should install");

        assert_eq!(graph.project_name(), "Demo");
        assert_eq!(graph.stream_count(), 1);
        assert_eq!(graph.plugin_count(), 1);
    }

    #[test]
    fn rejects_duplicate_entity_endpoint_stream_and_plugin() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity()))
            .expect("entity should register");
        let duplicate_entity = graph
            .apply_command(CoreCommand::CreateEntity(sample_entity()))
            .expect_err("duplicate entity should fail");
        assert_eq!(
            duplicate_entity,
            CoreError::EntityAlreadyExists("ent_part_001".to_string())
        );

        graph
            .apply_command(CoreCommand::RegisterEndpoint(sample_endpoint()))
            .expect("endpoint should register");
        let duplicate_endpoint = graph
            .apply_command(CoreCommand::RegisterEndpoint(sample_endpoint()))
            .expect_err("duplicate endpoint should fail");
        assert_eq!(
            duplicate_endpoint,
            CoreError::EndpointAlreadyExists("ext_wifi_001".to_string())
        );

        graph
            .apply_command(CoreCommand::RegisterStream(sample_stream("ext_wifi_001")))
            .expect("stream should register");
        let duplicate_stream = graph
            .apply_command(CoreCommand::RegisterStream(sample_stream("ext_wifi_001")))
            .expect_err("duplicate stream should fail");
        assert_eq!(
            duplicate_stream,
            CoreError::StreamAlreadyExists("str_bumper_001".to_string())
        );

        graph
            .apply_command(CoreCommand::InstallPlugin(sample_plugin()))
            .expect("plugin should install");
        let duplicate_plugin = graph
            .apply_command(CoreCommand::InstallPlugin(sample_plugin()))
            .expect_err("duplicate plugin should fail");
        assert_eq!(
            duplicate_plugin,
            CoreError::PluginAlreadyInstalled("plg.integration.viewer".to_string())
        );

        let missing_entity = graph
            .apply_command(CoreCommand::ReplaceEntity(EntityRecord {
                id: "missing".to_string(),
                ..sample_entity()
            }))
            .expect_err("missing entity update should fail");
        assert_eq!(
            missing_entity,
            CoreError::EntityNotFound("missing".to_string())
        );
    }

    #[test]
    fn plugin_enable_disable_requires_installation_and_records_both_states() {
        let mut graph = ProjectGraph::new("Demo");
        let missing = graph
            .apply_command(CoreCommand::SetPluginEnabled {
                plugin_id: "missing.plugin".to_string(),
                enabled: true,
            })
            .expect_err("unknown plugin should fail");
        assert_eq!(
            missing,
            CoreError::PluginNotInstalled("missing.plugin".to_string())
        );

        graph
            .apply_command(CoreCommand::InstallPlugin(sample_plugin()))
            .expect("plugin should install");
        graph
            .apply_command(CoreCommand::SetPluginEnabled {
                plugin_id: "plg.integration.viewer".to_string(),
                enabled: true,
            })
            .expect("plugin should enable");
        let disabled = graph
            .apply_command(CoreCommand::SetPluginEnabled {
                plugin_id: "plg.integration.viewer".to_string(),
                enabled: false,
            })
            .expect("plugin should disable");

        assert_eq!(disabled.kind, "plugin.disabled");
        assert_eq!(
            graph.plugin_state().get("plg.integration.viewer"),
            Some(&false)
        );
    }
}
