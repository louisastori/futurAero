use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type ProjectId = String;
pub type EntityId = String;
pub type RevisionId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DisplayUnits {
    pub length: String,
    pub angle: String,
    pub mass: String,
}

impl Default for DisplayUnits {
    fn default() -> Self {
        Self {
            length: "mm".to_string(),
            angle: "deg".to_string(),
            mass: "kg".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub project_id: ProjectId,
    pub name: String,
    pub format_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub app_version: String,
    pub display_units: DisplayUnits,
    pub default_frame: String,
    pub root_scene_id: Option<String>,
    pub active_configuration_id: String,
}

impl ProjectMetadata {
    pub fn scaffold(name: impl Into<String>) -> Self {
        Self {
            project_id: "prj_0001".to_string(),
            name: name.into(),
            format_version: "0.1.0".to_string(),
            created_at: "2026-04-06T00:00:00Z".to_string(),
            updated_at: "2026-04-06T00:00:00Z".to_string(),
            app_version: "0.1.0-alpha".to_string(),
            display_units: DisplayUnits::default(),
            default_frame: "world".to_string(),
            root_scene_id: None,
            active_configuration_id: "cfg_default".to_string(),
        }
    }
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self::scaffold("Unnamed Project")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntityRecord {
    pub id: EntityId,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub name: String,
    pub revision: RevisionId,
    pub status: String,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub edge_id: String,
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub relation_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    Live,
    Replay,
    Emulated,
    Gateway,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    Ros2,
    Opcua,
    Plc,
    RobotController,
    BluetoothLe,
    BluetoothClassic,
    WifiDevice,
    MqttBroker,
    WebsocketPeer,
    TcpStream,
    UdpStream,
    SerialDevice,
    FieldbusTrace,
    CustomStream,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransportProfile {
    pub transport_kind: String,
    pub adapter_id: Option<String>,
    pub discovery_mode: Option<String>,
    pub credential_policy: Option<String>,
    pub security_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Addressing {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LinkMetrics {
    pub latency_ms: Option<u32>,
    pub jitter_ms: Option<u32>,
    pub drop_rate: Option<f64>,
    pub rssi_dbm: Option<i32>,
    pub bandwidth_kbps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEndpoint {
    pub id: String,
    pub name: String,
    pub endpoint_type: EndpointType,
    pub transport_profile: TransportProfile,
    pub connection_profile: Value,
    pub addressing: Addressing,
    pub signal_map_ids: Vec<String>,
    pub mode: ConnectionMode,
    pub link_metrics: LinkMetrics,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimingProfile {
    pub expected_rate_hz: u32,
    pub max_latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QosProfile {
    pub delivery: String,
    pub ordering: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryStream {
    pub id: String,
    pub name: String,
    pub endpoint_id: String,
    pub stream_type: String,
    pub direction: StreamDirection,
    pub codec_profile: Value,
    pub schema_ref: String,
    pub timing_profile: TimingProfile,
    pub qos_profile: QosProfile,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkCaptureDataset {
    pub id: String,
    pub endpoint_id: String,
    pub capture_type: String,
    pub timestamp_range: String,
    pub asset_refs: Vec<String>,
    pub link_metrics: LinkMetrics,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope {
    pub command_id: String,
    pub kind: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub actor_id: String,
    pub timestamp: String,
    pub base_revision: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventEnvelope {
    pub event_id: String,
    pub kind: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub caused_by_command_id: String,
    pub timestamp: String,
    pub revision: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JobEnvelope {
    pub job_id: String,
    pub kind: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub status: String,
    pub progress: f32,
    pub phase: String,
    pub message: String,
    pub estimated_remaining_ms: Option<u64>,
    pub started_at: Option<String>,
    pub updated_at: String,
    pub result_ref: Option<String>,
    pub error: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub plugin_id: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub entrypoints: Vec<String>,
    pub compatibility: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenSpecDocument {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub body_format: String,
    pub entity_refs: Vec<String>,
    pub external_refs: Vec<String>,
    pub tags: Vec<String>,
    pub updated_at: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDocument {
    pub metadata: ProjectMetadata,
    pub nodes: BTreeMap<String, EntityRecord>,
    pub edges: Vec<GraphEdge>,
    pub endpoints: BTreeMap<String, ExternalEndpoint>,
    pub streams: BTreeMap<String, TelemetryStream>,
    pub plugin_manifests: BTreeMap<String, PluginManifest>,
    pub plugin_states: BTreeMap<String, bool>,
    pub open_spec_documents: BTreeMap<String, OpenSpecDocument>,
    pub commands: Vec<CommandEnvelope>,
    pub events: Vec<EventEnvelope>,
}

impl ProjectDocument {
    pub fn empty(name: String) -> Self {
        Self {
            metadata: ProjectMetadata::scaffold(name),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_document_scaffold_has_expected_defaults() {
        let document = ProjectDocument::empty("Demo".to_string());

        assert_eq!(document.metadata.name, "Demo");
        assert_eq!(document.metadata.project_id, "prj_0001");
        assert_eq!(document.metadata.display_units.length, "mm");
        assert!(document.nodes.is_empty());
    }

    #[test]
    fn endpoint_serializes_with_camel_case_fields() {
        let endpoint = ExternalEndpoint {
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
                host: Some("edge-box.local".to_string()),
                port: Some(9001),
                path: Some("/telemetry".to_string()),
                device_id: None,
            },
            signal_map_ids: vec!["sig_001".to_string()],
            mode: ConnectionMode::Live,
            link_metrics: LinkMetrics {
                latency_ms: Some(12),
                ..LinkMetrics::default()
            },
            status: "ready".to_string(),
        };

        let json = serde_json::to_value(endpoint).expect("endpoint should serialize");
        assert_eq!(json["transportProfile"]["transportKind"], "wifi");
        assert_eq!(json["mode"], "live");
    }

    #[test]
    fn open_spec_document_serializes_with_camel_case_fields() {
        let document = OpenSpecDocument {
            id: "ops_pick_layout".to_string(),
            title: "Intentions d implantation".to_string(),
            kind: "design_intent".to_string(),
            status: "active".to_string(),
            body_format: "markdown".to_string(),
            entity_refs: vec!["ent_cell_001".to_string()],
            external_refs: vec!["ext_robot_001".to_string()],
            tags: vec!["openspec".to_string(), "mvp".to_string()],
            updated_at: "2026-04-08T08:00:00Z".to_string(),
            content: "## Intent\nCellule lisible en clair.\n".to_string(),
        };

        let json = serde_json::to_value(document).expect("open spec document should serialize");
        assert_eq!(json["bodyFormat"], "markdown");
        assert_eq!(json["entityRefs"][0], "ent_cell_001");
        assert_eq!(json["updatedAt"], "2026-04-08T08:00:00Z");
    }
}
