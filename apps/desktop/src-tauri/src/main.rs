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
use faero_assembly::{
    AssemblySolveReport, AssemblySolveStatus, MateConstraint, MateType, Occurrence, Transform3D,
    solve_assembly,
};
use faero_core::{CoreCommand, ProjectGraph};
use faero_geometry::{
    ExtrusionDefinition, MaterialProfile, SketchConstraintState, rectangular_profile,
    regenerate_extrusion,
};
use faero_plugin_host::validate_manifest;
use faero_robotics::{CartesianPose, RobotTarget, validate_sequence};
use faero_safety::{SafetyInterlock, SafetyStatus, SafetyZone, SafetyZoneKind, evaluate_safety};
use faero_sim::{SimulationRequest, SimulationStatus, run_simulation};
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
    assembly_summary: Option<AssemblyEntitySummary>,
    robot_cell_summary: Option<RobotCellEntitySummary>,
    simulation_run_summary: Option<SimulationRunEntitySummary>,
    safety_report_summary: Option<SafetyReportEntitySummary>,
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
struct AssemblyEntitySummary {
    status: String,
    occurrence_count: usize,
    mate_count: usize,
    degrees_of_freedom_estimate: usize,
    warning_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RobotCellEntitySummary {
    target_count: usize,
    path_length_mm: f64,
    max_segment_mm: f64,
    estimated_cycle_time_ms: u32,
    safety_zone_count: usize,
    warning_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SimulationRunEntitySummary {
    status: String,
    collision_count: u32,
    cycle_time_ms: u32,
    max_tracking_error_mm: f64,
    energy_estimate_j: f64,
    timeline_sample_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SafetyReportEntitySummary {
    status: String,
    inhibited: bool,
    active_zone_count: usize,
    blocking_interlock_count: usize,
    advisory_zone_count: usize,
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

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PartRegenerationInput {
    width_mm: f64,
    height_mm: f64,
    depth_mm: f64,
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

    fn latest_parametric_part(&self) -> Option<EntityRecord> {
        self.graph
            .document()
            .nodes
            .values()
            .rev()
            .find(|entity| part_geometry_summary_from_entity(entity).is_some())
            .cloned()
    }

    fn regenerate_latest_part(
        &mut self,
        width_mm: f64,
        height_mm: f64,
        depth_mm: f64,
    ) -> Result<CommandResult, String> {
        let latest_part = self
            .latest_parametric_part()
            .ok_or_else(|| "aucune piece parametrique a regenerer".to_string())?;
        let (entity, summary) = build_parametric_part_entity(
            &latest_part.id,
            &latest_part.name,
            width_mm,
            height_mm,
            depth_mm,
        )?;

        self.graph
            .apply_command(CoreCommand::ReplaceEntity(entity))
            .map_err(|error| error.to_string())?;
        self.push_system_activity("part.regenerated", Some(latest_part.id));

        Ok(CommandResult {
            command_id: "build.regenerate_part".to_string(),
            status: "applied".to_string(),
            message: format!(
                "piece regeneree: {:.1} x {:.1} x {:.1} mm | {:.1} g",
                summary.width_mm, summary.height_mm, summary.depth_mm, summary.estimated_mass_grams
            ),
        })
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
            "build.regenerate_part" => {
                let latest_part = self
                    .latest_parametric_part()
                    .ok_or_else(|| "aucune piece parametrique a regenerer".to_string())?;
                let summary = part_geometry_summary_from_entity(&latest_part)
                    .ok_or_else(|| "piece parametrique invalide".to_string())?;
                self.regenerate_latest_part(summary.width_mm, summary.height_mm, summary.depth_mm)
            }
            "entity.create.assembly" => {
                let index = self.graph.entity_count() + 1;
                let mut part_ids = self
                    .graph
                    .document()
                    .nodes
                    .values()
                    .filter(|entity| entity.entity_type == "Part")
                    .map(|entity| entity.id.clone())
                    .collect::<Vec<_>>();
                while part_ids.len() < 2 {
                    let part_index = self.graph.entity_count() + 1;
                    let (part_entity, _) = sample_parametric_part_entity(part_index)?;
                    part_ids.push(part_entity.id.clone());
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(part_entity))
                        .map_err(|error| error.to_string())?;
                }
                let (entity, summary) = build_assembly_entity(
                    &format!("ent_asm_{index:03}"),
                    &format!("Assembly-{index:03}"),
                    &part_ids[..part_ids.len().min(3)],
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "assemblage ajoute: {} occurrences | {} mates | {}",
                        summary.occurrence_count, summary.mate_count, summary.status
                    ),
                })
            }
            "entity.create.robot_cell" => {
                let index = self.graph.entity_count() + 1;
                let (entity, summary) = build_robot_cell_entity(
                    &format!("ent_cell_{index:03}"),
                    &format!("RobotCell-{index:03}"),
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "cellule robotique ajoutee: {} cibles | {:.0} mm | {} ms",
                        summary.target_count,
                        summary.path_length_mm,
                        summary.estimated_cycle_time_ms
                    ),
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
                let robot_cell = if let Some(existing) = self
                    .graph
                    .document()
                    .nodes
                    .values()
                    .filter(|entity| robot_cell_summary_from_entity(entity).is_some())
                    .max_by(|left, right| left.id.cmp(&right.id))
                    .cloned()
                {
                    existing
                } else {
                    let index = self.graph.entity_count() + 1;
                    let (entity, _) = build_robot_cell_entity(
                        &format!("ent_cell_{index:03}"),
                        &format!("RobotCell-{index:03}"),
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(entity.clone()))
                        .map_err(|error| error.to_string())?;
                    entity
                };

                let run_index = self.graph.entity_count() + 1;
                let (entity, summary) = build_simulation_run_entity(
                    &format!("ent_run_{run_index:03}"),
                    &format!("SimulationRun-{run_index:03}"),
                    &robot_cell,
                    self.graph.endpoint_count(),
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.push_system_activity("simulation.run.completed", Some(entity.id));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "run de simulation termine: {} | {} ms | {} collision(s)",
                        summary.status, summary.cycle_time_ms, summary.collision_count
                    ),
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
            "analyze.safety" => {
                let robot_cell = if let Some(existing) = self
                    .graph
                    .document()
                    .nodes
                    .values()
                    .filter(|entity| robot_cell_summary_from_entity(entity).is_some())
                    .max_by(|left, right| left.id.cmp(&right.id))
                    .cloned()
                {
                    existing
                } else {
                    let index = self.graph.entity_count() + 1;
                    let (entity, _) = build_robot_cell_entity(
                        &format!("ent_cell_{index:03}"),
                        &format!("RobotCell-{index:03}"),
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(entity.clone()))
                        .map_err(|error| error.to_string())?;
                    entity
                };
                let report_index = self.graph.entity_count() + 1;
                let (entity, summary) = build_safety_report_entity(
                    &format!("ent_safe_{report_index:03}"),
                    &format!("SafetyReport-{report_index:03}"),
                    &robot_cell,
                    "robot.move",
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.push_system_activity("analysis.safety.generated", Some(entity.id));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "rapport safety genere: {} | {} zone(s) actives | {} blocage(s)",
                        summary.status, summary.active_zone_count, summary.blocking_interlock_count
                    ),
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
            let assembly_summary = assembly_summary_from_entity(entity);
            let robot_cell_summary = robot_cell_summary_from_entity(entity);
            let simulation_run_summary = simulation_run_summary_from_entity(entity);
            let safety_report_summary = safety_report_summary_from_entity(entity);
            EntitySummary {
                id: entity.id.clone(),
                entity_type: entity.entity_type.clone(),
                name: entity.name.clone(),
                revision: entity.revision.clone(),
                status: entity.status.clone(),
                detail: part_geometry
                    .as_ref()
                    .map(format_part_entity_detail)
                    .or_else(|| assembly_summary.as_ref().map(format_assembly_entity_detail))
                    .or_else(|| {
                        robot_cell_summary
                            .as_ref()
                            .map(format_robot_cell_entity_detail)
                    })
                    .or_else(|| {
                        simulation_run_summary
                            .as_ref()
                            .map(format_simulation_run_entity_detail)
                    })
                    .or_else(|| {
                        safety_report_summary
                            .as_ref()
                            .map(format_safety_report_entity_detail)
                    }),
                part_geometry,
                assembly_summary,
                robot_cell_summary,
                simulation_run_summary,
                safety_report_summary,
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

fn build_parametric_part_entity(
    id: &str,
    name: &str,
    width_mm: f64,
    height_mm: f64,
    depth_mm: f64,
) -> Result<(EntityRecord, PartGeometrySummary), String> {
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
        id,
        name,
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

fn sample_parametric_part_entity(
    index: usize,
) -> Result<(EntityRecord, PartGeometrySummary), String> {
    let width_mm = 120.0 + (index as f64 * 12.0);
    let height_mm = 80.0 + (index as f64 * 6.0);
    let depth_mm = 10.0 + (index as f64 * 2.0);
    build_parametric_part_entity(
        &format!("ent_part_{index:03}"),
        &format!("Part-{index:03}"),
        width_mm,
        height_mm,
        depth_mm,
    )
}

fn build_assembly_entity(
    id: &str,
    name: &str,
    part_entity_ids: &[String],
) -> Result<(EntityRecord, AssemblyEntitySummary), String> {
    let occurrences = part_entity_ids
        .iter()
        .enumerate()
        .map(|(index, part_id)| Occurrence {
            id: format!("occ_{:03}", index + 1),
            part_entity_id: part_id.clone(),
            transform: Transform3D {
                x_mm: index as f64 * 80.0,
                ..Transform3D::default()
            },
        })
        .collect::<Vec<_>>();

    let constraints = occurrences
        .windows(2)
        .enumerate()
        .map(|(index, pair)| MateConstraint {
            id: format!("mate_{:03}", index + 1),
            left_occurrence_id: pair[0].id.clone(),
            right_occurrence_id: pair[1].id.clone(),
            mate_type: if index == 0 {
                MateType::Coincident
            } else {
                MateType::Offset {
                    distance_mm: 25.0 * index as f64,
                }
            },
        })
        .collect::<Vec<_>>();

    let report = solve_assembly(&occurrences, &constraints).map_err(|error| error.to_string())?;
    let summary = AssemblyEntitySummary {
        status: assembly_solve_status_label(report.status).to_string(),
        occurrence_count: report.solved_occurrences.len(),
        mate_count: report.total_mate_count,
        degrees_of_freedom_estimate: report.degrees_of_freedom_estimate,
        warning_count: report.warnings.len(),
    };
    let entity = sample_entity(
        "Assembly",
        id,
        name,
        serde_json::json!({
            "occurrences": occurrences,
            "mateConstraints": constraints,
            "solveReport": assembly_solve_report_json(&report),
        }),
    );

    Ok((entity, summary))
}

fn assembly_solve_report_json(report: &AssemblySolveReport) -> serde_json::Value {
    serde_json::json!({
        "status": assembly_solve_status_label(report.status),
        "occurrenceCount": report.solved_occurrences.len(),
        "mateCount": report.total_mate_count,
        "degreesOfFreedomEstimate": report.degrees_of_freedom_estimate,
        "warningCount": report.warnings.len(),
        "warnings": report.warnings.clone(),
        "solvedOccurrences": report.solved_occurrences.clone(),
    })
}

fn sample_robot_targets() -> Vec<RobotTarget> {
    vec![
        RobotTarget {
            id: "pick".to_string(),
            pose: CartesianPose {
                x_mm: 0.0,
                y_mm: 0.0,
                z_mm: 120.0,
            },
            nominal_speed_mm_s: 250,
            dwell_time_ms: 120,
        },
        RobotTarget {
            id: "transfer".to_string(),
            pose: CartesianPose {
                x_mm: 450.0,
                y_mm: 60.0,
                z_mm: 240.0,
            },
            nominal_speed_mm_s: 320,
            dwell_time_ms: 40,
        },
        RobotTarget {
            id: "place".to_string(),
            pose: CartesianPose {
                x_mm: 860.0,
                y_mm: 120.0,
                z_mm: 140.0,
            },
            nominal_speed_mm_s: 240,
            dwell_time_ms: 160,
        },
    ]
}

fn sample_safety_zones() -> Vec<SafetyZone> {
    vec![
        SafetyZone {
            id: "zone_warning_perimeter".to_string(),
            kind: SafetyZoneKind::Warning,
            active: true,
        },
        SafetyZone {
            id: "zone_lidar_protective".to_string(),
            kind: SafetyZoneKind::LidarProtective,
            active: false,
        },
    ]
}

fn sample_safety_interlocks() -> Vec<SafetyInterlock> {
    vec![
        SafetyInterlock {
            id: "int_warning_reduce_speed".to_string(),
            source_zone_id: "zone_warning_perimeter".to_string(),
            inhibited_action: "robot.speed.up".to_string(),
            requires_manual_reset: false,
        },
        SafetyInterlock {
            id: "int_lidar_stop_move".to_string(),
            source_zone_id: "zone_lidar_protective".to_string(),
            inhibited_action: "robot.move".to_string(),
            requires_manual_reset: true,
        },
    ]
}

fn build_robot_cell_entity(
    id: &str,
    name: &str,
) -> Result<(EntityRecord, RobotCellEntitySummary), String> {
    let targets = sample_robot_targets();
    let validation = validate_sequence(&targets).map_err(|error| error.to_string())?;
    let safety_zones = sample_safety_zones();
    let safety_interlocks = sample_safety_interlocks();
    let safety_zone_count = safety_zones.len();
    let summary = RobotCellEntitySummary {
        target_count: validation.target_count,
        path_length_mm: validation.path_length_mm,
        max_segment_mm: validation.max_segment_mm,
        estimated_cycle_time_ms: validation.estimated_cycle_time_ms,
        safety_zone_count,
        warning_count: validation.warning_count,
    };
    let entity = sample_entity(
        "RobotCell",
        id,
        name,
        serde_json::json!({
            "controller": {
                "robotModel": "FAERO-X90",
                "tcpPayloadKg": 8.0
            },
            "targets": targets,
            "sequenceValidation": {
                "targetCount": summary.target_count,
                "pathLengthMm": summary.path_length_mm,
                "maxSegmentMm": summary.max_segment_mm,
                "estimatedCycleTimeMs": summary.estimated_cycle_time_ms,
                "warningCount": summary.warning_count
            },
            "safety": {
                "zoneCount": summary.safety_zone_count,
                "interlockCount": safety_interlocks.len(),
                "zones": safety_zones,
                "interlocks": safety_interlocks
            }
        }),
    );

    Ok((entity, summary))
}

fn build_simulation_run_entity(
    id: &str,
    name: &str,
    robot_cell: &EntityRecord,
    endpoint_count: usize,
) -> Result<(EntityRecord, SimulationRunEntitySummary), String> {
    let robot_summary = robot_cell_summary_from_entity(robot_cell)
        .ok_or_else(|| "robot cell summary missing".to_string())?;
    let request = SimulationRequest {
        scenario_name: robot_cell.name.clone(),
        seed: robot_summary.target_count as u64 * 97 + endpoint_count as u64 * 17,
        step_count: (robot_summary.target_count.max(3) as u32) * 4,
        planned_cycle_time_ms: robot_summary.estimated_cycle_time_ms,
        path_length_mm: robot_summary.path_length_mm,
        endpoint_count: endpoint_count as u32,
        safety_zone_count: robot_summary.safety_zone_count as u32,
    };
    let run = run_simulation(&request).map_err(|error| error.to_string())?;
    let summary = SimulationRunEntitySummary {
        status: simulation_status_label(&run.status).to_string(),
        collision_count: run.collision_count,
        cycle_time_ms: run.cycle_time_ms,
        max_tracking_error_mm: run.max_tracking_error_mm,
        energy_estimate_j: run.energy_estimate_j,
        timeline_sample_count: run.timeline_samples.len(),
    };
    let entity = sample_entity(
        "SimulationRun",
        id,
        name,
        serde_json::json!({
            "scenario": {
                "name": request.scenario_name,
                "seed": request.seed,
                "stepCount": request.step_count,
                "endpointCount": request.endpoint_count,
                "safetyZoneCount": request.safety_zone_count
            },
            "summary": {
                "status": summary.status.clone(),
                "collisionCount": summary.collision_count,
                "cycleTimeMs": summary.cycle_time_ms,
                "maxTrackingErrorMm": summary.max_tracking_error_mm,
                "energyEstimateJ": summary.energy_estimate_j,
                "timelineSampleCount": summary.timeline_sample_count
            },
            "timelineSamples": run.timeline_samples,
        }),
    );

    Ok((entity, summary))
}

fn build_safety_report_entity(
    id: &str,
    name: &str,
    robot_cell: &EntityRecord,
    attempted_action: &str,
) -> Result<(EntityRecord, SafetyReportEntitySummary), String> {
    let safety = robot_cell
        .data
        .get("safety")
        .ok_or_else(|| "robot cell safety configuration missing".to_string())?;
    let zones = serde_json::from_value::<Vec<SafetyZone>>(
        safety
            .get("zones")
            .cloned()
            .ok_or_else(|| "robot cell safety zones missing".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let interlocks = serde_json::from_value::<Vec<SafetyInterlock>>(
        safety
            .get("interlocks")
            .cloned()
            .ok_or_else(|| "robot cell safety interlocks missing".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let evaluation = evaluate_safety(&zones, &interlocks, attempted_action);
    let summary = SafetyReportEntitySummary {
        status: safety_status_label(&evaluation.status).to_string(),
        inhibited: evaluation.inhibited,
        active_zone_count: evaluation.active_zone_count,
        blocking_interlock_count: evaluation.blocking_interlock_count,
        advisory_zone_count: evaluation.advisory_zone_count,
    };
    let entity = sample_entity(
        "SafetyReport",
        id,
        name,
        serde_json::json!({
            "attemptedAction": attempted_action,
            "zones": zones,
            "interlocks": interlocks,
            "summary": {
                "status": summary.status.clone(),
                "inhibited": summary.inhibited,
                "activeZoneCount": summary.active_zone_count,
                "blockingInterlockCount": summary.blocking_interlock_count,
                "advisoryZoneCount": summary.advisory_zone_count,
                "causeZoneIds": evaluation.cause_zone_ids
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

fn assembly_solve_status_label(status: AssemblySolveStatus) -> &'static str {
    match status {
        AssemblySolveStatus::Solved => "solved",
        AssemblySolveStatus::Conflicting => "conflicting",
    }
}

fn assembly_summary_from_entity(entity: &EntityRecord) -> Option<AssemblyEntitySummary> {
    if entity.entity_type != "Assembly" {
        return None;
    }

    let summary = entity.data.get("solveReport")?;
    Some(AssemblyEntitySummary {
        status: string_from_value(summary, "status")?,
        occurrence_count: summary.get("occurrenceCount")?.as_u64()? as usize,
        mate_count: summary.get("mateCount")?.as_u64()? as usize,
        degrees_of_freedom_estimate: summary.get("degreesOfFreedomEstimate")?.as_u64()? as usize,
        warning_count: summary.get("warningCount")?.as_u64()? as usize,
    })
}

fn robot_cell_summary_from_entity(entity: &EntityRecord) -> Option<RobotCellEntitySummary> {
    if entity.entity_type != "RobotCell" {
        return None;
    }

    let validation = entity.data.get("sequenceValidation")?;
    let safety = entity.data.get("safety")?;
    Some(RobotCellEntitySummary {
        target_count: validation.get("targetCount")?.as_u64()? as usize,
        path_length_mm: number_from_value(validation, "pathLengthMm")?,
        max_segment_mm: number_from_value(validation, "maxSegmentMm")?,
        estimated_cycle_time_ms: validation.get("estimatedCycleTimeMs")?.as_u64()? as u32,
        safety_zone_count: safety.get("zoneCount")?.as_u64()? as usize,
        warning_count: validation.get("warningCount")?.as_u64()? as usize,
    })
}

fn simulation_status_label(status: &SimulationStatus) -> &'static str {
    match status {
        SimulationStatus::Completed => "completed",
        SimulationStatus::Warning => "warning",
        SimulationStatus::Collided => "collided",
    }
}

fn safety_status_label(status: &SafetyStatus) -> &'static str {
    match status {
        SafetyStatus::Clear => "clear",
        SafetyStatus::Warning => "warning",
        SafetyStatus::Blocked => "blocked",
    }
}

fn simulation_run_summary_from_entity(entity: &EntityRecord) -> Option<SimulationRunEntitySummary> {
    if entity.entity_type != "SimulationRun" {
        return None;
    }

    let summary = entity.data.get("summary")?;
    Some(SimulationRunEntitySummary {
        status: string_from_value(summary, "status")?,
        collision_count: summary.get("collisionCount")?.as_u64()? as u32,
        cycle_time_ms: summary.get("cycleTimeMs")?.as_u64()? as u32,
        max_tracking_error_mm: number_from_value(summary, "maxTrackingErrorMm")?,
        energy_estimate_j: number_from_value(summary, "energyEstimateJ")?,
        timeline_sample_count: summary.get("timelineSampleCount")?.as_u64()? as usize,
    })
}

fn safety_report_summary_from_entity(entity: &EntityRecord) -> Option<SafetyReportEntitySummary> {
    if entity.entity_type != "SafetyReport" {
        return None;
    }

    let summary = entity.data.get("summary")?;
    Some(SafetyReportEntitySummary {
        status: string_from_value(summary, "status")?,
        inhibited: summary.get("inhibited")?.as_bool()?,
        active_zone_count: summary.get("activeZoneCount")?.as_u64()? as usize,
        blocking_interlock_count: summary.get("blockingInterlockCount")?.as_u64()? as usize,
        advisory_zone_count: summary.get("advisoryZoneCount")?.as_u64()? as usize,
    })
}

fn format_part_entity_detail(summary: &PartGeometrySummary) -> String {
    format!(
        "{:.1} x {:.1} x {:.1} mm | {:.1} g",
        summary.width_mm, summary.height_mm, summary.depth_mm, summary.estimated_mass_grams
    )
}

fn format_assembly_entity_detail(summary: &AssemblyEntitySummary) -> String {
    format!(
        "{} | {} occ | {} mates | {} ddl",
        summary.status,
        summary.occurrence_count,
        summary.mate_count,
        summary.degrees_of_freedom_estimate
    )
}

fn format_robot_cell_entity_detail(summary: &RobotCellEntitySummary) -> String {
    format!(
        "{} pts | {:.0} mm | {} ms",
        summary.target_count, summary.path_length_mm, summary.estimated_cycle_time_ms
    )
}

fn format_simulation_run_entity_detail(summary: &SimulationRunEntitySummary) -> String {
    format!(
        "{} | {} ms | {} coll",
        summary.status, summary.cycle_time_ms, summary.collision_count
    )
}

fn format_safety_report_entity_detail(summary: &SafetyReportEntitySummary) -> String {
    format!(
        "{} | {} active | {} block",
        summary.status, summary.active_zone_count, summary.blocking_interlock_count
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
fn workspace_regenerate_latest_part(
    payload: PartRegenerationInput,
    state: State<'_, SharedWorkspace>,
) -> Result<CommandResponse, String> {
    let mut session = lock_workspace(&state)?;
    let result =
        session.regenerate_latest_part(payload.width_mm, payload.height_mm, payload.depth_mm)?;
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
            workspace_regenerate_latest_part,
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
    fn regenerate_latest_part_updates_existing_geometry_metrics() {
        let mut session = WorkspaceSession::empty("Session");
        session
            .execute_command("entity.create.part")
            .expect("part should be created");

        let result = session
            .regenerate_latest_part(200.0, 90.0, 20.0)
            .expect("part should regenerate");
        let snapshot = session.snapshot();
        let geometry = snapshot.entities[0]
            .part_geometry
            .as_ref()
            .expect("updated geometry should exist");

        assert_eq!(result.command_id, "build.regenerate_part");
        assert_eq!(geometry.width_mm, 200.0);
        assert_eq!(geometry.height_mm, 90.0);
        assert_eq!(geometry.depth_mm, 20.0);
        assert!(result.message.contains("200.0 x 90.0 x 20.0"));
    }

    #[test]
    fn created_assembly_exposes_occurrence_and_mate_summary() {
        let mut session = WorkspaceSession::empty("Session");

        let result = session
            .execute_command("entity.create.assembly")
            .expect("assembly should be created");
        let snapshot = session.snapshot();
        let assembly = snapshot
            .entities
            .iter()
            .find(|entity| entity.entity_type == "Assembly")
            .expect("assembly summary should exist");
        let summary = assembly
            .assembly_summary
            .as_ref()
            .expect("assembly details should exist");

        assert_eq!(result.status, "applied");
        assert_eq!(summary.status, "solved");
        assert!(summary.occurrence_count >= 2);
        assert!(summary.mate_count >= 1);
        assert!(
            assembly
                .detail
                .as_ref()
                .is_some_and(|detail| detail.contains("occ"))
        );
    }

    #[test]
    fn created_robot_cell_exposes_path_and_cycle_summary() {
        let mut session = WorkspaceSession::empty("Session");

        let result = session
            .execute_command("entity.create.robot_cell")
            .expect("robot cell should be created");
        let snapshot = session.snapshot();
        let robot_cell = snapshot
            .entities
            .iter()
            .find(|entity| entity.entity_type == "RobotCell")
            .expect("robot cell summary should exist");
        let summary = robot_cell
            .robot_cell_summary
            .as_ref()
            .expect("robot cell details should exist");

        assert_eq!(result.status, "applied");
        assert_eq!(summary.target_count, 3);
        assert!(summary.path_length_mm > 850.0);
        assert!(summary.estimated_cycle_time_ms > 3_000);
        assert!(
            robot_cell
                .detail
                .as_ref()
                .is_some_and(|detail| detail.contains("pts"))
        );
    }

    #[test]
    fn simulation_run_creates_a_summary_entity_and_activity() {
        let mut session = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
            .expect("fixture-backed session should load");

        let result = session
            .execute_command("simulation.run.start")
            .expect("simulation command should run");
        let snapshot = session.snapshot();
        let simulation = snapshot
            .entities
            .iter()
            .find(|entity| entity.entity_type == "SimulationRun")
            .expect("simulation run should be created");
        let summary = simulation
            .simulation_run_summary
            .as_ref()
            .expect("simulation summary should exist");

        assert_eq!(result.status, "applied");
        assert_eq!(summary.status, "completed");
        assert_eq!(summary.collision_count, 0);
        assert!(summary.cycle_time_ms > 3_000);
        assert!(summary.timeline_sample_count >= 10);
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| entry.kind == "simulation.run.completed")
        );
    }

    #[test]
    fn safety_analysis_creates_a_report_entity_and_activity() {
        let mut session = WorkspaceSession::empty("Session");

        let result = session
            .execute_command("analyze.safety")
            .expect("safety command should run");
        let snapshot = session.snapshot();
        let report = snapshot
            .entities
            .iter()
            .find(|entity| entity.entity_type == "SafetyReport")
            .expect("safety report should be created");
        let summary = report
            .safety_report_summary
            .as_ref()
            .expect("safety summary should exist");

        assert_eq!(result.status, "applied");
        assert_eq!(summary.status, "warning");
        assert!(!summary.inhibited);
        assert_eq!(summary.active_zone_count, 1);
        assert_eq!(summary.advisory_zone_count, 1);
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| entry.kind == "analysis.safety.generated")
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
