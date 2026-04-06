use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use faero_ai::{
    AiChatResponse, AiConversationMessage, AiRuntimeStatus, chat_with_project,
    query_runtime_status as query_ai_runtime_status,
};
use faero_core::{CoreCommand, ProjectGraph};
use faero_plugin_host::validate_manifest;
use faero_types::{
    EntityRecord, ExternalEndpoint, PluginManifest, ProjectDocument, QosProfile, StreamDirection,
    TelemetryStream, TimingProfile,
};
use serde::Serialize;
use tauri::State;

const DEFAULT_FIXTURE_ID: &str = "pick-and-place-demo.faero";
const UNTITLED_SESSION_ID: &str = "session:untitled";

type SharedWorkspace = Mutex<WorkspaceSession>;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BackendStatus {
    runtime: String,
    fixture_id: String,
    project_name: String,
    entity_count: usize,
    endpoint_count: usize,
    stream_count: usize,
    plugin_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FixtureProject {
    id: String,
    project_name: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProjectDetails {
    project_id: String,
    format_version: String,
    default_frame: String,
    root_scene_id: Option<String>,
    active_configuration_id: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct EntitySummary {
    id: String,
    entity_type: String,
    name: String,
    revision: String,
    status: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct EndpointSummary {
    id: String,
    name: String,
    endpoint_type: String,
    transport_kind: String,
    mode: String,
    address: String,
    status: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StreamSummary {
    id: String,
    name: String,
    endpoint_id: String,
    stream_type: String,
    direction: String,
    status: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PluginSummary {
    plugin_id: String,
    version: String,
    enabled: bool,
    status: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ActivityEntry {
    id: String,
    channel: String,
    kind: String,
    timestamp: String,
    target_id: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProjectSnapshot {
    status: BackendStatus,
    details: ProjectDetails,
    entities: Vec<EntitySummary>,
    endpoints: Vec<EndpointSummary>,
    streams: Vec<StreamSummary>,
    plugins: Vec<PluginSummary>,
    recent_activity: Vec<ActivityEntry>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CommandResult {
    command_id: String,
    status: String,
    message: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CommandResponse {
    snapshot: ProjectSnapshot,
    result: CommandResult,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkspaceBootstrap {
    fixtures: Vec<FixtureProject>,
    snapshot: ProjectSnapshot,
}

#[derive(Debug)]
struct WorkspaceSession {
    fixture_id: String,
    graph: ProjectGraph,
    system_activity: Vec<ActivityEntry>,
    system_counter: usize,
}

impl WorkspaceSession {
    fn load_fixture(project_id: &str) -> Result<Self, String> {
        let mut session = Self {
            fixture_id: project_id.to_string(),
            graph: ProjectGraph::from_document(load_project_document(project_id)?),
            system_activity: Vec::new(),
            system_counter: 1,
        };
        session.push_system_activity("workspace.loaded", Some(project_id.to_string()));
        Ok(session)
    }

    fn empty(name: &str) -> Self {
        let mut session = Self {
            fixture_id: UNTITLED_SESSION_ID.to_string(),
            graph: ProjectGraph::new(name),
            system_activity: Vec::new(),
            system_counter: 1,
        };
        session.push_system_activity("workspace.created", Some(name.to_string()));
        session
    }

    fn snapshot(&self) -> ProjectSnapshot {
        build_project_snapshot_from_document(
            &self.fixture_id,
            self.graph.document(),
            &self.system_activity,
        )
    }

    fn push_system_activity(&mut self, kind: &str, target_id: Option<String>) {
        let system_id = format!("sys_{:04}", self.system_counter);
        let timestamp = format!(
            "2026-04-06T12:{:02}:{:02}Z",
            (self.system_counter / 60) % 60,
            self.system_counter % 60
        );
        self.system_counter += 1;
        self.system_activity.push(ActivityEntry {
            id: system_id,
            channel: "system".to_string(),
            kind: kind.to_string(),
            timestamp,
            target_id,
        });
    }

    fn execute_command(&mut self, command_id: &str) -> Result<CommandResult, String> {
        match command_id {
            "project.create" => {
                *self = WorkspaceSession::empty("FutureAero Session");
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "nouveau projet de session cree".to_string(),
                })
            }
            "entity.create.part" => {
                let index = self.graph.entity_count() + 1;
                let entity = sample_entity(
                    "Part",
                    &format!("ent_part_{index:03}"),
                    &format!("Part-{index:03}"),
                    serde_json::json!({
                        "geometrySource": "desktop_session",
                        "parameterSet": { "width": 120 + index as i32 }
                    }),
                );
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "piece ajoutee a la session".to_string(),
                })
            }
            "entity.create.assembly" => {
                let index = self.graph.entity_count() + 1;
                let entity = sample_entity(
                    "Assembly",
                    &format!("ent_asm_{index:03}"),
                    &format!("Assembly-{index:03}"),
                    serde_json::json!({
                        "source": "desktop_session",
                        "occurrenceIds": []
                    }),
                );
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "assemblage ajoute a la session".to_string(),
                })
            }
            "entity.create.robot_cell" => {
                let index = self.graph.entity_count() + 1;
                let entity = sample_entity(
                    "RobotCell",
                    &format!("ent_cell_{index:03}"),
                    &format!("RobotCell-{index:03}"),
                    serde_json::json!({
                        "robotIds": [],
                        "equipmentIds": [],
                        "sequenceIds": []
                    }),
                );
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "cellule robotique ajoutee".to_string(),
                })
            }
            "entity.create.sensor_rig" => {
                let index = self.graph.entity_count() + 1;
                let entity = sample_entity(
                    "SensorRig",
                    &format!("ent_rig_{index:03}"),
                    &format!("SensorRig-{index:03}"),
                    serde_json::json!({
                        "sensorTypes": ["lidar", "camera"],
                        "source": "desktop_session"
                    }),
                );
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "rig capteurs ajoute".to_string(),
                })
            }
            "entity.create.external_endpoint" => {
                let endpoint_index = self.graph.endpoint_count() + 1;
                let (endpoint, stream) = sample_wireless_endpoint_with_stream(endpoint_index);
                self.graph
                    .apply_command(CoreCommand::RegisterEndpoint(endpoint))
                    .map_err(|error| error.to_string())?;
                self.graph
                    .apply_command(CoreCommand::RegisterStream(stream))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "endpoint externe et flux telemetrie ajoutes".to_string(),
                })
            }
            "plugin.manage" => {
                let plugin_id = "plg.desktop.runtime";
                if !self
                    .graph
                    .document()
                    .plugin_manifests
                    .contains_key(plugin_id)
                {
                    let manifest = sample_plugin_manifest(self.graph.plugin_count() + 1, plugin_id);
                    validate_manifest(&manifest).map_err(|error| error.to_string())?;
                    self.graph
                        .apply_command(CoreCommand::InstallPlugin(manifest))
                        .map_err(|error| error.to_string())?;
                    self.graph
                        .apply_command(CoreCommand::SetPluginEnabled {
                            plugin_id: plugin_id.to_string(),
                            enabled: true,
                        })
                        .map_err(|error| error.to_string())?;
                    Ok(CommandResult {
                        command_id: command_id.to_string(),
                        status: "applied".to_string(),
                        message: "plugin runtime installe et active".to_string(),
                    })
                } else {
                    let currently_enabled =
                        *self.graph.plugin_state().get(plugin_id).unwrap_or(&false);
                    self.graph
                        .apply_command(CoreCommand::SetPluginEnabled {
                            plugin_id: plugin_id.to_string(),
                            enabled: !currently_enabled,
                        })
                        .map_err(|error| error.to_string())?;
                    Ok(CommandResult {
                        command_id: command_id.to_string(),
                        status: "applied".to_string(),
                        message: if currently_enabled {
                            "plugin runtime desactive".to_string()
                        } else {
                            "plugin runtime active".to_string()
                        },
                    })
                }
            }
            "simulation.run.start" => {
                self.push_system_activity("simulation.run.started", Some(self.fixture_id.clone()));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "simulated".to_string(),
                    message: "run de simulation lance en mode shell".to_string(),
                })
            }
            "test.run_fixture" => {
                self.push_system_activity("test.fixture.executed", Some(self.fixture_id.clone()));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "simulated".to_string(),
                    message: "fixture de test rejouee".to_string(),
                })
            }
            "analyze.validation_report" => {
                self.push_system_activity(
                    "analysis.validation.generated",
                    Some(self.fixture_id.clone()),
                );
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "simulated".to_string(),
                    message: "rapport de validation genere".to_string(),
                })
            }
            _ => {
                self.push_system_activity("command.simulated", Some(command_id.to_string()));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "simulated".to_string(),
                    message: "commande simulee dans le shell desktop".to_string(),
                })
            }
        }
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixtures_root() -> PathBuf {
    repo_root().join("examples/projects")
}

fn serialized_variant<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|json| json.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_string())
}

fn endpoint_address_label(
    host: Option<&str>,
    port: Option<u16>,
    path: Option<&str>,
    device_id: Option<&str>,
) -> String {
    if let Some(host) = host {
        return match (port, path) {
            (Some(port), Some(path)) => format!("{host}:{port}{path}"),
            (Some(port), None) => format!("{host}:{port}"),
            (None, Some(path)) => format!("{host}{path}"),
            (None, None) => host.to_string(),
        };
    }

    if let Some(device_id) = device_id {
        return device_id.to_string();
    }

    "n/a".to_string()
}

fn load_project_document(project_id: &str) -> Result<ProjectDocument, String> {
    faero_storage::load_project(fixtures_root().join(project_id))
        .map_err(|error| format!("failed to load fixture `{project_id}`: {error}"))
}

fn status_from_document(project_id: &str, document: &ProjectDocument) -> BackendStatus {
    BackendStatus {
        runtime: "tauri-rust".to_string(),
        fixture_id: project_id.to_string(),
        project_name: document.metadata.name.clone(),
        entity_count: document.nodes.len(),
        endpoint_count: document.endpoints.len(),
        stream_count: document.streams.len(),
        plugin_count: document.plugin_manifests.len(),
    }
}

fn build_project_snapshot_from_document(
    project_id: &str,
    document: &ProjectDocument,
    extra_activity: &[ActivityEntry],
) -> ProjectSnapshot {
    let status = status_from_document(project_id, document);
    let details = ProjectDetails {
        project_id: document.metadata.project_id.clone(),
        format_version: document.metadata.format_version.clone(),
        default_frame: document.metadata.default_frame.clone(),
        root_scene_id: document.metadata.root_scene_id.clone(),
        active_configuration_id: document.metadata.active_configuration_id.clone(),
    };

    let entities = document
        .nodes
        .values()
        .map(|entity| EntitySummary {
            id: entity.id.clone(),
            entity_type: entity.entity_type.clone(),
            name: entity.name.clone(),
            revision: entity.revision.clone(),
            status: entity.status.clone(),
        })
        .collect::<Vec<_>>();

    let endpoints = document
        .endpoints
        .values()
        .map(|endpoint| EndpointSummary {
            id: endpoint.id.clone(),
            name: endpoint.name.clone(),
            endpoint_type: serialized_variant(&endpoint.endpoint_type),
            transport_kind: endpoint.transport_profile.transport_kind.clone(),
            mode: serialized_variant(&endpoint.mode),
            address: endpoint_address_label(
                endpoint.addressing.host.as_deref(),
                endpoint.addressing.port,
                endpoint.addressing.path.as_deref(),
                endpoint.addressing.device_id.as_deref(),
            ),
            status: endpoint.status.clone(),
        })
        .collect::<Vec<_>>();

    let streams = document
        .streams
        .values()
        .map(|stream| StreamSummary {
            id: stream.id.clone(),
            name: stream.name.clone(),
            endpoint_id: stream.endpoint_id.clone(),
            stream_type: stream.stream_type.clone(),
            direction: serialized_variant(&stream.direction),
            status: stream.status.clone(),
        })
        .collect::<Vec<_>>();

    let plugins = document
        .plugin_manifests
        .values()
        .map(|plugin| PluginSummary {
            plugin_id: plugin.plugin_id.clone(),
            version: plugin.version.clone(),
            enabled: *document
                .plugin_states
                .get(&plugin.plugin_id)
                .unwrap_or(&false),
            status: plugin.status.clone(),
        })
        .collect::<Vec<_>>();

    let mut recent_activity = document
        .commands
        .iter()
        .map(|command| ActivityEntry {
            id: command.command_id.clone(),
            channel: "command".to_string(),
            kind: command.kind.clone(),
            timestamp: command.timestamp.clone(),
            target_id: command.target_id.clone(),
        })
        .chain(document.events.iter().map(|event| ActivityEntry {
            id: event.event_id.clone(),
            channel: "event".to_string(),
            kind: event.kind.clone(),
            timestamp: event.timestamp.clone(),
            target_id: event.target_id.clone(),
        }))
        .chain(extra_activity.iter().cloned())
        .collect::<Vec<_>>();
    recent_activity.sort_by(|left, right| {
        right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| right.id.cmp(&left.id))
    });
    recent_activity.truncate(12);

    ProjectSnapshot {
        status,
        details,
        entities,
        endpoints,
        streams,
        plugins,
        recent_activity,
    }
}

fn build_backend_status(project_id: &str) -> Result<BackendStatus, String> {
    let document = load_project_document(project_id)?;
    Ok(status_from_document(project_id, &document))
}

fn build_project_snapshot(project_id: &str) -> Result<ProjectSnapshot, String> {
    let document = load_project_document(project_id)?;
    Ok(build_project_snapshot_from_document(
        project_id,
        &document,
        &[],
    ))
}

fn available_fixtures() -> Result<Vec<FixtureProject>, String> {
    let root = fixtures_root();
    let mut entries = fs::read_dir(&root)
        .map_err(|error| format!("failed to read fixture root `{}`: {error}", root.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to enumerate fixture root `{}`: {error}",
                root.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut fixtures = Vec::new();
    for entry in entries {
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "failed to read fixture entry type `{}`: {error}",
                entry.path().display()
            )
        })?;
        if !file_type.is_dir() {
            continue;
        }

        let fixture_id = entry.file_name().to_string_lossy().to_string();
        let document = faero_storage::load_project(entry.path())
            .map_err(|error| format!("failed to index fixture `{fixture_id}`: {error}"))?;

        fixtures.push(FixtureProject {
            id: fixture_id,
            project_name: document.metadata.name,
        });
    }

    Ok(fixtures)
}

fn sample_entity(entity_type: &str, id: &str, name: &str, data: serde_json::Value) -> EntityRecord {
    EntityRecord {
        id: id.to_string(),
        entity_type: entity_type.to_string(),
        name: name.to_string(),
        revision: "rev_seed".to_string(),
        status: "active".to_string(),
        data,
    }
}

fn sample_wireless_endpoint_with_stream(index: usize) -> (ExternalEndpoint, TelemetryStream) {
    let mut endpoint = faero_integration::stub_wifi_endpoint();
    endpoint.id = format!("ext_wifi_{index:03}");
    endpoint.name = format!("Wireless Edge {index:03}");
    endpoint.addressing.host = Some(format!("wireless-edge-{index:03}.local"));
    endpoint.signal_map_ids = vec![format!("sig_wireless_{index:03}")];

    let stream = TelemetryStream {
        id: format!("str_wifi_{index:03}"),
        name: format!("Telemetry-{index:03}"),
        endpoint_id: endpoint.id.clone(),
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
    };

    (endpoint, stream)
}

fn sample_plugin_manifest(index: usize, plugin_id: &str) -> PluginManifest {
    PluginManifest {
        id: format!("ent_plugin_{index:03}"),
        plugin_id: plugin_id.to_string(),
        version: "0.1.0".to_string(),
        capabilities: vec!["panel".to_string(), "command".to_string()],
        permissions: vec!["project.read".to_string(), "plugin.ui.mount".to_string()],
        entrypoints: vec!["plugins/desktop-runtime/index.js".to_string()],
        compatibility: vec!["faero-core@0.1".to_string()],
        status: "installed".to_string(),
    }
}

fn lock_workspace<'a>(
    state: &'a State<'_, SharedWorkspace>,
) -> Result<std::sync::MutexGuard<'a, WorkspaceSession>, String> {
    state
        .lock()
        .map_err(|_| "workspace session lock poisoned".to_string())
}

#[tauri::command]
fn backend_status() -> BackendStatus {
    build_backend_status(DEFAULT_FIXTURE_ID).unwrap_or(BackendStatus {
        runtime: "tauri-rust".to_string(),
        fixture_id: DEFAULT_FIXTURE_ID.to_string(),
        project_name: "FutureAero Desktop Shell".to_string(),
        entity_count: 0,
        endpoint_count: 0,
        stream_count: 0,
        plugin_count: 0,
    })
}

#[tauri::command]
fn available_fixture_projects() -> Result<Vec<FixtureProject>, String> {
    available_fixtures()
}

#[tauri::command]
fn load_fixture_project(project_id: String) -> Result<BackendStatus, String> {
    build_backend_status(&project_id)
}

#[tauri::command]
fn load_project_snapshot(project_id: String) -> Result<ProjectSnapshot, String> {
    build_project_snapshot(&project_id)
}

#[tauri::command]
fn workspace_bootstrap(state: State<'_, SharedWorkspace>) -> Result<WorkspaceBootstrap, String> {
    let fixtures = available_fixtures()?;
    let session = lock_workspace(&state)?;
    Ok(WorkspaceBootstrap {
        fixtures,
        snapshot: session.snapshot(),
    })
}

#[tauri::command]
fn workspace_load_fixture(
    project_id: String,
    state: State<'_, SharedWorkspace>,
) -> Result<ProjectSnapshot, String> {
    let mut session = lock_workspace(&state)?;
    *session = WorkspaceSession::load_fixture(&project_id)?;
    Ok(session.snapshot())
}

#[tauri::command]
fn workspace_execute_command(
    command_id: String,
    state: State<'_, SharedWorkspace>,
) -> Result<CommandResponse, String> {
    let mut session = lock_workspace(&state)?;
    let result = session.execute_command(&command_id)?;
    Ok(CommandResponse {
        snapshot: session.snapshot(),
        result,
    })
}

#[tauri::command]
fn ai_runtime_status() -> AiRuntimeStatus {
    query_ai_runtime_status()
}

#[tauri::command]
fn ai_chat_send_message(
    message: String,
    locale: String,
    history: Vec<AiConversationMessage>,
    state: State<'_, SharedWorkspace>,
) -> Result<AiChatResponse, String> {
    let document = {
        let session = lock_workspace(&state)?;
        session.graph.document().clone()
    };

    chat_with_project(&document, &locale, &history, &message).map_err(|error| error.to_string())
}

fn main() {
    let workspace = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
        .unwrap_or_else(|_| WorkspaceSession::empty("FutureAero Session"));

    tauri::Builder::default()
        .manage(Mutex::new(workspace))
        .invoke_handler(tauri::generate_handler![
            backend_status,
            available_fixture_projects,
            load_fixture_project,
            load_project_snapshot,
            workspace_bootstrap,
            workspace_load_fixture,
            workspace_execute_command,
            ai_runtime_status,
            ai_chat_send_message
        ])
        .run(tauri::generate_context!())
        .expect("error while running FutureAero desktop shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_fixture_projects_from_examples_directory() {
        let fixtures = available_fixture_projects().expect("fixtures should index");

        assert!(
            fixtures
                .iter()
                .any(|fixture| fixture.id == "empty-project.faero")
        );
        assert!(
            fixtures
                .iter()
                .any(|fixture| fixture.id == "pick-and-place-demo.faero")
        );
    }

    #[test]
    fn loads_pick_and_place_fixture_into_backend_status() {
        let status = build_backend_status(DEFAULT_FIXTURE_ID).expect("fixture should load");

        assert_eq!(status.runtime, "tauri-rust");
        assert_eq!(status.fixture_id, DEFAULT_FIXTURE_ID);
        assert_eq!(status.project_name, "Pick And Place Demo");
        assert_eq!(status.entity_count, 2);
        assert_eq!(status.endpoint_count, 1);
        assert_eq!(status.stream_count, 1);
        assert_eq!(status.plugin_count, 1);
    }

    #[test]
    fn project_snapshot_exposes_workspace_entities_and_activity() {
        let snapshot = build_project_snapshot(DEFAULT_FIXTURE_ID).expect("snapshot should load");

        assert_eq!(snapshot.details.project_id, "prj_1001");
        assert_eq!(snapshot.entities.len(), 2);
        assert_eq!(snapshot.endpoints.len(), 1);
        assert_eq!(snapshot.streams.len(), 1);
        assert_eq!(snapshot.plugins.len(), 1);
        assert_eq!(snapshot.endpoints[0].transport_kind, "robot_controller");
        assert!(snapshot.plugins[0].enabled);
        assert!(!snapshot.recent_activity.is_empty());
    }

    #[test]
    fn workspace_session_applies_entity_endpoint_and_plugin_commands() {
        let mut session = WorkspaceSession::empty("Session");

        session
            .execute_command("entity.create.part")
            .expect("part should be created");
        session
            .execute_command("entity.create.external_endpoint")
            .expect("endpoint should be created");
        session
            .execute_command("plugin.manage")
            .expect("plugin should install and enable");

        let snapshot = session.snapshot();
        assert_eq!(snapshot.status.entity_count, 1);
        assert_eq!(snapshot.status.endpoint_count, 1);
        assert_eq!(snapshot.status.stream_count, 1);
        assert_eq!(snapshot.status.plugin_count, 1);
        assert!(snapshot.plugins[0].enabled);
    }

    #[test]
    fn workspace_session_records_simulated_commands_in_system_activity() {
        let mut session = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
            .expect("fixture-backed session should load");

        let result = session
            .execute_command("simulation.run.start")
            .expect("simulation command should be simulated");

        assert_eq!(result.status, "simulated");
        assert!(
            session
                .snapshot()
                .recent_activity
                .iter()
                .any(|entry| entry.channel == "system")
        );
    }
}
