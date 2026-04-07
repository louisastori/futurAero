use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use faero_ai::{
    AiChatResponse, AiConversationMessage, AiRuntimeStatus, chat_with_project,
    query_runtime_status as query_ai_runtime_status,
};
use faero_core::{CoreCommand, ProjectGraph};
use faero_geometry::{
    ExtrusionDefinition, MaterialProfile, SketchConstraintState, rectangular_profile,
    regenerate_extrusion,
};
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
    detail: Option<String>,
    part_geometry: Option<PartGeometrySummary>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PartGeometrySummary {
    state: String,
    width_mm: f64,
    height_mm: f64,
    depth_mm: f64,
    point_count: usize,
    perimeter_mm: f64,
    area_mm2: f64,
    volume_mm3: f64,
    estimated_mass_grams: f64,
    material_name: String,
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
            "project.save" => {
                let output_path =
                    save_document_copy("saves", &self.fixture_id, self.graph.document())?;
                self.push_system_activity("project.saved", Some(output_path.display().to_string()));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!("session enregistree dans {}", output_path.display()),
                })
            }
            "project.save_all" => {
                let output_path =
                    save_document_copy("saves", &self.fixture_id, self.graph.document())?;
                self.push_system_activity(
                    "project.saved_all",
                    Some(output_path.display().to_string()),
                );
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "toutes les donnees session enregistrees dans {}",
                        output_path.display()
                    ),
                })
            }
            "project.import" => {
                let imported = import_document_copy(self.graph.document());
                let imported_id = imported.metadata.project_id.clone();
                self.fixture_id = format!("session:{imported_id}");
                self.graph = ProjectGraph::from_document(imported);
                self.system_activity.clear();
                self.system_counter = 1;
                self.push_system_activity("project.imported", Some(self.fixture_id.clone()));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "copie importee dans une nouvelle session editable".to_string(),
                })
            }
            "project.export" => {
                let output_path =
                    save_document_copy("exports", &self.fixture_id, self.graph.document())?;
                self.push_system_activity(
                    "project.exported",
                    Some(output_path.display().to_string()),
                );
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!("export du projet cree dans {}", output_path.display()),
                })
            }
            "entity.create.part" => {
                let index = self.graph.entity_count() + 1;
                let (entity, summary) = sample_parametric_part_entity(index)?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "piece parametrique regeneree: {:.1} x {:.1} x {:.1} mm | {:.1} g",
                        summary.width_mm,
                        summary.height_mm,
                        summary.depth_mm,
                        summary.estimated_mass_grams
                    ),
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

fn artifacts_root() -> PathBuf {
    repo_root().join("artifacts")
}

fn fixtures_root() -> PathBuf {
    repo_root().join("examples/projects")
}

fn command_timestamp_token() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn sanitize_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();

    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "workspace".to_string()
    } else {
        trimmed.to_string()
    }
}

fn save_document_copy(
    category: &str,
    fixture_id: &str,
    document: &ProjectDocument,
) -> Result<PathBuf, String> {
    let file_name = format!(
        "{}-{}.faero",
        sanitize_path_segment(fixture_id),
        command_timestamp_token()
    );
    let output_path = artifacts_root().join(category).join(file_name);
    faero_storage::save_project(&output_path, document).map_err(|error| {
        format!(
            "failed to save project copy `{}`: {error}",
            output_path.display()
        )
    })?;
    Ok(output_path)
}

fn import_document_copy(document: &ProjectDocument) -> ProjectDocument {
    let token = command_timestamp_token();
    let mut imported = document.clone();
    imported.metadata.project_id = format!("prj_import_{token}");
    imported.metadata.name = format!("{} Imported", document.metadata.name);
    imported
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
        .map(|entity| {
            let part_geometry = part_geometry_summary_from_entity(entity);
            EntitySummary {
                id: entity.id.clone(),
                entity_type: entity.entity_type.clone(),
                name: entity.name.clone(),
                revision: entity.revision.clone(),
                status: entity.status.clone(),
                detail: part_geometry.as_ref().map(format_part_entity_detail),
                part_geometry,
            }
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

fn sketch_constraint_state_label(state: SketchConstraintState) -> &'static str {
    match state {
        SketchConstraintState::UnderConstrained => "under_constrained",
        SketchConstraintState::WellConstrained => "well_constrained",
        SketchConstraintState::OverConstrained => "over_constrained",
    }
}

fn sample_parametric_part_entity(
    index: usize,
) -> Result<(EntityRecord, PartGeometrySummary), String> {
    let width_mm = 120.0 + (index as f64 * 12.0);
    let height_mm = 80.0 + (index as f64 * 6.0);
    let depth_mm = 10.0 + (index as f64 * 2.0);
    let profile = rectangular_profile(width_mm, height_mm, 4).map_err(|error| error.to_string())?;
    let material = MaterialProfile::aluminum_6061();
    let regeneration = regenerate_extrusion(&profile, &ExtrusionDefinition { depth_mm }, &material)
        .map_err(|error| error.to_string())?;

    let summary = PartGeometrySummary {
        state: sketch_constraint_state_label(regeneration.state).to_string(),
        width_mm,
        height_mm,
        depth_mm,
        point_count: regeneration.point_count,
        perimeter_mm: regeneration.perimeter_mm,
        area_mm2: regeneration.area_mm2,
        volume_mm3: regeneration.volume_mm3,
        estimated_mass_grams: regeneration.estimated_mass_grams,
        material_name: material.name.clone(),
    };

    let entity = sample_entity(
        "Part",
        &format!("ent_part_{index:03}"),
        &format!("Part-{index:03}"),
        serde_json::json!({
            "geometrySource": "parametric_sketch_extrude",
            "parameterSet": {
                "widthMm": width_mm,
                "heightMm": height_mm,
                "depthMm": depth_mm
            },
            "sketch": {
                "profileType": "rectangle",
                "points": profile.points,
                "solvedConstraintCount": profile.solved_constraint_count
            },
            "extrusion": {
                "depthMm": depth_mm
            },
            "material": {
                "name": material.name,
                "densityKgM3": material.density_kg_m3
            },
            "summary": {
                "state": summary.state.clone(),
                "pointCount": summary.point_count,
                "perimeterMm": summary.perimeter_mm,
                "areaMm2": summary.area_mm2,
                "volumeMm3": summary.volume_mm3,
                "estimatedMassGrams": summary.estimated_mass_grams,
                "materialName": summary.material_name.clone()
            },
            "centroid": {
                "xMm": regeneration.centroid_x_mm,
                "yMm": regeneration.centroid_y_mm
            }
        }),
    );

    Ok((entity, summary))
}

fn number_from_value(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key)?.as_f64()
}

fn string_from_value(value: &serde_json::Value, key: &str) -> Option<String> {
    Some(value.get(key)?.as_str()?.to_string())
}

fn part_geometry_summary_from_entity(entity: &EntityRecord) -> Option<PartGeometrySummary> {
    if entity.entity_type != "Part" {
        return None;
    }

    let parameters = entity.data.get("parameterSet")?;
    let summary = entity.data.get("summary")?;

    Some(PartGeometrySummary {
        state: string_from_value(summary, "state")?,
        width_mm: number_from_value(parameters, "widthMm")?,
        height_mm: number_from_value(parameters, "heightMm")?,
        depth_mm: number_from_value(parameters, "depthMm")?,
        point_count: summary.get("pointCount")?.as_u64()? as usize,
        perimeter_mm: number_from_value(summary, "perimeterMm")?,
        area_mm2: number_from_value(summary, "areaMm2")?,
        volume_mm3: number_from_value(summary, "volumeMm3")?,
        estimated_mass_grams: number_from_value(summary, "estimatedMassGrams")?,
        material_name: string_from_value(summary, "materialName")?,
    })
}

fn format_part_entity_detail(summary: &PartGeometrySummary) -> String {
    format!(
        "{:.1} x {:.1} x {:.1} mm | {:.1} g",
        summary.width_mm, summary.height_mm, summary.depth_mm, summary.estimated_mass_grams
    )
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
    selected_model: Option<String>,
    state: State<'_, SharedWorkspace>,
) -> Result<AiChatResponse, String> {
    let document = {
        let session = lock_workspace(&state)?;
        session.graph.document().clone()
    };

    chat_with_project(
        &document,
        &locale,
        &history,
        &message,
        selected_model.as_deref(),
    )
    .map_err(|error| error.to_string())
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
    use std::fs;

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
    fn created_parametric_part_exposes_geometry_summary_in_snapshot() {
        let mut session = WorkspaceSession::empty("Session");

        let result = session
            .execute_command("entity.create.part")
            .expect("part should be created");
        let snapshot = session.snapshot();
        let part = snapshot
            .entities
            .first()
            .expect("part summary should exist");
        let geometry = part
            .part_geometry
            .as_ref()
            .expect("part geometry summary should exist");

        assert_eq!(result.status, "applied");
        assert!(result.message.contains("piece parametrique regeneree"));
        assert_eq!(part.entity_type, "Part");
        assert_eq!(geometry.state, "well_constrained");
        assert!(geometry.width_mm > 0.0);
        assert!(geometry.area_mm2 > 0.0);
        assert!(
            part.detail
                .as_ref()
                .is_some_and(|detail| detail.contains("mm"))
        );
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

    #[test]
    fn save_document_copy_writes_a_loadable_project_under_artifacts() {
        let document = load_project_document(DEFAULT_FIXTURE_ID).expect("fixture should load");
        let output_path =
            save_document_copy("test-saves", "fixture:test", &document).expect("save should work");

        let reloaded =
            faero_storage::load_project(&output_path).expect("saved project should reload");
        assert_eq!(reloaded.metadata.name, document.metadata.name);
        assert!(output_path.exists());

        fs::remove_dir_all(output_path).expect("saved test artifact should be removable");
    }

    #[test]
    fn import_command_creates_a_new_editable_session() {
        let mut session = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
            .expect("fixture-backed session should load");

        let result = session
            .execute_command("project.import")
            .expect("import command should succeed");

        assert_eq!(result.status, "applied");
        assert!(session.fixture_id.starts_with("session:prj_import_"));
        assert!(
            session
                .graph
                .document()
                .metadata
                .name
                .ends_with(" Imported")
        );
        assert!(
            session
                .snapshot()
                .recent_activity
                .iter()
                .any(|entry| entry.kind == "project.imported")
        );
    }
}
