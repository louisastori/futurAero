use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use faero_ai::{
    AiChatResponse, AiConversationMessage, AiRuntimeStatus, chat_with_project,
    query_runtime_status_with_profile as query_ai_runtime_status,
};
use faero_commissioning::{
    AsBuiltMeasurement, CommissioningCapture, compare_as_built, start_commissioning_session,
};
use faero_core::{CoreCommand, ProjectGraph};
use faero_geometry::{
    ExtrusionDefinition, MaterialProfile, SketchConstraintState, rectangular_profile,
    regenerate_extrusion,
};
use faero_integration::{
    IntegrationStubRegistry, degraded_wireless_profile, stub_bluetooth_endpoint,
    stub_opcua_endpoint, stub_plc_endpoint, stub_robot_controller_endpoint, stub_ros2_endpoint,
    stub_wifi_endpoint,
};
use faero_optimization::{run_study, seeded_study};
use faero_perception::{
    NominalSceneTarget, PointCloudFrame, calibrate_rig, run_perception, seeded_sensor_rig,
};
use faero_plugin_host::validate_manifest;
use faero_robotics::{
    CartesianPose, EquipmentModel, EquipmentParameterSet, EquipmentType, RobotCellControlModel,
    RobotCellControlSummary, RobotCellModel, RobotModel, RobotPayloadLimits, RobotSequenceModel,
    RobotTarget, RobotTargetModel, RobotToolMountRef, RobotWorkspaceBounds,
    summarize_robot_cell_control, validate_robot_cell_control, validate_robot_cell_structure,
    validate_sequence, validate_target_models,
};
use faero_safety::{SafetyInterlock, SafetyStatus, SafetyZone, SafetyZoneKind, evaluate_safety};
use faero_sim::{SimulationRequest, SimulationStatus, run_simulation};
use faero_types::{
    AiSessionLog, AssemblyData, AssemblyJoint, AssemblyJointAxis, AssemblyJointLimits,
    AssemblyJointType, AssemblyMateConstraint, AssemblyMateType, AssemblyOccurrence,
    AssemblySolveStatus, AssemblyTransform, ControlTransition, ControllerState,
    ControllerStateMachine, EntityRecord, ExternalEndpoint, NetworkCaptureDataset,
    PluginContribution, PluginManifest, ProjectDocument, QosProfile, ScheduledSignalChange,
    SignalAssignment, SignalComparator, SignalCondition, SignalDefinition, SignalKind, SignalValue,
    SimulationContactPair, StreamDirection, TelemetryStream, TimingProfile,
};
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Runtime, State,
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
};

const DEFAULT_FIXTURE_ID: &str = "pick-and-place-demo.faero";
const UNTITLED_SESSION_ID: &str = "session:untitled";
const NATIVE_MENU_EVENT_NAME: &str = "futureaero:menu-command";

type SharedWorkspace = Mutex<WorkspaceSession>;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct NativeMenuCommandPayload {
    command_id: String,
}

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
    data: serde_json::Value,
    part_geometry: Option<PartGeometrySummary>,
    assembly_summary: Option<AssemblyEntitySummary>,
    robot_cell_summary: Option<RobotCellEntitySummary>,
    sensor_rig_summary: Option<SensorRigEntitySummary>,
    simulation_run_summary: Option<SimulationRunEntitySummary>,
    safety_report_summary: Option<SafetyReportEntitySummary>,
    perception_run_summary: Option<PerceptionRunEntitySummary>,
    commissioning_session_summary: Option<CommissioningSessionEntitySummary>,
    as_built_comparison_summary: Option<AsBuiltComparisonEntitySummary>,
    optimization_study_summary: Option<OptimizationStudyEntitySummary>,
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
    joint_count: usize,
    joint_state_summary: Option<String>,
    degrees_of_freedom_estimate: usize,
    warning_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RobotCellEntitySummary {
    scene_assembly_id: Option<String>,
    target_preview: Option<String>,
    target_count: usize,
    path_length_mm: f64,
    max_segment_mm: f64,
    estimated_cycle_time_ms: u32,
    equipment_count: usize,
    sequence_count: usize,
    safety_zone_count: usize,
    signal_count: usize,
    controller_transition_count: usize,
    blocked_sequence_detected: bool,
    blocked_state_id: Option<String>,
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
    blocked_sequence_detected: bool,
    blocked_state_id: Option<String>,
    contact_count: usize,
    signal_sample_count: usize,
    controller_state_sample_count: usize,
    timeline_sample_count: usize,
    job_status: String,
    job_phase: String,
    job_progress: f64,
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
struct SensorRigEntitySummary {
    sensor_count: usize,
    lidar_count: usize,
    sample_rate_hz: f64,
    calibration_status: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PerceptionRunEntitySummary {
    status: String,
    frame_count: usize,
    average_coverage_ratio: f64,
    unknown_obstacle_count: u32,
    deviation_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CommissioningSessionEntitySummary {
    status: String,
    progress_ratio: f64,
    capture_count: usize,
    adjustment_count: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AsBuiltComparisonEntitySummary {
    accepted_count: usize,
    rejected_count: usize,
    average_deviation_mm: f64,
    max_deviation_mm: f64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OptimizationStudyEntitySummary {
    candidate_count: usize,
    objective_count: usize,
    best_candidate_id: Option<String>,
    best_score: f64,
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OpenSpecDocumentSummary {
    id: String,
    title: String,
    kind: String,
    status: String,
    linked_entity_count: usize,
    linked_external_count: usize,
    tag_count: usize,
    excerpt: String,
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
    open_spec_documents: Vec<OpenSpecDocumentSummary>,
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

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntityPropertyUpdateInput {
    entity_id: String,
    changes: serde_json::Value,
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

    fn latest_entity_of_type(&self, entity_type: &str) -> Option<EntityRecord> {
        self.graph
            .document()
            .nodes
            .values()
            .filter(|entity| entity.entity_type == entity_type)
            .max_by(|left, right| left.id.cmp(&right.id))
            .cloned()
    }

    fn ensure_robot_cell_control_entities(
        &mut self,
        robot_cell: &EntityRecord,
    ) -> Result<(Vec<EntityRecord>, EntityRecord), String> {
        for entity in build_robot_cell_support_entities(robot_cell)? {
            if self.graph.document().nodes.contains_key(&entity.id) {
                continue;
            }
            self.graph
                .apply_command(CoreCommand::CreateEntity(entity))
                .map_err(|error| error.to_string())?;
        }
        self.materialize_robot_cell_scene_layout(robot_cell)?;
        self.sync_robot_cell_control_dependents(&robot_cell.id)?;

        let signal_entities = self
            .graph
            .document()
            .nodes
            .values()
            .filter(|entity| entity.entity_type == "Signal")
            .filter(|entity| {
                entity.data.get("cellId").and_then(|value| value.as_str())
                    == Some(robot_cell.id.as_str())
            })
            .cloned()
            .collect::<Vec<_>>();
        let controller_entity = self
            .graph
            .document()
            .nodes
            .values()
            .find(|entity| entity.entity_type == "ControllerModel")
            .filter(|entity| {
                entity.data.get("cellId").and_then(|value| value.as_str())
                    == Some(robot_cell.id.as_str())
            })
            .cloned()
            .ok_or_else(|| "controller entity missing for robot cell".to_string())?;

        Ok((signal_entities, controller_entity))
    }

    fn materialize_robot_cell_scene_layout(
        &mut self,
        robot_cell: &EntityRecord,
    ) -> Result<(), String> {
        let fallback_scene_assembly_id = robot_cell_scene_assembly_entity_id(&robot_cell.id);
        let scene_assembly_id = robot_cell
            .data
            .get("sceneAssemblyId")
            .and_then(|value| value.as_str())
            .unwrap_or(fallback_scene_assembly_id.as_str())
            .to_string();
        let assembly_entity = self
            .graph
            .document()
            .nodes
            .get(&scene_assembly_id)
            .cloned()
            .ok_or_else(|| "scene assembly missing for robot cell".to_string())?;
        let assembly = serde_json::from_value::<AssemblyData>(assembly_entity.data.clone())
            .map_err(|error| error.to_string())?;
        if !assembly.occurrences.is_empty() {
            return Ok(());
        }

        for occurrence in robot_cell_scene_occurrences(&robot_cell.id) {
            self.graph
                .apply_command(CoreCommand::AddAssemblyOccurrence {
                    assembly_id: scene_assembly_id.clone(),
                    occurrence,
                })
                .map_err(|error| error.to_string())?;
        }
        for mate in robot_cell_scene_mates() {
            self.graph
                .apply_command(CoreCommand::AddAssemblyMate {
                    assembly_id: scene_assembly_id.clone(),
                    mate,
                })
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    fn update_entity_properties(
        &mut self,
        payload: &EntityPropertyUpdateInput,
    ) -> Result<CommandResult, String> {
        let entity = self
            .graph
            .document()
            .nodes
            .get(&payload.entity_id)
            .cloned()
            .ok_or_else(|| format!("entite introuvable: {}", payload.entity_id))?;
        let changes = payload
            .changes
            .as_object()
            .cloned()
            .ok_or_else(|| "les changements doivent former un objet JSON".to_string())?;
        let next_name = changes
            .get("name")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| entity.name.clone());
        if next_name.trim().is_empty() {
            return Err("le nom ne peut pas etre vide".to_string());
        }

        let current_tags = entity
            .data
            .get("tags")
            .and_then(|value| value.as_array())
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let normalized_tags = if let Some(tags_value) = changes.get("tags") {
            tags_value
                .as_array()
                .ok_or_else(|| "tags doit rester un tableau".to_string())?
                .iter()
                .filter_map(|value| value.as_str().map(str::trim))
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        } else {
            current_tags
        };

        if entity.entity_type == "Part" {
            let current = part_geometry_summary_from_entity(&entity)
                .ok_or_else(|| "piece parametrique invalide".to_string())?;
            let width_mm =
                extract_f64_change(&changes, "parameterSet.widthMm").unwrap_or(current.width_mm);
            let height_mm =
                extract_f64_change(&changes, "parameterSet.heightMm").unwrap_or(current.height_mm);
            let depth_mm =
                extract_f64_change(&changes, "parameterSet.depthMm").unwrap_or(current.depth_mm);
            validate_positive_dimension("parameterSet.widthMm", width_mm)?;
            validate_positive_dimension("parameterSet.heightMm", height_mm)?;
            validate_positive_dimension("parameterSet.depthMm", depth_mm)?;
            let (mut next_entity, _) = build_parametric_part_entity(
                &entity.id, &next_name, width_mm, height_mm, depth_mm,
            )?;
            if let Some(data) = next_entity.data.as_object_mut() {
                data.insert("tags".to_string(), serde_json::json!(normalized_tags));
                merge_generic_parameter_changes(data, &entity.data, &changes)?;
                validate_entity_change_set(&entity, data)?;
            }
            self.graph
                .apply_command(CoreCommand::ReplaceEntity(next_entity))
                .map_err(|error| error.to_string())?;
        } else {
            let mut next_entity = entity.clone();
            next_entity.name = next_name;
            if !next_entity.data.is_object() {
                next_entity.data = serde_json::json!({});
            }
            if let Some(data) = next_entity.data.as_object_mut() {
                data.insert("tags".to_string(), serde_json::json!(normalized_tags));
                apply_data_changes(data, &changes)?;
                if entity.entity_type == "Signal"
                    && let Some(kind) = data.get("kind").and_then(|value| value.as_str())
                {
                    let current_value_valid = match kind {
                        "boolean" => data
                            .get("currentValue")
                            .is_some_and(|value| value.is_boolean()),
                        "scalar" => data
                            .get("currentValue")
                            .and_then(|value| value.as_f64())
                            .is_some(),
                        "text" => data
                            .get("currentValue")
                            .and_then(|value| value.as_str())
                            .is_some(),
                        _ => false,
                    };
                    if !current_value_valid {
                        data.insert(
                            "currentValue".to_string(),
                            default_signal_value_for_kind(kind),
                        );
                    }
                }
                if entity.entity_type == "RobotTarget" {
                    normalize_robot_target_entity_data(&entity.id, data)?;
                }
                validate_entity_change_set(&entity, data)?;
            }
            self.preview_robot_cell_control_update(&next_entity)?;
            self.graph
                .apply_command(CoreCommand::ReplaceEntity(next_entity.clone()))
                .map_err(|error| error.to_string())?;

            if entity.entity_type == "Signal"
                && let Some(current_value) = next_entity.data.get("currentValue")
            {
                let parsed_value = serde_json::from_value::<SignalValue>(current_value.clone())
                    .map_err(|error| error.to_string())?;
                self.graph
                    .apply_command(CoreCommand::SetSignalValue {
                        entity_id: entity.id.clone(),
                        value: parsed_value,
                    })
                    .map_err(|error| error.to_string())?;
                if let Some(cell_id) = next_entity
                    .data
                    .get("cellId")
                    .and_then(|value| value.as_str())
                {
                    self.sync_robot_cell_control_dependents(cell_id)?;
                }
            }
            if entity.entity_type == "ControllerModel"
                && let Some(cell_id) = next_entity
                    .data
                    .get("cellId")
                    .and_then(|value| value.as_str())
            {
                self.sync_robot_cell_control_dependents(cell_id)?;
            }
            if entity.entity_type == "RobotTarget" {
                self.sync_robot_target_dependents(&next_entity)?;
            }
        }

        self.push_system_activity("entity.properties.updated", Some(entity.id.clone()));
        Ok(CommandResult {
            command_id: "entity.properties.update".to_string(),
            status: "applied".to_string(),
            message: format!("proprietes mises a jour pour {}", entity.id),
        })
    }

    fn record_ai_response(
        &mut self,
        message: &str,
        response: &AiChatResponse,
    ) -> Result<String, String> {
        let suggestion_id = format!("ent_ai_suggestion_{:03}", self.graph.entity_count() + 1);
        let session_entity = self.latest_entity_of_type("AiSession");
        let (session_id, mut created_suggestion_ids, accepted_suggestion_ids, session_entity_id) =
            if let Some(entity) = session_entity {
                let session_id = entity
                    .data
                    .get("sessionId")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| "ai_session_001".to_string());
                let created = entity
                    .data
                    .get("createdSuggestionIds")
                    .and_then(|value| value.as_array())
                    .map(|entries| {
                        entries
                            .iter()
                            .filter_map(|value| value.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let accepted = entity
                    .data
                    .get("acceptedSuggestionIds")
                    .and_then(|value| value.as_array())
                    .map(|entries| {
                        entries
                            .iter()
                            .filter_map(|value| value.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                (session_id, created, accepted, entity.id)
            } else {
                (
                    "ai_session_001".to_string(),
                    Vec::new(),
                    Vec::new(),
                    format!("ent_ai_session_{:03}", self.graph.entity_count() + 2),
                )
            };
        created_suggestion_ids.push(suggestion_id.clone());

        let structured = response
            .structured
            .clone()
            .ok_or_else(|| "structured AI response missing".to_string())?;
        let session_log = AiSessionLog {
            session_id: session_id.clone(),
            user_intent: message.to_string(),
            mode: "explain".to_string(),
            runtime_profile: response.runtime.active_profile.clone(),
            model_info: format!(
                "{}:{}",
                response.runtime.provider,
                response
                    .runtime
                    .active_model
                    .clone()
                    .unwrap_or_else(|| response.runtime.mode.clone())
            ),
            critic_model_info: Some(format!(
                "{}:{}:critic",
                response.runtime.provider, response.runtime.active_profile
            )),
            critique_pass_count: structured.critique_passes.len(),
            context_refs: structured.context_refs.clone(),
            prompt_hash: stable_hash(message),
            response_hash: stable_hash(&response.answer),
            created_suggestion_ids: created_suggestion_ids.clone(),
            accepted_suggestion_ids,
        };
        let session_record = sample_entity(
            "AiSession",
            &session_entity_id,
            "AI Session",
            serde_json::json!({
                "sessionId": session_log.session_id,
                "userIntent": session_log.user_intent,
                "mode": session_log.mode,
                "runtimeProfile": session_log.runtime_profile,
                "modelInfo": session_log.model_info,
                "criticModelInfo": session_log.critic_model_info,
                "critiquePassCount": session_log.critique_pass_count,
                "contextRefs": session_log.context_refs,
                "promptHash": session_log.prompt_hash,
                "responseHash": session_log.response_hash,
                "createdSuggestionIds": session_log.created_suggestion_ids,
                "acceptedSuggestionIds": session_log.accepted_suggestion_ids,
                "tags": ["ai", "journal"],
                "parameterSet": {
                    "createdSuggestionCount": created_suggestion_ids.len(),
                    "runtimeAvailable": response.runtime.available,
                    "critiquePassCount": structured.critique_passes.len()
                }
            }),
        );
        if self.graph.document().nodes.contains_key(&session_entity_id) {
            self.graph
                .apply_command(CoreCommand::ReplaceEntity(session_record))
                .map_err(|error| error.to_string())?;
        } else {
            self.graph
                .apply_command(CoreCommand::CreateEntity(session_record))
                .map_err(|error| error.to_string())?;
        }
        let risk_level = match &structured.risk_level {
            faero_types::AiRiskLevel::Low => "low",
            faero_types::AiRiskLevel::Medium => "medium",
            faero_types::AiRiskLevel::High => "high",
        };

        let suggestion_record = sample_entity(
            "AiSuggestion",
            &suggestion_id,
            "AI Suggestion",
            serde_json::json!({
                "sessionId": session_id,
                "source": response.source.clone(),
                "answer": response.answer.clone(),
                "runtime": response.runtime.clone(),
                "references": response.references.clone(),
                "summary": structured.summary.clone(),
                "runtimeProfile": structured.runtime_profile.clone(),
                "contextRefs": structured.context_refs.clone(),
                "confidence": structured.confidence,
                "riskLevel": risk_level,
                "reviewStatus": "pending",
                "limitations": structured.limitations.clone(),
                "critiquePasses": structured.critique_passes.clone(),
                "proposedCommands": structured.proposed_commands.clone(),
                "explanation": structured.explanation.clone(),
                "tags": ["ai", "explain", "local", response.runtime.active_profile.clone()],
                "parameterSet": {
                    "confidence": structured.confidence,
                    "referenceCount": response.references.len(),
                    "proposedCommandCount": structured.proposed_commands.len(),
                    "critiquePassCount": structured.critique_passes.len()
                }
            }),
        );
        self.graph
            .apply_command(CoreCommand::CreateEntity(suggestion_record))
            .map_err(|error| error.to_string())?;
        self.push_system_activity("ai.suggestion.created", Some(suggestion_id.clone()));
        Ok(suggestion_id)
    }

    fn apply_ai_suggestion(&mut self, suggestion_id: &str) -> Result<CommandResult, String> {
        let suggestion = self
            .graph
            .document()
            .nodes
            .get(suggestion_id)
            .cloned()
            .ok_or_else(|| format!("suggestion IA introuvable: {suggestion_id}"))?;
        if suggestion.entity_type != "AiSuggestion" {
            return Err(format!(
                "entite {suggestion_id} n est pas une suggestion IA"
            ));
        }
        let proposed_commands = suggestion
            .data
            .get("proposedCommands")
            .cloned()
            .ok_or_else(|| "aucune commande proposee".to_string())
            .and_then(|value| {
                serde_json::from_value::<Vec<faero_types::AiProposedCommand>>(value)
                    .map_err(|error| error.to_string())
            })?;
        if proposed_commands.is_empty() {
            return Err("aucune commande proposee".to_string());
        }

        let mut applied_kinds = Vec::new();
        let mut applied_messages = Vec::new();
        for command in proposed_commands {
            match command.kind.as_str() {
                "entity.properties.update" => {
                    let target_id = command
                        .target_id
                        .clone()
                        .ok_or_else(|| "entity.properties.update exige targetId".to_string())?;
                    let changes = command
                        .payload
                        .get("changes")
                        .cloned()
                        .unwrap_or_else(|| command.payload.clone());
                    let result = self.update_entity_properties(&EntityPropertyUpdateInput {
                        entity_id: target_id,
                        changes,
                    })?;
                    applied_kinds.push(command.kind.clone());
                    applied_messages.push(result.message);
                }
                other => {
                    let result = self.execute_command(other)?;
                    applied_kinds.push(other.to_string());
                    applied_messages.push(result.message);
                }
            }
        }

        self.update_ai_suggestion_review_state(suggestion_id, "accepted", &applied_kinds)?;
        self.push_system_activity("ai.suggestion.applied", Some(suggestion_id.to_string()));

        Ok(CommandResult {
            command_id: "ai.suggestion.apply".to_string(),
            status: "applied".to_string(),
            message: format!(
                "suggestion {} appliquee: {}",
                suggestion_id,
                applied_messages.join(" | ")
            ),
        })
    }

    fn reject_ai_suggestion(&mut self, suggestion_id: &str) -> Result<CommandResult, String> {
        self.update_ai_suggestion_review_state(suggestion_id, "rejected", &[])?;
        self.push_system_activity("ai.suggestion.rejected", Some(suggestion_id.to_string()));
        Ok(CommandResult {
            command_id: "ai.suggestion.reject".to_string(),
            status: "applied".to_string(),
            message: format!("suggestion {} rejetee sans effet de bord", suggestion_id),
        })
    }

    fn update_ai_suggestion_review_state(
        &mut self,
        suggestion_id: &str,
        review_status: &str,
        applied_command_kinds: &[String],
    ) -> Result<(), String> {
        let suggestion = self
            .graph
            .document()
            .nodes
            .get(suggestion_id)
            .cloned()
            .ok_or_else(|| format!("suggestion IA introuvable: {suggestion_id}"))?;
        let mut updated_suggestion = suggestion.clone();
        if let Some(data) = updated_suggestion.data.as_object_mut() {
            data.insert(
                "reviewStatus".to_string(),
                serde_json::Value::String(review_status.to_string()),
            );
            data.insert(
                "appliedCommandKinds".to_string(),
                serde_json::json!(applied_command_kinds),
            );
            if let Some(parameter_set) = data
                .entry("parameterSet")
                .or_insert_with(|| serde_json::json!({}))
                .as_object_mut()
            {
                parameter_set.insert(
                    "appliedCommandCount".to_string(),
                    serde_json::json!(applied_command_kinds.len()),
                );
            }
        }
        self.graph
            .apply_command(CoreCommand::ReplaceEntity(updated_suggestion))
            .map_err(|error| error.to_string())?;

        let session_id = suggestion
            .data
            .get("sessionId")
            .and_then(|value| value.as_str())
            .map(str::to_string);
        if review_status == "accepted"
            && let Some(session_id) = session_id
        {
            self.accept_suggestion_in_session(&session_id, suggestion_id)?;
        }

        Ok(())
    }

    fn accept_suggestion_in_session(
        &mut self,
        session_id: &str,
        suggestion_id: &str,
    ) -> Result<(), String> {
        let Some(session_entity) = self
            .graph
            .document()
            .nodes
            .values()
            .find(|entity| entity.entity_type == "AiSession")
            .filter(|entity| {
                entity
                    .data
                    .get("sessionId")
                    .and_then(|value| value.as_str())
                    == Some(session_id)
            })
            .cloned()
        else {
            return Ok(());
        };

        let mut updated = session_entity.clone();
        if let Some(data) = updated.data.as_object_mut() {
            let mut accepted_ids = data
                .get("acceptedSuggestionIds")
                .and_then(|value| value.as_array())
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|value| value.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if !accepted_ids.iter().any(|entry| entry == suggestion_id) {
                accepted_ids.push(suggestion_id.to_string());
            }
            data.insert(
                "acceptedSuggestionIds".to_string(),
                serde_json::json!(accepted_ids),
            );
        }
        self.graph
            .apply_command(CoreCommand::ReplaceEntity(updated))
            .map_err(|error| error.to_string())?;
        Ok(())
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
                let assembly_id = format!("ent_asm_{index:03}");
                let entity =
                    build_empty_assembly_entity(&assembly_id, &format!("Assembly-{index:03}"))?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                let assembly_part_ids = part_ids[..part_ids.len().min(3)].to_vec();
                for (occurrence_index, part_id) in assembly_part_ids.iter().enumerate() {
                    self.graph
                        .apply_command(CoreCommand::AddAssemblyOccurrence {
                            assembly_id: assembly_id.clone(),
                            occurrence: sample_assembly_occurrence(part_id, occurrence_index),
                        })
                        .map_err(|error| error.to_string())?;
                }
                for (mate_index, pair) in
                    (0..assembly_part_ids.len().saturating_sub(1)).map(|index| {
                        (
                            index,
                            (&assembly_part_ids[index], &assembly_part_ids[index + 1]),
                        )
                    })
                {
                    let _ = pair;
                    self.graph
                        .apply_command(CoreCommand::AddAssemblyMate {
                            assembly_id: assembly_id.clone(),
                            mate: sample_assembly_mate(mate_index),
                        })
                        .map_err(|error| error.to_string())?;
                }
                if assembly_part_ids.len() >= 2 {
                    self.graph
                        .apply_command(CoreCommand::CreateAssemblyJoint {
                            assembly_id: assembly_id.clone(),
                            joint: sample_assembly_joint(0),
                        })
                        .map_err(|error| error.to_string())?;
                    self.graph
                        .apply_command(CoreCommand::SetAssemblyJointState {
                            assembly_id: assembly_id.clone(),
                            joint_id: "joint_001".to_string(),
                            current_position: 0.35,
                        })
                        .map_err(|error| error.to_string())?;
                }
                let assembly = self
                    .graph
                    .document()
                    .nodes
                    .get(&assembly_id)
                    .cloned()
                    .ok_or_else(|| "assembly entity missing after creation".to_string())?;
                let summary = assembly_summary_from_entity(&assembly)
                    .ok_or_else(|| "assembly summary missing after creation".to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "assemblage ajoute: {} occurrences | {} mates | {} joints | {}",
                        summary.occurrence_count,
                        summary.mate_count,
                        summary.joint_count,
                        summary.status
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
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.ensure_robot_cell_control_entities(&entity)?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "cellule robotique ajoutee: {} cibles | {} equipements | {} signaux | {} ms",
                        summary.target_count,
                        summary.equipment_count,
                        summary.signal_count,
                        summary.estimated_cycle_time_ms
                    ),
                })
            }
            "entity.create.sensor_rig" => {
                let index = self.graph.entity_count() + 1;
                let (entity, summary) = build_sensor_rig_entity(
                    &format!("ent_rig_{index:03}"),
                    &format!("SensorRig-{index:03}"),
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "rig capteurs ajoute: {} capteur(s) | {} lidar(s)",
                        summary.sensor_count, summary.lidar_count
                    ),
                })
            }
            "entity.create.external_endpoint" => {
                let endpoint_index = self.graph.endpoint_count() + 1;
                let (endpoint, stream) = sample_endpoint_with_stream(endpoint_index);
                self.graph
                    .apply_command(CoreCommand::RegisterEndpoint(endpoint))
                    .map_err(|error| error.to_string())?;
                self.graph
                    .apply_command(CoreCommand::RegisterStream(stream))
                    .map_err(|error| error.to_string())?;
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: "endpoint externe, binding et flux telemetrie ajoutes".to_string(),
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
                    self.ensure_robot_cell_control_entities(&entity)?;
                    entity
                };
                let (signal_entities, controller_entity) =
                    self.ensure_robot_cell_control_entities(&robot_cell)?;

                let run_index = self.graph.entity_count() + 1;
                let (entity, summary) = build_simulation_run_entity(
                    &format!("ent_run_{run_index:03}"),
                    &format!("SimulationRun-{run_index:03}"),
                    &robot_cell,
                    &signal_entities,
                    &controller_entity,
                    self.graph.endpoint_count(),
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                if let Some(progress_samples) = entity
                    .data
                    .get("job")
                    .and_then(|job| job.get("progressSamples"))
                    .and_then(|value| value.as_array())
                {
                    for sample in progress_samples {
                        if let Some(phase) = sample.get("phase").and_then(|value| value.as_str()) {
                            self.push_system_activity(
                                &format!("simulation.run.{phase}"),
                                Some(entity.id.clone()),
                            );
                        }
                    }
                }
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "run de simulation termine: {} | {} ms | {} collision(s) | {} signal(s)",
                        summary.status,
                        summary.cycle_time_ms,
                        summary.collision_count,
                        summary.signal_sample_count
                    ),
                })
            }
            "perception.run.start" => {
                let sensor_rig = if let Some(existing) = self
                    .graph
                    .document()
                    .nodes
                    .values()
                    .filter(|entity| sensor_rig_summary_from_entity(entity).is_some())
                    .max_by(|left, right| left.id.cmp(&right.id))
                    .cloned()
                {
                    existing
                } else {
                    let index = self.graph.entity_count() + 1;
                    let (entity, _) = build_sensor_rig_entity(
                        &format!("ent_rig_{index:03}"),
                        &format!("SensorRig-{index:03}"),
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(entity.clone()))
                        .map_err(|error| error.to_string())?;
                    entity
                };

                let run_index = self.graph.entity_count() + 1;
                let (entity, summary) = build_perception_run_entity(
                    &format!("ent_perc_{run_index:03}"),
                    &format!("PerceptionRun-{run_index:03}"),
                    &sensor_rig,
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.push_system_activity("perception.run.completed", Some(entity.id));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "run perception termine: {} | {} frame(s) | {} ecart(s)",
                        summary.status, summary.frame_count, summary.deviation_count
                    ),
                })
            }
            "integration.replay.degraded" => {
                let mut registry = IntegrationStubRegistry::seeded();
                let endpoint_id = self
                    .graph
                    .document()
                    .endpoints
                    .keys()
                    .find(|id| id.starts_with("ext_wifi") || id.starts_with("ext_ble"))
                    .cloned()
                    .unwrap_or_else(|| "ext_wifi_001".to_string());
                registry.register_trace(NetworkCaptureDataset {
                    id: format!("trace_{endpoint_id}"),
                    endpoint_id: endpoint_id.clone(),
                    capture_type: "pcap".to_string(),
                    timestamp_range: "2026-04-08T08:00:00Z/2026-04-08T08:00:12Z".to_string(),
                    asset_refs: vec![
                        format!("captures/{endpoint_id}/trace_001.pcap"),
                        format!("captures/{endpoint_id}/trace_001.sidecar.json"),
                    ],
                    link_metrics: registry
                        .simulate_link(&endpoint_id, None)
                        .unwrap_or_default(),
                    status: "ready".to_string(),
                });
                let report = registry
                    .replay_trace(
                        &format!("trace_{endpoint_id}"),
                        Some(&degraded_wireless_profile()),
                    )
                    .ok_or_else(|| "trace industrielle introuvable".to_string())?;
                let entity = sample_entity(
                    "IndustrialReplay",
                    &format!("ent_replay_{:03}", self.graph.entity_count() + 1),
                    "IndustrialReplay",
                    serde_json::json!({
                        "endpointId": endpoint_id,
                        "summary": {
                            "sampleCount": report.sample_count,
                            "degraded": report.degraded,
                            "latencyMs": report.effective_metrics.latency_ms,
                            "dropRate": report.effective_metrics.drop_rate
                        },
                        "tags": ["integration", "replay", "wireless"]
                    }),
                );
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.push_system_activity("integration.replay.completed", Some(entity.id));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "replay degrade: {} echantillon(s) | latence {:?} ms | drop {:?}",
                        report.sample_count,
                        report.effective_metrics.latency_ms,
                        report.effective_metrics.drop_rate
                    ),
                })
            }
            "commissioning.session.start" => {
                let perception_run = if let Some(existing) = self
                    .graph
                    .document()
                    .nodes
                    .values()
                    .filter(|entity| perception_run_summary_from_entity(entity).is_some())
                    .max_by(|left, right| left.id.cmp(&right.id))
                    .cloned()
                {
                    existing
                } else {
                    let rig_index = self.graph.entity_count() + 1;
                    let (rig_entity, _) = build_sensor_rig_entity(
                        &format!("ent_rig_{rig_index:03}"),
                        &format!("SensorRig-{rig_index:03}"),
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(rig_entity.clone()))
                        .map_err(|error| error.to_string())?;
                    let run_index = self.graph.entity_count() + 1;
                    let (run_entity, _) = build_perception_run_entity(
                        &format!("ent_perc_{run_index:03}"),
                        &format!("PerceptionRun-{run_index:03}"),
                        &rig_entity,
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(run_entity.clone()))
                        .map_err(|error| error.to_string())?;
                    run_entity
                };
                let session_index = self.graph.entity_count() + 1;
                let (entity, summary) = build_commissioning_session_entity(
                    &format!("ent_comm_{session_index:03}"),
                    &format!("CommissioningSession-{session_index:03}"),
                    &perception_run,
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.push_system_activity("commissioning.session.started", Some(entity.id));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "session commissioning ouverte: {:.0}% | {} capture(s)",
                        summary.progress_ratio * 100.0,
                        summary.capture_count
                    ),
                })
            }
            "commissioning.compare.as_built" => {
                let session = if let Some(existing) = self
                    .graph
                    .document()
                    .nodes
                    .values()
                    .filter(|entity| commissioning_session_summary_from_entity(entity).is_some())
                    .max_by(|left, right| left.id.cmp(&right.id))
                    .cloned()
                {
                    existing
                } else {
                    let perception_index = self.graph.entity_count() + 1;
                    let (rig_entity, _) = build_sensor_rig_entity(
                        &format!("ent_rig_{perception_index:03}"),
                        &format!("SensorRig-{perception_index:03}"),
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(rig_entity.clone()))
                        .map_err(|error| error.to_string())?;
                    let (run_entity, _) = build_perception_run_entity(
                        &format!("ent_perc_{:03}", self.graph.entity_count() + 1),
                        &format!("PerceptionRun-{:03}", self.graph.entity_count() + 1),
                        &rig_entity,
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(run_entity.clone()))
                        .map_err(|error| error.to_string())?;
                    let (session_entity, _) = build_commissioning_session_entity(
                        &format!("ent_comm_{:03}", self.graph.entity_count() + 1),
                        &format!("CommissioningSession-{:03}", self.graph.entity_count() + 1),
                        &run_entity,
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(session_entity.clone()))
                        .map_err(|error| error.to_string())?;
                    session_entity
                };
                let comparison_index = self.graph.entity_count() + 1;
                let (entity, summary) = build_as_built_comparison_entity(
                    &format!("ent_ab_{comparison_index:03}"),
                    &format!("AsBuiltComparison-{comparison_index:03}"),
                    &session,
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.push_system_activity("commissioning.compare.completed", Some(entity.id));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "comparaison as-built: {} ok | {} ko | {:.2} mm max",
                        summary.accepted_count, summary.rejected_count, summary.max_deviation_mm
                    ),
                })
            }
            "optimization.run.start" => {
                let comparison = if let Some(existing) = self
                    .graph
                    .document()
                    .nodes
                    .values()
                    .filter(|entity| as_built_comparison_summary_from_entity(entity).is_some())
                    .max_by(|left, right| left.id.cmp(&right.id))
                    .cloned()
                {
                    existing
                } else {
                    let session_index = self.graph.entity_count() + 1;
                    let session = sample_entity(
                        "CommissioningSession",
                        &format!("ent_comm_{session_index:03}"),
                        &format!("CommissioningSession-{session_index:03}"),
                        serde_json::json!({
                            "summary": {
                                "status": "capturing",
                                "progressRatio": 0.6,
                                "captureCount": 2,
                                "adjustmentCount": 2
                            }
                        }),
                    );
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(session.clone()))
                        .map_err(|error| error.to_string())?;
                    let (comparison_entity, _) = build_as_built_comparison_entity(
                        &format!("ent_ab_{:03}", self.graph.entity_count() + 1),
                        &format!("AsBuiltComparison-{:03}", self.graph.entity_count() + 1),
                        &session,
                    )?;
                    self.graph
                        .apply_command(CoreCommand::CreateEntity(comparison_entity.clone()))
                        .map_err(|error| error.to_string())?;
                    comparison_entity
                };
                let study_index = self.graph.entity_count() + 1;
                let (entity, summary) = build_optimization_study_entity(
                    &format!("ent_opt_{study_index:03}"),
                    &format!("OptimizationStudy-{study_index:03}"),
                    &comparison,
                )?;
                self.graph
                    .apply_command(CoreCommand::CreateEntity(entity.clone()))
                    .map_err(|error| error.to_string())?;
                self.push_system_activity("optimization.run.completed", Some(entity.id));
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: format!(
                        "optimisation terminee: {} candidat(s) | meilleur {:?} | {:.2}",
                        summary.candidate_count, summary.best_candidate_id, summary.best_score
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
            "help.openspec" => {
                let (document_count, latest_id, latest_title) = {
                    let open_spec_documents = self
                        .graph
                        .document()
                        .open_spec_documents
                        .values()
                        .collect::<Vec<_>>();
                    let latest = open_spec_documents
                        .iter()
                        .max_by(|left, right| left.updated_at.cmp(&right.updated_at));
                    (
                        open_spec_documents.len(),
                        latest.map(|document| document.id.clone()),
                        latest.map(|document| document.title.clone()),
                    )
                };
                self.push_system_activity("help.openspec.reviewed", latest_id);
                Ok(CommandResult {
                    command_id: command_id.to_string(),
                    status: "applied".to_string(),
                    message: if let Some(latest_title) = latest_title {
                        format!(
                            "{} document(s) OpenSpec lisible(s) | dernier: {}",
                            document_count, latest_title
                        )
                    } else {
                        "aucun document OpenSpec enregistre dans ce projet".to_string()
                    },
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

fn stable_hash(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
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

fn open_spec_excerpt(content: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.trim_start_matches('#').trim().to_string())
        .unwrap_or_else(|| "Document OpenSpec sans contenu".to_string())
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
            let sensor_rig_summary = sensor_rig_summary_from_entity(entity);
            let simulation_run_summary = simulation_run_summary_from_entity(entity);
            let safety_report_summary = safety_report_summary_from_entity(entity);
            let perception_run_summary = perception_run_summary_from_entity(entity);
            let commissioning_session_summary = commissioning_session_summary_from_entity(entity);
            let as_built_comparison_summary = as_built_comparison_summary_from_entity(entity);
            let optimization_study_summary = optimization_study_summary_from_entity(entity);
            EntitySummary {
                id: entity.id.clone(),
                entity_type: entity.entity_type.clone(),
                name: entity.name.clone(),
                revision: entity.revision.clone(),
                status: entity.status.clone(),
                data: entity.data.clone(),
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
                        sensor_rig_summary
                            .as_ref()
                            .map(format_sensor_rig_entity_detail)
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
                    })
                    .or_else(|| {
                        perception_run_summary
                            .as_ref()
                            .map(format_perception_run_entity_detail)
                    })
                    .or_else(|| {
                        commissioning_session_summary
                            .as_ref()
                            .map(format_commissioning_session_entity_detail)
                    })
                    .or_else(|| {
                        as_built_comparison_summary
                            .as_ref()
                            .map(format_as_built_comparison_entity_detail)
                    })
                    .or_else(|| {
                        optimization_study_summary
                            .as_ref()
                            .map(format_optimization_study_entity_detail)
                    })
                    .or_else(|| generic_entity_detail(entity)),
                part_geometry,
                assembly_summary,
                robot_cell_summary,
                sensor_rig_summary,
                simulation_run_summary,
                safety_report_summary,
                perception_run_summary,
                commissioning_session_summary,
                as_built_comparison_summary,
                optimization_study_summary,
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

    let open_spec_documents = document
        .open_spec_documents
        .values()
        .map(|document| OpenSpecDocumentSummary {
            id: document.id.clone(),
            title: document.title.clone(),
            kind: document.kind.clone(),
            status: document.status.clone(),
            linked_entity_count: document.entity_refs.len(),
            linked_external_count: document.external_refs.len(),
            tag_count: document.tags.len(),
            excerpt: open_spec_excerpt(&document.content),
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
        open_spec_documents,
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
            "tags": ["part", "parametric"],
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

fn build_empty_assembly_entity(id: &str, name: &str) -> Result<EntityRecord, String> {
    let payload = serde_json::to_value(AssemblyData {
        tags: vec!["assembly".to_string()],
        ..AssemblyData::default()
    })
    .map_err(|error| error.to_string())?;
    Ok(sample_entity("Assembly", id, name, payload))
}

fn sample_assembly_occurrence(part_id: &str, index: usize) -> AssemblyOccurrence {
    AssemblyOccurrence {
        id: format!("occ_{:03}", index + 1),
        definition_entity_id: part_id.to_string(),
        transform: AssemblyTransform {
            x_mm: index as f64 * 80.0,
            ..AssemblyTransform::default()
        },
    }
}

fn sample_assembly_mate(index: usize) -> AssemblyMateConstraint {
    AssemblyMateConstraint {
        id: format!("mate_{:03}", index + 1),
        left_occurrence_id: format!("occ_{:03}", index + 1),
        right_occurrence_id: format!("occ_{:03}", index + 2),
        mate_type: if index == 0 {
            AssemblyMateType::Coincident
        } else {
            AssemblyMateType::Offset {
                distance_mm: 25.0 * index as f64,
            }
        },
    }
}

fn sample_assembly_joint(index: usize) -> AssemblyJoint {
    AssemblyJoint {
        id: format!("joint_{:03}", index + 1),
        joint_type: if index.is_multiple_of(2) {
            AssemblyJointType::Revolute
        } else {
            AssemblyJointType::Prismatic
        },
        source_occurrence_id: format!("occ_{:03}", index + 1),
        target_occurrence_id: format!("occ_{:03}", index + 2),
        axis: AssemblyJointAxis {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        limits: Some(AssemblyJointLimits {
            min: -1.57,
            max: 1.57,
        }),
        current_position: 0.0,
        degrees_of_freedom: 0,
    }
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

fn robot_cell_token(cell_id: &str) -> String {
    cell_id
        .strip_prefix("ent_cell_")
        .unwrap_or(cell_id)
        .to_string()
}

fn robot_cell_scene_assembly_entity_id(cell_id: &str) -> String {
    format!("ent_asm_cell_{}", robot_cell_token(cell_id))
}

fn robot_cell_part_entity_id(cell_id: &str, part_kind: &str) -> String {
    format!("ent_part_{}_{}", part_kind, robot_cell_token(cell_id))
}

fn robot_cell_occurrence_id(part_kind: &str) -> String {
    format!("occ_{}_001", part_kind)
}

fn robot_cell_robot_entity_id(cell_id: &str) -> String {
    format!("ent_robot_{}", robot_cell_token(cell_id))
}

fn robot_cell_equipment_entity_id(cell_id: &str, equipment_kind: &str) -> String {
    format!("ent_{}_{}", equipment_kind, robot_cell_token(cell_id))
}

fn robot_cell_sequence_entity_id(cell_id: &str) -> String {
    format!("ent_seq_{}", robot_cell_token(cell_id))
}

fn robot_cell_target_entity_id(cell_id: &str, target_key: &str) -> String {
    format!("ent_target_{}_{}", robot_cell_token(cell_id), target_key)
}

fn robot_cell_safety_zone_entity_id(cell_id: &str, zone_suffix: &str) -> String {
    format!("ent_zone_{}_{}", robot_cell_token(cell_id), zone_suffix)
}

fn robot_cell_controller_entity_id(cell_id: &str) -> String {
    format!("ent_ctrl_{}", robot_cell_token(cell_id))
}

fn robot_cell_signal_entity_id(cell_id: &str, signal_suffix: &str) -> String {
    format!("ent_sig_{}_{}", robot_cell_token(cell_id), signal_suffix)
}

fn robot_cell_signal_definitions(safety_clear: bool) -> Vec<SignalDefinition> {
    vec![
        SignalDefinition {
            id: "sig_cycle_start".to_string(),
            name: "Cycle Start".to_string(),
            kind: SignalKind::Boolean,
            initial_value: SignalValue::Bool(false),
            unit: None,
            tags: vec!["control".to_string(), "boolean".to_string()],
        },
        SignalDefinition {
            id: "sig_progress_gate".to_string(),
            name: "Progress Gate".to_string(),
            kind: SignalKind::Scalar,
            initial_value: SignalValue::Scalar(0.62),
            unit: Some("ratio".to_string()),
            tags: vec!["control".to_string(), "scalar".to_string()],
        },
        SignalDefinition {
            id: "sig_safety_clear".to_string(),
            name: "Safety Clear".to_string(),
            kind: SignalKind::Boolean,
            initial_value: SignalValue::Bool(safety_clear),
            unit: None,
            tags: vec!["safety".to_string(), "boolean".to_string()],
        },
        SignalDefinition {
            id: "sig_payload_released".to_string(),
            name: "Payload Released".to_string(),
            kind: SignalKind::Boolean,
            initial_value: SignalValue::Bool(false),
            unit: None,
            tags: vec!["process".to_string(), "boolean".to_string()],
        },
        SignalDefinition {
            id: "sig_operator_mode".to_string(),
            name: "Operator Mode".to_string(),
            kind: SignalKind::Text,
            initial_value: SignalValue::Text("auto".to_string()),
            unit: None,
            tags: vec!["control".to_string(), "text".to_string()],
        },
    ]
}

fn ordered_target_preview(targets: &[RobotTarget]) -> String {
    targets
        .iter()
        .map(|target| target.id.as_str())
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn robot_cell_controller_state_machine(cell_id: &str) -> ControllerStateMachine {
    ControllerStateMachine {
        id: format!("ctrl_{}", robot_cell_token(cell_id)),
        name: format!("Controller {}", robot_cell_token(cell_id)),
        initial_state_id: "idle".to_string(),
        states: vec![
            ControllerState {
                id: "idle".to_string(),
                name: "Idle".to_string(),
                terminal: false,
            },
            ControllerState {
                id: "transfer".to_string(),
                name: "Transfer".to_string(),
                terminal: false,
            },
            ControllerState {
                id: "place".to_string(),
                name: "Place".to_string(),
                terminal: false,
            },
            ControllerState {
                id: "done".to_string(),
                name: "Done".to_string(),
                terminal: true,
            },
        ],
        transitions: vec![
            ControlTransition {
                id: "tr_start_cycle".to_string(),
                from_state_id: "idle".to_string(),
                to_state_id: "transfer".to_string(),
                conditions: vec![
                    SignalCondition {
                        signal_id: "sig_cycle_start".to_string(),
                        comparator: SignalComparator::Equal,
                        expected_value: SignalValue::Bool(true),
                    },
                    SignalCondition {
                        signal_id: "sig_safety_clear".to_string(),
                        comparator: SignalComparator::Equal,
                        expected_value: SignalValue::Bool(true),
                    },
                ],
                assignments: vec![],
                description: Some("cycle_start_confirmed".to_string()),
            },
            ControlTransition {
                id: "tr_reach_place".to_string(),
                from_state_id: "transfer".to_string(),
                to_state_id: "place".to_string(),
                conditions: vec![SignalCondition {
                    signal_id: "sig_progress_gate".to_string(),
                    comparator: SignalComparator::GreaterThanOrEqual,
                    expected_value: SignalValue::Scalar(0.55),
                }],
                assignments: vec![],
                description: Some("progress_gate_reached".to_string()),
            },
            ControlTransition {
                id: "tr_finish_cycle".to_string(),
                from_state_id: "place".to_string(),
                to_state_id: "done".to_string(),
                conditions: vec![SignalCondition {
                    signal_id: "sig_progress_gate".to_string(),
                    comparator: SignalComparator::GreaterThanOrEqual,
                    expected_value: SignalValue::Scalar(0.95),
                }],
                assignments: vec![SignalAssignment {
                    signal_id: "sig_payload_released".to_string(),
                    value: SignalValue::Bool(true),
                }],
                description: Some("placement_complete".to_string()),
            },
        ],
    }
}

fn robot_cell_control_model(cell_id: &str, safety_clear: bool) -> RobotCellControlModel {
    RobotCellControlModel {
        cell_id: cell_id.to_string(),
        signals: robot_cell_signal_definitions(safety_clear),
        controller: robot_cell_controller_state_machine(cell_id),
    }
}

fn robot_cell_control_summary(
    control_model: &RobotCellControlModel,
) -> Result<RobotCellControlSummary, String> {
    validate_robot_cell_control(control_model).map_err(|error| error.to_string())?;
    summarize_robot_cell_control(control_model).map_err(|error| error.to_string())
}

fn robot_cell_control_payload(
    cell_id: &str,
    controller_entity_id: &str,
    control_model: &RobotCellControlModel,
    control_summary: &RobotCellControlSummary,
    contact_pairs: serde_json::Value,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "signalCount": control_summary.signal_count,
        "controllerTransitionCount": control_summary.controller_transition_count,
        "blockedSequenceDetected": control_summary.blocked_sequence_detected,
        "blockedStateId": control_summary.blocked_state_id.clone(),
        "controllerId": controller_entity_id,
        "signalIds": control_model
            .signals
            .iter()
            .map(|signal| robot_cell_signal_entity_id(cell_id, signal.id.trim_start_matches("sig_")))
            .collect::<Vec<_>>(),
        "states": control_model.controller.states.clone(),
        "transitions": control_model.controller.transitions.clone(),
        "contactPairs": contact_pairs,
    }))
}

fn robot_cell_contact_pairs(cell_id: &str) -> Vec<SimulationContactPair> {
    let token = robot_cell_token(cell_id);
    vec![
        SimulationContactPair {
            id: format!("pair_{}_tool_fixture", token),
            left_entity_id: format!("ent_tool_{}", token),
            right_entity_id: format!("ent_fixture_{}", token),
            base_clearance_mm: 0.42,
        },
        SimulationContactPair {
            id: format!("pair_{}_tool_conveyor", token),
            left_entity_id: format!("ent_tool_{}", token),
            right_entity_id: format!("ent_conveyor_{}", token),
            base_clearance_mm: 0.36,
        },
    ]
}

fn robot_cell_model(cell_id: &str, safety_zones: &[SafetyZone]) -> RobotCellModel {
    RobotCellModel {
        id: cell_id.to_string(),
        scene_assembly_id: robot_cell_scene_assembly_entity_id(cell_id),
        robot_ids: vec![robot_cell_robot_entity_id(cell_id)],
        equipment_ids: vec![
            robot_cell_equipment_entity_id(cell_id, "conveyor"),
            robot_cell_equipment_entity_id(cell_id, "fixture"),
            robot_cell_equipment_entity_id(cell_id, "tool"),
        ],
        safety_zone_ids: safety_zones
            .iter()
            .map(|zone| match zone.kind {
                SafetyZoneKind::Warning => robot_cell_safety_zone_entity_id(cell_id, "warning"),
                SafetyZoneKind::ProtectiveStop | SafetyZoneKind::LidarProtective => {
                    robot_cell_safety_zone_entity_id(cell_id, "protective")
                }
            })
            .collect(),
        sequence_ids: vec![robot_cell_sequence_entity_id(cell_id)],
        controller_model_ids: vec![robot_cell_controller_entity_id(cell_id)],
    }
}

fn robot_cell_robot_model(cell_id: &str) -> RobotModel {
    RobotModel {
        id: robot_cell_robot_entity_id(cell_id),
        cell_id: cell_id.to_string(),
        kinematic_chain: vec![
            "base".to_string(),
            "shoulder".to_string(),
            "wrist".to_string(),
            "tool".to_string(),
        ],
        joint_ids: vec!["joint_axis_001".to_string()],
        tool_mount_ref: RobotToolMountRef {
            equipment_id: robot_cell_equipment_entity_id(cell_id, "tool"),
            role: "tool".to_string(),
        },
        workspace_bounds: RobotWorkspaceBounds {
            reach_radius_mm: 1_450.0,
            vertical_span_mm: 1_900.0,
        },
        payload_limits: RobotPayloadLimits {
            nominal_kg: 8.0,
            max_kg: 12.0,
        },
        calibration_state: "seeded".to_string(),
    }
}

fn robot_cell_equipment_models(cell_id: &str) -> Vec<EquipmentModel> {
    vec![
        EquipmentModel {
            id: robot_cell_equipment_entity_id(cell_id, "conveyor"),
            cell_id: cell_id.to_string(),
            equipment_type: EquipmentType::Conveyor,
            assembly_occurrence_id: robot_cell_occurrence_id("conveyor"),
            parameter_set: EquipmentParameterSet {
                width_mm: 850.0,
                height_mm: 220.0,
                depth_mm: 600.0,
                nominal_speed_mm_s: Some(320),
            },
            io_port_ids: vec![robot_cell_signal_entity_id(cell_id, "cycle_start")],
        },
        EquipmentModel {
            id: robot_cell_equipment_entity_id(cell_id, "fixture"),
            cell_id: cell_id.to_string(),
            equipment_type: EquipmentType::Workstation,
            assembly_occurrence_id: robot_cell_occurrence_id("fixture"),
            parameter_set: EquipmentParameterSet {
                width_mm: 640.0,
                height_mm: 180.0,
                depth_mm: 480.0,
                nominal_speed_mm_s: None,
            },
            io_port_ids: vec![robot_cell_signal_entity_id(cell_id, "progress_gate")],
        },
        EquipmentModel {
            id: robot_cell_equipment_entity_id(cell_id, "tool"),
            cell_id: cell_id.to_string(),
            equipment_type: EquipmentType::Gripper,
            assembly_occurrence_id: robot_cell_occurrence_id("tool"),
            parameter_set: EquipmentParameterSet {
                width_mm: 110.0,
                height_mm: 80.0,
                depth_mm: 140.0,
                nominal_speed_mm_s: None,
            },
            io_port_ids: vec![robot_cell_signal_entity_id(cell_id, "payload_released")],
        },
    ]
}

fn robot_cell_target_models(
    cell_id: &str,
    sequence_id: &str,
    targets: &[RobotTarget],
) -> Vec<RobotTargetModel> {
    targets
        .iter()
        .enumerate()
        .map(|(index, target)| RobotTargetModel {
            id: robot_cell_target_entity_id(cell_id, &target.id),
            cell_id: cell_id.to_string(),
            sequence_id: sequence_id.to_string(),
            target_key: target.id.clone(),
            order_index: (index + 1) as u32,
            pose: target.pose,
            nominal_speed_mm_s: target.nominal_speed_mm_s,
            dwell_time_ms: target.dwell_time_ms,
        })
        .collect()
}

fn robot_target_entity(model: &RobotTargetModel) -> EntityRecord {
    sample_entity(
        "RobotTarget",
        &model.id,
        &format!("Target {}", model.target_key),
        serde_json::json!({
            "id": model.id,
            "cellId": model.cell_id,
            "sequenceId": model.sequence_id,
            "targetKey": model.target_key,
            "orderIndex": model.order_index,
            "pose": model.pose,
            "nominalSpeedMmS": model.nominal_speed_mm_s,
            "dwellTimeMs": model.dwell_time_ms,
            "tags": ["robotics", "target", "sequence"],
            "parameterSet": {
                "orderIndex": model.order_index,
                "xMm": model.pose.x_mm,
                "yMm": model.pose.y_mm,
                "zMm": model.pose.z_mm,
                "nominalSpeedMmS": model.nominal_speed_mm_s,
                "dwellTimeMs": model.dwell_time_ms
            }
        }),
    )
}

fn robot_target_model_from_entity(entity: &EntityRecord) -> Option<RobotTargetModel> {
    if entity.entity_type != "RobotTarget" {
        return None;
    }
    let parameters = entity.data.get("parameterSet")?.as_object()?;
    Some(RobotTargetModel {
        id: entity.id.clone(),
        cell_id: entity.data.get("cellId")?.as_str()?.to_string(),
        sequence_id: entity.data.get("sequenceId")?.as_str()?.to_string(),
        target_key: entity.data.get("targetKey")?.as_str()?.to_string(),
        order_index: parameters.get("orderIndex")?.as_u64()? as u32,
        pose: CartesianPose {
            x_mm: parameters.get("xMm")?.as_f64()?,
            y_mm: parameters.get("yMm")?.as_f64()?,
            z_mm: parameters.get("zMm")?.as_f64()?,
        },
        nominal_speed_mm_s: parameters.get("nominalSpeedMmS")?.as_u64()? as u32,
        dwell_time_ms: parameters.get("dwellTimeMs")?.as_u64()? as u32,
    })
}

fn robot_cell_sequence_model(
    cell_id: &str,
    target_models: &[RobotTargetModel],
    validation: &faero_robotics::SequenceValidation,
) -> RobotSequenceModel {
    let mut ordered_target_ids = target_models
        .iter()
        .map(|target| (target.order_index, target.id.clone()))
        .collect::<Vec<_>>();
    ordered_target_ids.sort_by_key(|(order_index, _)| *order_index);
    RobotSequenceModel {
        id: robot_cell_sequence_entity_id(cell_id),
        cell_id: cell_id.to_string(),
        robot_id: robot_cell_robot_entity_id(cell_id),
        target_ids: ordered_target_ids
            .into_iter()
            .map(|(_, target_id)| target_id)
            .collect(),
        path_length_mm: validation.path_length_mm,
        estimated_cycle_time_ms: validation.estimated_cycle_time_ms,
    }
}

fn robot_cell_scene_parts(cell_id: &str) -> Result<Vec<EntityRecord>, String> {
    let definitions = [
        ("robot", "RobotBase", 520.0, 640.0, 420.0),
        ("conveyor", "Conveyor", 850.0, 220.0, 600.0),
        ("fixture", "Workstation", 640.0, 180.0, 480.0),
        ("tool", "Gripper", 110.0, 80.0, 140.0),
    ];
    definitions
        .into_iter()
        .map(|(kind, label, width_mm, height_mm, depth_mm)| {
            let id = robot_cell_part_entity_id(cell_id, kind);
            build_parametric_part_entity(
                &id,
                &format!("{} / {}", robot_cell_token(cell_id), label),
                width_mm,
                height_mm,
                depth_mm,
            )
            .map(|(entity, _)| entity)
        })
        .collect()
}

fn robot_cell_scene_occurrences(cell_id: &str) -> Vec<AssemblyOccurrence> {
    vec![
        AssemblyOccurrence {
            id: robot_cell_occurrence_id("robot"),
            definition_entity_id: robot_cell_part_entity_id(cell_id, "robot"),
            transform: AssemblyTransform::default(),
        },
        AssemblyOccurrence {
            id: robot_cell_occurrence_id("conveyor"),
            definition_entity_id: robot_cell_part_entity_id(cell_id, "conveyor"),
            transform: AssemblyTransform {
                x_mm: 620.0,
                y_mm: 120.0,
                ..AssemblyTransform::default()
            },
        },
        AssemblyOccurrence {
            id: robot_cell_occurrence_id("fixture"),
            definition_entity_id: robot_cell_part_entity_id(cell_id, "fixture"),
            transform: AssemblyTransform {
                x_mm: 980.0,
                y_mm: -140.0,
                ..AssemblyTransform::default()
            },
        },
        AssemblyOccurrence {
            id: robot_cell_occurrence_id("tool"),
            definition_entity_id: robot_cell_part_entity_id(cell_id, "tool"),
            transform: AssemblyTransform {
                x_mm: 120.0,
                z_mm: 260.0,
                ..AssemblyTransform::default()
            },
        },
    ]
}

fn robot_cell_scene_mates() -> Vec<AssemblyMateConstraint> {
    vec![
        AssemblyMateConstraint {
            id: "mate_robot_conveyor".to_string(),
            left_occurrence_id: robot_cell_occurrence_id("robot"),
            right_occurrence_id: robot_cell_occurrence_id("conveyor"),
            mate_type: AssemblyMateType::Offset { distance_mm: 620.0 },
        },
        AssemblyMateConstraint {
            id: "mate_conveyor_fixture".to_string(),
            left_occurrence_id: robot_cell_occurrence_id("conveyor"),
            right_occurrence_id: robot_cell_occurrence_id("fixture"),
            mate_type: AssemblyMateType::Offset { distance_mm: 360.0 },
        },
        AssemblyMateConstraint {
            id: "mate_robot_tool".to_string(),
            left_occurrence_id: robot_cell_occurrence_id("robot"),
            right_occurrence_id: robot_cell_occurrence_id("tool"),
            mate_type: AssemblyMateType::Offset { distance_mm: 120.0 },
        },
    ]
}

fn build_robot_cell_support_entities(cell: &EntityRecord) -> Result<Vec<EntityRecord>, String> {
    let safety = cell
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
    let raw_targets = serde_json::from_value::<Vec<RobotTarget>>(
        cell.data
            .get("targets")
            .cloned()
            .ok_or_else(|| "robot cell targets missing".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let sequence_id = robot_cell_sequence_entity_id(&cell.id);
    let target_models = robot_cell_target_models(&cell.id, &sequence_id, &raw_targets);
    let targets = validate_target_models(&cell.id, &sequence_id, &target_models)
        .map_err(|error| error.to_string())?;
    let validation = validate_sequence(&targets).map_err(|error| error.to_string())?;
    let target_preview = ordered_target_preview(&targets);
    let safety_clear = !evaluate_safety(&zones, &interlocks, "robot.move").inhibited;
    let control_model = robot_cell_control_model(&cell.id, safety_clear);
    let structure = robot_cell_model(&cell.id, &zones);
    let robot_model = robot_cell_robot_model(&cell.id);
    let equipment_models = robot_cell_equipment_models(&cell.id);
    let sequence_model = robot_cell_sequence_model(&cell.id, &target_models, &validation);
    let structure_summary = validate_robot_cell_structure(
        &structure,
        std::slice::from_ref(&robot_model),
        &equipment_models,
        std::slice::from_ref(&sequence_model),
    )
    .map_err(|error| error.to_string())?;
    let mut entities = vec![sample_entity(
        "Assembly",
        &structure.scene_assembly_id,
        &format!("{} / Scene", cell.name),
        serde_json::to_value(AssemblyData {
            tags: vec![
                "robotics".to_string(),
                "scene".to_string(),
                "mvp".to_string(),
            ],
            ..AssemblyData::default()
        })
        .map_err(|error| error.to_string())?,
    )];
    entities.extend(robot_cell_scene_parts(&cell.id)?);
    entities.push(sample_entity(
        "RobotModel",
        &robot_model.id,
        &format!("{} / Robot", cell.name),
        serde_json::to_value(&robot_model).map_err(|error| error.to_string())?,
    ));
    entities.extend(equipment_models.iter().map(|model| {
        let label = match model.equipment_type {
            EquipmentType::Conveyor => "Conveyor",
            EquipmentType::Workstation => "Workstation",
            EquipmentType::Gripper => "Gripper",
        };
        sample_entity(
            "EquipmentModel",
            &model.id,
            &format!("{} / {}", cell.name, label),
            serde_json::to_value(model).expect("equipment model should serialize"),
        )
    }));
    entities.extend(zones.iter().map(|zone| {
        let zone_suffix = match zone.kind {
            SafetyZoneKind::Warning => "warning",
            SafetyZoneKind::ProtectiveStop | SafetyZoneKind::LidarProtective => "protective",
        };
        sample_entity(
            "SafetyZoneModel",
            &robot_cell_safety_zone_entity_id(&cell.id, zone_suffix),
            &format!("{} / {}", cell.name, zone.id),
            serde_json::json!({
                "id": robot_cell_safety_zone_entity_id(&cell.id, zone_suffix),
                "cellId": cell.id,
                "zoneId": zone.id,
                "zoneKind": match zone.kind {
                    SafetyZoneKind::Warning => "warning",
                    SafetyZoneKind::ProtectiveStop => "protective_stop",
                    SafetyZoneKind::LidarProtective => "lidar_protective",
                },
                "active": zone.active,
                "interlockIds": interlocks
                    .iter()
                    .filter(|interlock| interlock.source_zone_id == zone.id)
                    .map(|interlock| interlock.id.clone())
                    .collect::<Vec<_>>()
            }),
        )
    }));
    entities.extend(target_models.iter().map(robot_target_entity));
    entities.push(sample_entity(
        "RobotSequence",
        &sequence_model.id,
        &format!("{} / Sequence", cell.name),
        serde_json::json!({
            "id": sequence_model.id,
            "cellId": sequence_model.cell_id,
            "robotId": sequence_model.robot_id,
            "targetIds": sequence_model.target_ids,
            "targets": targets,
            "pathLengthMm": sequence_model.path_length_mm,
            "estimatedCycleTimeMs": sequence_model.estimated_cycle_time_ms,
            "targetCount": validation.target_count,
            "targetPreview": target_preview,
            "structureSummary": {
                "robotCount": structure_summary.robot_count,
                "equipmentCount": structure_summary.equipment_count,
                "safetyZoneCount": structure_summary.safety_zone_count,
                "sequenceCount": structure_summary.sequence_count,
                "controllerCount": structure_summary.controller_count
            }
        }),
    ));
    entities.extend(control_model.signals.iter().map(|signal| {
        sample_entity(
            "Signal",
            &robot_cell_signal_entity_id(&cell.id, signal.id.trim_start_matches("sig_")),
            &format!("{} / {}", cell.name, signal.name),
            serde_json::json!({
                "cellId": cell.id,
                "signalId": signal.id,
                "kind": match &signal.kind {
                    SignalKind::Boolean => "boolean",
                    SignalKind::Scalar => "scalar",
                    SignalKind::Text => "text",
                },
                "initialValue": signal.initial_value.clone(),
                "currentValue": signal.initial_value.clone(),
                "tags": signal.tags.clone(),
                "parameterSet": {
                    "currentValue": signal.initial_value.clone(),
                    "unit": signal.unit.clone(),
                }
            }),
        )
    }));
    entities.push(sample_entity(
        "ControllerModel",
        &robot_cell_controller_entity_id(&cell.id),
        &format!("{} / Controller", cell.name),
        serde_json::json!({
            "cellId": cell.id,
            "stateMachine": control_model.controller.clone(),
            "tags": ["control", "state_machine"],
            "parameterSet": {
                "stateCount": control_model.controller.states.len(),
                "transitionCount": control_model.controller.transitions.len(),
            }
        }),
    ));
    Ok(entities)
}

fn signal_definition_from_entity(entity: &EntityRecord) -> Option<SignalDefinition> {
    if entity.entity_type != "Signal" {
        return None;
    }
    let kind = match entity.data.get("kind")?.as_str()? {
        "boolean" => SignalKind::Boolean,
        "scalar" => SignalKind::Scalar,
        "text" => SignalKind::Text,
        _ => return None,
    };
    Some(SignalDefinition {
        id: entity.data.get("signalId")?.as_str()?.to_string(),
        name: entity.name.clone(),
        kind,
        initial_value: serde_json::from_value(
            entity
                .data
                .get("currentValue")
                .cloned()
                .or_else(|| entity.data.get("initialValue").cloned())?,
        )
        .ok()?,
        unit: entity
            .data
            .get("parameterSet")
            .and_then(|value| value.get("unit"))
            .and_then(|value| value.as_str())
            .map(str::to_string),
        tags: entity
            .data
            .get("tags")
            .and_then(|value| value.as_array())
            .map(|tags| {
                tags.iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    })
}

fn controller_state_machine_from_entity(entity: &EntityRecord) -> Option<ControllerStateMachine> {
    if entity.entity_type != "ControllerModel" {
        return None;
    }
    serde_json::from_value(entity.data.get("stateMachine")?.clone()).ok()
}

fn robot_cell_control_model_from_entities(
    cell_id: &str,
    signal_entities: &[EntityRecord],
    controller_entity: &EntityRecord,
) -> Result<RobotCellControlModel, String> {
    let controller = controller_state_machine_from_entity(controller_entity)
        .ok_or_else(|| "controller state machine missing".to_string())?;
    let signals = signal_entities
        .iter()
        .map(signal_definition_from_entity)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "invalid signal entity found in robot cell control graph".to_string())?;
    let control_model = RobotCellControlModel {
        cell_id: cell_id.to_string(),
        signals,
        controller,
    };
    validate_robot_cell_control(&control_model).map_err(|error| error.to_string())?;
    Ok(control_model)
}

fn build_robot_cell_entity(
    id: &str,
    name: &str,
) -> Result<(EntityRecord, RobotCellEntitySummary), String> {
    let sequence_id = robot_cell_sequence_entity_id(id);
    let target_models = robot_cell_target_models(id, &sequence_id, &sample_robot_targets());
    let targets = validate_target_models(id, &sequence_id, &target_models)
        .map_err(|error| error.to_string())?;
    let validation = validate_sequence(&targets).map_err(|error| error.to_string())?;
    let target_preview = ordered_target_preview(&targets);
    let safety_zones = sample_safety_zones();
    let safety_interlocks = sample_safety_interlocks();
    let safety_clear = !evaluate_safety(&safety_zones, &safety_interlocks, "robot.move").inhibited;
    let control_model = robot_cell_control_model(id, safety_clear);
    let control_summary = robot_cell_control_summary(&control_model)?;
    let structure = robot_cell_model(id, &safety_zones);
    let robot_model = robot_cell_robot_model(id);
    let equipment_models = robot_cell_equipment_models(id);
    let sequence_model = robot_cell_sequence_model(id, &target_models, &validation);
    let structure_summary = validate_robot_cell_structure(
        &structure,
        std::slice::from_ref(&robot_model),
        &equipment_models,
        std::slice::from_ref(&sequence_model),
    )
    .map_err(|error| error.to_string())?;
    let control_payload = robot_cell_control_payload(
        id,
        &robot_cell_controller_entity_id(id),
        &control_model,
        &control_summary,
        serde_json::json!(robot_cell_contact_pairs(id)),
    )?;
    let summary = RobotCellEntitySummary {
        scene_assembly_id: Some(structure.scene_assembly_id.clone()),
        target_preview: Some(target_preview.clone()),
        target_count: validation.target_count,
        path_length_mm: validation.path_length_mm,
        max_segment_mm: validation.max_segment_mm,
        estimated_cycle_time_ms: validation.estimated_cycle_time_ms,
        equipment_count: structure_summary.equipment_count,
        sequence_count: structure_summary.sequence_count,
        safety_zone_count: structure_summary.safety_zone_count,
        signal_count: control_summary.signal_count,
        controller_transition_count: control_summary.controller_transition_count,
        blocked_sequence_detected: control_summary.blocked_sequence_detected,
        blocked_state_id: control_summary.blocked_state_id.clone(),
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
            "tags": ["robotics", "simulation", "mvp"],
            "parameterSet": {
                "tcpPayloadKg": 8.0,
                "estimatedCycleTimeMs": summary.estimated_cycle_time_ms,
                "equipmentCount": summary.equipment_count,
                "sequenceCount": summary.sequence_count
            },
            "id": structure.id,
            "sceneAssemblyId": structure.scene_assembly_id,
            "robotIds": structure.robot_ids,
            "equipmentIds": structure.equipment_ids,
            "safetyZoneIds": structure.safety_zone_ids,
            "sequenceIds": structure.sequence_ids,
            "controllerModelIds": structure.controller_model_ids,
            "targetIds": target_models.iter().map(|target| target.id.clone()).collect::<Vec<_>>(),
            "targetPreview": target_preview,
            "targets": targets,
            "sequenceValidation": {
                "targetCount": summary.target_count,
                "pathLengthMm": summary.path_length_mm,
                "maxSegmentMm": summary.max_segment_mm,
                "estimatedCycleTimeMs": summary.estimated_cycle_time_ms,
                "warningCount": summary.warning_count,
                "sequenceEntityId": sequence_model.id
            },
            "control": control_payload,
            "safety": {
                "zoneCount": summary.safety_zone_count,
                "interlockCount": safety_interlocks.len(),
                "zones": safety_zones,
                "interlocks": safety_interlocks
            },
            "structureSummary": {
                "robotCount": structure_summary.robot_count,
                "equipmentCount": structure_summary.equipment_count,
                "safetyZoneCount": structure_summary.safety_zone_count,
                "sequenceCount": structure_summary.sequence_count,
                "controllerCount": structure_summary.controller_count
            }
        }),
    );

    Ok((entity, summary))
}

fn normalize_robot_target_entity_data(
    entity_id: &str,
    data: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<RobotTargetModel, String> {
    let parameters = data
        .get("parameterSet")
        .and_then(|value| value.as_object())
        .ok_or_else(|| "RobotTarget.parameterSet requis".to_string())?;
    let order_index = parameters
        .get("orderIndex")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| {
            "RobotTarget.parameterSet.orderIndex doit rester entier positif".to_string()
        })? as u32;
    let x_mm = parameters
        .get("xMm")
        .and_then(|value| value.as_f64())
        .ok_or_else(|| "RobotTarget.parameterSet.xMm doit rester numerique".to_string())?;
    let y_mm = parameters
        .get("yMm")
        .and_then(|value| value.as_f64())
        .ok_or_else(|| "RobotTarget.parameterSet.yMm doit rester numerique".to_string())?;
    let z_mm = parameters
        .get("zMm")
        .and_then(|value| value.as_f64())
        .ok_or_else(|| "RobotTarget.parameterSet.zMm doit rester numerique".to_string())?;
    let nominal_speed_mm_s = parameters
        .get("nominalSpeedMmS")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| {
            "RobotTarget.parameterSet.nominalSpeedMmS doit rester entier positif".to_string()
        })? as u32;
    let dwell_time_ms = parameters
        .get("dwellTimeMs")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| "RobotTarget.parameterSet.dwellTimeMs doit rester entier".to_string())?
        as u32;
    let model = RobotTargetModel {
        id: entity_id.to_string(),
        cell_id: data
            .get("cellId")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "RobotTarget.cellId requis".to_string())?
            .to_string(),
        sequence_id: data
            .get("sequenceId")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "RobotTarget.sequenceId requis".to_string())?
            .to_string(),
        target_key: data
            .get("targetKey")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "RobotTarget.targetKey requis".to_string())?
            .to_string(),
        order_index,
        pose: CartesianPose { x_mm, y_mm, z_mm },
        nominal_speed_mm_s,
        dwell_time_ms,
    };
    data.insert(
        "orderIndex".to_string(),
        serde_json::json!(model.order_index),
    );
    data.insert(
        "pose".to_string(),
        serde_json::to_value(model.pose).map_err(|error| error.to_string())?,
    );
    data.insert(
        "nominalSpeedMmS".to_string(),
        serde_json::json!(model.nominal_speed_mm_s),
    );
    data.insert(
        "dwellTimeMs".to_string(),
        serde_json::json!(model.dwell_time_ms),
    );
    Ok(model)
}

fn format_robot_target_entity_detail(model: &RobotTargetModel) -> String {
    format!(
        "#{} {} | {:.0}, {:.0}, {:.0} | {} mm/s",
        model.order_index,
        model.target_key,
        model.pose.x_mm,
        model.pose.y_mm,
        model.pose.z_mm,
        model.nominal_speed_mm_s
    )
}

impl WorkspaceSession {
    fn preview_robot_cell_control_update(&self, next_entity: &EntityRecord) -> Result<(), String> {
        if !matches!(
            next_entity.entity_type.as_str(),
            "Signal" | "ControllerModel"
        ) {
            return Ok(());
        }
        let cell_id = next_entity
            .data
            .get("cellId")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "control entity missing cellId".to_string())?;
        let signal_entities = self
            .graph
            .document()
            .nodes
            .values()
            .filter(|entity| entity.entity_type == "Signal")
            .filter(|entity| {
                entity.data.get("cellId").and_then(|value| value.as_str()) == Some(cell_id)
            })
            .map(|entity| {
                if entity.id == next_entity.id {
                    next_entity.clone()
                } else {
                    entity.clone()
                }
            })
            .collect::<Vec<_>>();
        let controller_entity = self
            .graph
            .document()
            .nodes
            .values()
            .find(|entity| entity.entity_type == "ControllerModel")
            .filter(|entity| {
                entity.data.get("cellId").and_then(|value| value.as_str()) == Some(cell_id)
            })
            .map(|entity| {
                if entity.id == next_entity.id {
                    next_entity.clone()
                } else {
                    entity.clone()
                }
            })
            .ok_or_else(|| "controller entity missing for robot cell".to_string())?;
        let control_model =
            robot_cell_control_model_from_entities(cell_id, &signal_entities, &controller_entity)?;
        robot_cell_control_summary(&control_model)?;
        Ok(())
    }

    fn sync_robot_target_dependents(&mut self, target_entity: &EntityRecord) -> Result<(), String> {
        let target_model = robot_target_model_from_entity(target_entity)
            .ok_or_else(|| "RobotTarget invalide".to_string())?;
        let mut target_models = self
            .graph
            .document()
            .nodes
            .values()
            .filter_map(robot_target_model_from_entity)
            .filter(|model| {
                model.cell_id == target_model.cell_id
                    && model.sequence_id == target_model.sequence_id
            })
            .collect::<Vec<_>>();
        let ordered_targets = validate_target_models(
            &target_model.cell_id,
            &target_model.sequence_id,
            &target_models,
        )
        .map_err(|error| error.to_string())?;
        let validation = validate_sequence(&ordered_targets).map_err(|error| error.to_string())?;
        target_models.sort_by_key(|model| model.order_index);
        let ordered_target_ids = target_models
            .iter()
            .map(|model| model.id.clone())
            .collect::<Vec<_>>();
        let target_preview = ordered_target_preview(&ordered_targets);

        if let Some(sequence) = self
            .graph
            .document()
            .nodes
            .get(&target_model.sequence_id)
            .cloned()
        {
            let mut next_sequence = sequence;
            if let Some(data) = next_sequence.data.as_object_mut() {
                data.insert(
                    "targetIds".to_string(),
                    serde_json::json!(ordered_target_ids.clone()),
                );
                data.insert(
                    "targets".to_string(),
                    serde_json::to_value(&ordered_targets).map_err(|error| error.to_string())?,
                );
                data.insert(
                    "pathLengthMm".to_string(),
                    serde_json::json!(validation.path_length_mm),
                );
                data.insert(
                    "estimatedCycleTimeMs".to_string(),
                    serde_json::json!(validation.estimated_cycle_time_ms),
                );
                data.insert(
                    "targetCount".to_string(),
                    serde_json::json!(validation.target_count),
                );
                data.insert(
                    "targetPreview".to_string(),
                    serde_json::json!(target_preview.clone()),
                );
            }
            self.graph
                .apply_command(CoreCommand::ReplaceEntity(next_sequence))
                .map_err(|error| error.to_string())?;
        }

        if let Some(cell) = self
            .graph
            .document()
            .nodes
            .get(&target_model.cell_id)
            .cloned()
        {
            let mut next_cell = cell;
            if let Some(data) = next_cell.data.as_object_mut() {
                data.insert(
                    "targetIds".to_string(),
                    serde_json::json!(ordered_target_ids),
                );
                data.insert(
                    "targets".to_string(),
                    serde_json::to_value(&ordered_targets).map_err(|error| error.to_string())?,
                );
                data.insert(
                    "targetPreview".to_string(),
                    serde_json::json!(target_preview.clone()),
                );
                if let Some(validation_data) = data
                    .get_mut("sequenceValidation")
                    .and_then(|value| value.as_object_mut())
                {
                    validation_data.insert(
                        "targetCount".to_string(),
                        serde_json::json!(validation.target_count),
                    );
                    validation_data.insert(
                        "pathLengthMm".to_string(),
                        serde_json::json!(validation.path_length_mm),
                    );
                    validation_data.insert(
                        "maxSegmentMm".to_string(),
                        serde_json::json!(validation.max_segment_mm),
                    );
                    validation_data.insert(
                        "estimatedCycleTimeMs".to_string(),
                        serde_json::json!(validation.estimated_cycle_time_ms),
                    );
                    validation_data.insert(
                        "warningCount".to_string(),
                        serde_json::json!(validation.warning_count),
                    );
                }
                if let Some(parameters) = data
                    .get_mut("parameterSet")
                    .and_then(|value| value.as_object_mut())
                {
                    parameters.insert(
                        "targetCount".to_string(),
                        serde_json::json!(validation.target_count),
                    );
                    parameters.insert(
                        "estimatedCycleTimeMs".to_string(),
                        serde_json::json!(validation.estimated_cycle_time_ms),
                    );
                }
            }
            robot_cell_summary_from_entity(&next_cell)
                .ok_or_else(|| "resume RobotCell invalide apres sync cible".to_string())?;
            self.graph
                .apply_command(CoreCommand::ReplaceEntity(next_cell))
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    fn sync_robot_cell_control_dependents(
        &mut self,
        cell_id: &str,
    ) -> Result<RobotCellEntitySummary, String> {
        let robot_cell = self
            .graph
            .document()
            .nodes
            .get(cell_id)
            .cloned()
            .ok_or_else(|| "robot cell missing for control sync".to_string())?;
        let signal_entities = self
            .graph
            .document()
            .nodes
            .values()
            .filter(|entity| entity.entity_type == "Signal")
            .filter(|entity| {
                entity.data.get("cellId").and_then(|value| value.as_str()) == Some(cell_id)
            })
            .cloned()
            .collect::<Vec<_>>();
        let controller_entity = self
            .graph
            .document()
            .nodes
            .values()
            .find(|entity| entity.entity_type == "ControllerModel")
            .filter(|entity| {
                entity.data.get("cellId").and_then(|value| value.as_str()) == Some(cell_id)
            })
            .cloned()
            .ok_or_else(|| "controller entity missing for robot cell".to_string())?;
        let control_model =
            robot_cell_control_model_from_entities(cell_id, &signal_entities, &controller_entity)?;
        let control_summary = robot_cell_control_summary(&control_model)?;

        let mut next_controller = controller_entity.clone();
        if let Some(data) = next_controller.data.as_object_mut() {
            data.insert(
                "stateMachine".to_string(),
                serde_json::to_value(&control_model.controller)
                    .map_err(|error| error.to_string())?,
            );
            let parameter_set = data
                .entry("parameterSet".to_string())
                .or_insert_with(|| serde_json::json!({}));
            if let Some(parameter_set) = parameter_set.as_object_mut() {
                parameter_set.insert(
                    "stateCount".to_string(),
                    serde_json::json!(control_model.controller.states.len()),
                );
                parameter_set.insert(
                    "transitionCount".to_string(),
                    serde_json::json!(control_model.controller.transitions.len()),
                );
            }
        }
        if next_controller.data != controller_entity.data
            || next_controller.name != controller_entity.name
        {
            self.graph
                .apply_command(CoreCommand::ReplaceEntity(next_controller))
                .map_err(|error| error.to_string())?;
        }

        let mut next_cell = robot_cell.clone();
        if let Some(data) = next_cell.data.as_object_mut() {
            let contact_pairs = data
                .get("control")
                .and_then(|value| value.get("contactPairs"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!(robot_cell_contact_pairs(cell_id)));
            data.insert(
                "control".to_string(),
                robot_cell_control_payload(
                    cell_id,
                    &controller_entity.id,
                    &control_model,
                    &control_summary,
                    contact_pairs,
                )?,
            );
        }
        let summary = robot_cell_summary_from_entity(&next_cell)
            .ok_or_else(|| "resume RobotCell invalide apres sync controle".to_string())?;
        self.graph
            .apply_command(CoreCommand::ReplaceEntity(next_cell))
            .map_err(|error| error.to_string())?;

        Ok(summary)
    }
}

fn build_simulation_run_entity(
    id: &str,
    name: &str,
    robot_cell: &EntityRecord,
    signal_entities: &[EntityRecord],
    controller_entity: &EntityRecord,
    endpoint_count: usize,
) -> Result<(EntityRecord, SimulationRunEntitySummary), String> {
    let robot_summary = robot_cell_summary_from_entity(robot_cell)
        .ok_or_else(|| "robot cell summary missing".to_string())?;
    let control_model =
        robot_cell_control_model_from_entities(&robot_cell.id, signal_entities, controller_entity)?;
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
    let safety_evaluation = evaluate_safety(&zones, &interlocks, "robot.move");
    let mut scheduled_signal_changes = vec![
        ScheduledSignalChange {
            step_index: 1,
            signal_id: "sig_cycle_start".to_string(),
            value: SignalValue::Bool(true),
            reason: "simulation.run.start".to_string(),
        },
        ScheduledSignalChange {
            step_index: 3,
            signal_id: "sig_progress_gate".to_string(),
            value: SignalValue::Scalar(0.62),
            reason: "robot.transfer.reached".to_string(),
        },
    ];
    if !safety_evaluation.inhibited {
        scheduled_signal_changes.push(ScheduledSignalChange {
            step_index: 7,
            signal_id: "sig_progress_gate".to_string(),
            value: SignalValue::Scalar(1.0),
            reason: "robot.place.completed".to_string(),
        });
    }
    let request = SimulationRequest {
        scenario_name: robot_cell.name.clone(),
        seed: robot_summary.target_count as u64 * 97 + endpoint_count as u64 * 17,
        engine_version: "faero-sim@0.2.0".to_string(),
        step_count: (robot_summary.target_count.max(3) as u32) * 4,
        planned_cycle_time_ms: robot_summary.estimated_cycle_time_ms,
        path_length_mm: robot_summary.path_length_mm,
        endpoint_count: endpoint_count as u32,
        safety_zone_count: robot_summary.safety_zone_count as u32,
        signals: control_model.signals.clone(),
        controller: Some(control_model.controller.clone()),
        scheduled_signal_changes,
        contact_pairs: robot_cell_contact_pairs(&robot_cell.id),
    };
    let run = run_simulation(&request).map_err(|error| error.to_string())?;
    let summary = SimulationRunEntitySummary {
        status: simulation_status_label(&run.summary.status).to_string(),
        collision_count: run.metrics.collision_count,
        cycle_time_ms: run.metrics.cycle_time_ms,
        max_tracking_error_mm: run.metrics.max_tracking_error_mm,
        energy_estimate_j: run.metrics.energy_estimate_j,
        blocked_sequence_detected: run.summary.blocked_sequence_detected,
        blocked_state_id: run.summary.blocked_state_id.clone(),
        contact_count: run.summary.contact_count,
        signal_sample_count: run.summary.signal_sample_count,
        controller_state_sample_count: run.summary.controller_state_sample_count,
        timeline_sample_count: run.summary.timeline_sample_count,
        job_status: "completed".to_string(),
        job_phase: "completed".to_string(),
        job_progress: 1.0,
    };
    let signal_ids = control_model
        .signals
        .iter()
        .map(|signal| signal.id.clone())
        .collect::<Vec<_>>();
    let entity = sample_entity(
        "SimulationRun",
        id,
        name,
        serde_json::json!({
            "tags": ["simulation", "artifact", "mvp"],
            "robotCellId": robot_cell.id.clone(),
            "scenario": {
                "name": run.scenario.name.clone(),
                "seed": run.scenario.seed,
                "engineVersion": run.scenario.engine_version.clone(),
                "stepCount": run.scenario.step_count,
                "plannedCycleTimeMs": run.scenario.planned_cycle_time_ms,
                "pathLengthMm": run.scenario.path_length_mm,
                "endpointCount": run.scenario.endpoint_count,
                "safetyZoneCount": run.scenario.safety_zone_count,
                "signalCount": run.scenario.signal_count,
                "scheduledSignalChangeCount": run.scenario.scheduled_signal_change_count,
                "contactPairCount": run.scenario.contact_pair_count,
                "source": {
                    "robotCellId": robot_cell.id.clone(),
                    "controllerId": controller_entity.id.clone(),
                    "signalIds": signal_ids,
                }
            },
            "summary": {
                "status": summary.status.clone(),
                "blockedSequenceDetected": summary.blocked_sequence_detected,
                "blockedStateId": summary.blocked_state_id.clone(),
                "contactCount": summary.contact_count,
                "signalSampleCount": summary.signal_sample_count,
                "controllerStateSampleCount": summary.controller_state_sample_count,
                "timelineSampleCount": summary.timeline_sample_count
            },
            "metrics": {
                "collisionCount": summary.collision_count,
                "cycleTimeMs": summary.cycle_time_ms,
                "maxTrackingErrorMm": summary.max_tracking_error_mm,
                "energyEstimateJ": summary.energy_estimate_j
            },
            "job": {
                "jobId": format!("job_{}", id.trim_start_matches("ent_")),
                "status": summary.job_status.clone(),
                "progress": summary.job_progress,
                "phase": summary.job_phase.clone(),
                "progressSamples": run.progress_samples,
                "message": if summary.blocked_sequence_detected {
                    "simulation completed with blocked sequence"
                } else {
                    "simulation completed successfully"
                }
            },
            "report": run.report,
            "timelineSamples": run.timeline_samples,
            "signalSamples": run.signal_samples,
            "controllerStateSamples": run.controller_state_samples,
            "contacts": run.contacts,
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
            "tags": ["safety", "analysis"],
            "robotCellId": robot_cell.id.clone(),
            "parameterSet": {
                "attemptedAction": attempted_action
            },
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

fn sample_perception_frames() -> Vec<PointCloudFrame> {
    vec![
        PointCloudFrame {
            point_count: 1_200,
            coverage_ratio: 0.82,
            timestamp_ms: 0,
            observed_obstacle_count: 2,
        },
        PointCloudFrame {
            point_count: 1_420,
            coverage_ratio: 0.88,
            timestamp_ms: 80,
            observed_obstacle_count: 3,
        },
        PointCloudFrame {
            point_count: 1_510,
            coverage_ratio: 0.91,
            timestamp_ms: 160,
            observed_obstacle_count: 3,
        },
    ]
}

fn sample_nominal_scene_targets() -> Vec<NominalSceneTarget> {
    vec![
        NominalSceneTarget {
            id: "fixture_pick".to_string(),
            label: "Pick fixture".to_string(),
            expected_clearance_mm: 12.0,
        },
        NominalSceneTarget {
            id: "fixture_place".to_string(),
            label: "Place fixture".to_string(),
            expected_clearance_mm: 12.0,
        },
    ]
}

fn build_sensor_rig_entity(
    id: &str,
    name: &str,
) -> Result<(EntityRecord, SensorRigEntitySummary), String> {
    let rig = seeded_sensor_rig(id.to_string(), name.to_string());
    let calibration = calibrate_rig(&rig, 2.5, 0.4).map_err(|error| error.to_string())?;
    let summary = SensorRigEntitySummary {
        sensor_count: rig.mounts.len(),
        lidar_count: rig
            .mounts
            .iter()
            .filter(|mount| {
                matches!(
                    mount.sensor_kind,
                    faero_perception::SensorKind::Lidar2d
                        | faero_perception::SensorKind::Lidar3d
                        | faero_perception::SensorKind::SafetyLidar
                )
            })
            .count(),
        sample_rate_hz: rig
            .mounts
            .iter()
            .map(|mount| mount.sample_rate_hz as f64)
            .fold(0.0, f64::max),
        calibration_status: Some(calibration.status.clone()),
    };
    let entity = sample_entity(
        "SensorRig",
        id,
        name,
        serde_json::json!({
            "baseFrameId": rig.base_frame_id,
            "mounts": serde_json::to_value(&rig.mounts).map_err(|error| error.to_string())?,
            "calibration": serde_json::to_value(&calibration).map_err(|error| error.to_string())?,
            "tags": ["perception", "sensor_rig"],
            "parameterSet": {
                "sensorCount": summary.sensor_count,
                "lidarCount": summary.lidar_count,
                "sampleRateHz": summary.sample_rate_hz
            }
        }),
    );

    Ok((entity, summary))
}

fn build_perception_run_entity(
    id: &str,
    name: &str,
    rig_entity: &EntityRecord,
) -> Result<(EntityRecord, PerceptionRunEntitySummary), String> {
    let rig = seeded_sensor_rig(rig_entity.id.clone(), rig_entity.name.clone());
    let calibration = calibrate_rig(&rig, 2.5, 0.4).map_err(|error| error.to_string())?;
    let run = run_perception(
        &rig,
        &calibration,
        &sample_perception_frames(),
        &sample_nominal_scene_targets(),
    )
    .map_err(|error| error.to_string())?;
    let summary = PerceptionRunEntitySummary {
        status: run.status.clone(),
        frame_count: run.frame_count,
        average_coverage_ratio: run.average_coverage_ratio as f64,
        unknown_obstacle_count: run.unknown_obstacle_count,
        deviation_count: run.comparison.deviation_count,
    };
    let entity = sample_entity(
        "PerceptionRun",
        id,
        name,
        serde_json::json!({
            "rigId": rig_entity.id.clone(),
            "summary": {
                "status": summary.status.clone(),
                "frameCount": summary.frame_count,
                "averageCoverageRatio": summary.average_coverage_ratio,
                "unknownObstacleCount": summary.unknown_obstacle_count,
                "deviationCount": summary.deviation_count
            },
            "job": {
                "jobId": format!("job_{}", id.trim_start_matches("ent_")),
                "status": "completed",
                "progress": 1.0,
                "phase": "compare",
            },
            "frames": serde_json::to_value(sample_perception_frames()).map_err(|error| error.to_string())?,
            "occupancyMap": serde_json::to_value(&run.occupancy_cells).map_err(|error| error.to_string())?,
            "comparison": serde_json::to_value(&run.comparison).map_err(|error| error.to_string())?,
            "progressSamples": serde_json::to_value(&run.progress_samples).map_err(|error| error.to_string())?,
            "tags": ["perception", "artifact"],
            "parameterSet": {
                "frameCount": summary.frame_count,
                "status": summary.status.clone()
            }
        }),
    );

    Ok((entity, summary))
}

fn build_commissioning_session_entity(
    id: &str,
    name: &str,
    perception_run: &EntityRecord,
) -> Result<(EntityRecord, CommissioningSessionEntitySummary), String> {
    let captures = vec![
        CommissioningCapture {
            id: format!("cap_{}_01", id.trim_start_matches("ent_")),
            source: "lidar".to_string(),
            capture_type: "point_cloud".to_string(),
            asset_ref: format!("captures/{}/scan_001.pcd", perception_run.id),
        },
        CommissioningCapture {
            id: format!("cap_{}_02", id.trim_start_matches("ent_")),
            source: "wifi".to_string(),
            capture_type: "network_trace".to_string(),
            asset_ref: format!("captures/{}/trace_001.pcap", perception_run.id),
        },
    ];
    let session = start_commissioning_session(id.to_string(), captures);
    let summary = CommissioningSessionEntitySummary {
        status: session.status.clone(),
        progress_ratio: session.progress_ratio as f64,
        capture_count: session.captures.len(),
        adjustment_count: session.adjustments.len(),
    };
    let entity = sample_entity(
        "CommissioningSession",
        id,
        name,
        serde_json::json!({
            "perceptionRunId": perception_run.id.clone(),
            "captures": serde_json::to_value(&session.captures).map_err(|error| error.to_string())?,
            "adjustments": serde_json::to_value(&session.adjustments).map_err(|error| error.to_string())?,
            "summary": {
                "status": summary.status.clone(),
                "progressRatio": summary.progress_ratio,
                "captureCount": summary.capture_count,
                "adjustmentCount": summary.adjustment_count
            },
            "tags": ["commissioning", "field"],
            "parameterSet": {
                "progressRatio": summary.progress_ratio
            }
        }),
    );

    Ok((entity, summary))
}

fn build_as_built_comparison_entity(
    id: &str,
    name: &str,
    session: &EntityRecord,
) -> Result<(EntityRecord, AsBuiltComparisonEntitySummary), String> {
    let measurements = vec![
        AsBuiltMeasurement {
            id: "m_001".to_string(),
            target_id: "fixture_pick".to_string(),
            deviation_mm: 0.8,
            tolerance_mm: 1.0,
            source_capture_id: "cap_001".to_string(),
        },
        AsBuiltMeasurement {
            id: "m_002".to_string(),
            target_id: "fixture_place".to_string(),
            deviation_mm: 1.7,
            tolerance_mm: 1.0,
            source_capture_id: "cap_002".to_string(),
        },
    ];
    let comparison = compare_as_built(measurements);
    let summary = AsBuiltComparisonEntitySummary {
        accepted_count: comparison.accepted_count,
        rejected_count: comparison.rejected_count,
        average_deviation_mm: comparison.average_deviation_mm as f64,
        max_deviation_mm: comparison.max_deviation_mm as f64,
    };
    let entity = sample_entity(
        "AsBuiltComparison",
        id,
        name,
        serde_json::json!({
            "sessionId": session.id.clone(),
            "measurements": serde_json::to_value(&comparison.measurements).map_err(|error| error.to_string())?,
            "summary": {
                "acceptedCount": summary.accepted_count,
                "rejectedCount": summary.rejected_count,
                "averageDeviationMm": summary.average_deviation_mm,
                "maxDeviationMm": summary.max_deviation_mm
            },
            "tags": ["commissioning", "as_built"],
            "parameterSet": {
                "measurementCount": comparison.measurements.len()
            }
        }),
    );

    Ok((entity, summary))
}

fn build_optimization_study_entity(
    id: &str,
    name: &str,
    comparison: &EntityRecord,
) -> Result<(EntityRecord, OptimizationStudyEntitySummary), String> {
    let mut study = seeded_study(id.to_string());
    let max_deviation_mm = comparison
        .data
        .get("summary")
        .and_then(|summary| summary.get("maxDeviationMm"))
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    if max_deviation_mm > 1.0
        && let Some(variable) = study.variables.first_mut()
    {
        variable.current = 0.78;
    }
    let report = run_study(&study).map_err(|error| error.to_string())?;
    let best_score = report
        .ranked_candidates
        .first()
        .map(|candidate| candidate.score as f64)
        .unwrap_or(0.0);
    let summary = OptimizationStudyEntitySummary {
        candidate_count: report.candidate_count,
        objective_count: study.objectives.len(),
        best_candidate_id: report.best_candidate_id.clone(),
        best_score,
    };
    let entity = sample_entity(
        "OptimizationStudy",
        id,
        name,
        serde_json::json!({
            "comparisonId": comparison.id.clone(),
            "objectives": serde_json::to_value(&study.objectives).map_err(|error| error.to_string())?,
            "constraints": serde_json::to_value(&study.constraints).map_err(|error| error.to_string())?,
            "variables": serde_json::to_value(&study.variables).map_err(|error| error.to_string())?,
            "candidates": serde_json::to_value(&study.candidates).map_err(|error| error.to_string())?,
            "rankedCandidates": serde_json::to_value(&report.ranked_candidates).map_err(|error| error.to_string())?,
            "summary": {
                "candidateCount": summary.candidate_count,
                "objectiveCount": summary.objective_count,
                "bestCandidateId": summary.best_candidate_id.clone(),
                "bestScore": summary.best_score
            },
            "tags": ["optimization", "study"],
            "parameterSet": {
                "candidateCount": summary.candidate_count,
                "objectiveCount": summary.objective_count
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

fn extract_f64_change(
    changes: &serde_json::Map<String, serde_json::Value>,
    path: &str,
) -> Option<f64> {
    changes.get(path).and_then(|value| value.as_f64())
}

fn validate_positive_dimension(label: &str, value: f64) -> Result<(), String> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(format!("{label} doit rester strictement positif"))
    }
}

fn apply_data_changes(
    data: &mut serde_json::Map<String, serde_json::Value>,
    changes: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    for (path, value) in changes {
        if matches!(path.as_str(), "name" | "tags") {
            continue;
        }
        set_json_path(data, path, value.clone())?;
    }
    Ok(())
}

fn merge_generic_parameter_changes(
    data: &mut serde_json::Map<String, serde_json::Value>,
    _existing_data: &serde_json::Value,
    changes: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    apply_data_changes(data, changes)
}

fn set_json_path(
    root: &mut serde_json::Map<String, serde_json::Value>,
    path: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    let segments = path
        .split('.')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return Err("chemin de propriete vide".to_string());
    }

    let mut current = root;
    for segment in &segments[..segments.len() - 1] {
        let entry = current
            .entry((*segment).to_string())
            .or_insert_with(|| serde_json::json!({}));
        if !entry.is_object() {
            *entry = serde_json::json!({});
        }
        current = entry
            .as_object_mut()
            .ok_or_else(|| format!("chemin invalide: {path}"))?;
    }
    current.insert(segments[segments.len() - 1].to_string(), value);
    Ok(())
}

fn default_signal_value_for_kind(kind: &str) -> serde_json::Value {
    match kind {
        "scalar" => serde_json::json!(0.0),
        "text" => serde_json::json!(""),
        _ => serde_json::json!(false),
    }
}

fn validate_entity_change_set(
    entity: &EntityRecord,
    data: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    if let Some(parameters) = data.get("parameterSet").and_then(|value| value.as_object()) {
        for dimension in ["widthMm", "heightMm", "depthMm"] {
            if let Some(value) = parameters.get(dimension).and_then(|value| value.as_f64()) {
                validate_positive_dimension(&format!("parameterSet.{dimension}"), value)?;
            }
        }
    }

    if entity.entity_type == "Signal" {
        let kind = data
            .get("kind")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "le signal doit conserver un kind valide".to_string())?;
        if !matches!(kind, "boolean" | "scalar" | "text") {
            return Err("le kind du signal doit etre boolean, scalar ou text".to_string());
        }
        let current_value = data
            .get("currentValue")
            .cloned()
            .unwrap_or_else(|| default_signal_value_for_kind(kind));
        match kind {
            "boolean" if !current_value.is_boolean() => {
                return Err("currentValue doit rester booleen pour un signal boolean".to_string());
            }
            "scalar" if current_value.as_f64().is_none() => {
                return Err("currentValue doit rester numerique pour un signal scalar".to_string());
            }
            "text" if current_value.as_str().is_none() => {
                return Err("currentValue doit rester texte pour un signal text".to_string());
            }
            _ => {}
        }
    }

    if entity.entity_type == "RobotTarget" {
        let parameters = data
            .get("parameterSet")
            .and_then(|value| value.as_object())
            .ok_or_else(|| "RobotTarget.parameterSet requis".to_string())?;
        let order_index = parameters
            .get("orderIndex")
            .and_then(|value| value.as_u64())
            .ok_or_else(|| {
                "RobotTarget.parameterSet.orderIndex doit rester entier positif".to_string()
            })?;
        if order_index == 0 {
            return Err(
                "RobotTarget.parameterSet.orderIndex doit rester strictement positif".to_string(),
            );
        }
        for coordinate in ["xMm", "yMm", "zMm"] {
            if !parameters
                .get(coordinate)
                .and_then(|value| value.as_f64())
                .is_some_and(f64::is_finite)
            {
                return Err(format!(
                    "RobotTarget.parameterSet.{coordinate} doit rester numerique"
                ));
            }
        }
        let nominal_speed = parameters
            .get("nominalSpeedMmS")
            .and_then(|value| value.as_u64())
            .ok_or_else(|| {
                "RobotTarget.parameterSet.nominalSpeedMmS doit rester entier positif".to_string()
            })?;
        if nominal_speed == 0 {
            return Err(
                "RobotTarget.parameterSet.nominalSpeedMmS doit rester strictement positif"
                    .to_string(),
            );
        }
        parameters
            .get("dwellTimeMs")
            .and_then(|value| value.as_u64())
            .ok_or_else(|| "RobotTarget.parameterSet.dwellTimeMs doit rester entier".to_string())?;
    }

    Ok(())
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

fn assembly_joint_type_label(joint_type: AssemblyJointType) -> &'static str {
    match joint_type {
        AssemblyJointType::Fixed => "fixed",
        AssemblyJointType::Revolute => "revolute",
        AssemblyJointType::Prismatic => "prismatic",
    }
}

fn assembly_joint_state_summary(assembly: &AssemblyData) -> Option<String> {
    let joint = assembly.joints.first()?;
    let limits = joint
        .limits
        .map(|limits| format!(" [{:.2}, {:.2}]", limits.min, limits.max))
        .unwrap_or_default();
    Some(format!(
        "{} {} @ {:.2}{}",
        assembly_joint_type_label(joint.joint_type),
        joint.id,
        joint.current_position,
        limits
    ))
}

fn assembly_summary_from_entity(entity: &EntityRecord) -> Option<AssemblyEntitySummary> {
    if entity.entity_type != "Assembly" {
        return None;
    }

    let assembly = serde_json::from_value::<AssemblyData>(entity.data.clone()).ok()?;
    let solve_report = assembly.solve_report.as_ref()?;
    Some(AssemblyEntitySummary {
        status: assembly_solve_status_label(solve_report.status).to_string(),
        occurrence_count: assembly.occurrences.len(),
        mate_count: assembly.mate_constraints.len(),
        joint_count: assembly.joints.len(),
        joint_state_summary: assembly_joint_state_summary(&assembly),
        degrees_of_freedom_estimate: solve_report.degrees_of_freedom_estimate,
        warning_count: solve_report.warnings.len(),
    })
}

fn robot_cell_summary_from_entity(entity: &EntityRecord) -> Option<RobotCellEntitySummary> {
    if entity.entity_type != "RobotCell" {
        return None;
    }

    let validation = entity.data.get("sequenceValidation")?;
    let safety = entity.data.get("safety")?;
    let control = entity.data.get("control");
    Some(RobotCellEntitySummary {
        scene_assembly_id: entity
            .data
            .get("sceneAssemblyId")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        target_preview: entity
            .data
            .get("targetPreview")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        target_count: validation.get("targetCount")?.as_u64()? as usize,
        path_length_mm: number_from_value(validation, "pathLengthMm")?,
        max_segment_mm: number_from_value(validation, "maxSegmentMm")?,
        estimated_cycle_time_ms: validation.get("estimatedCycleTimeMs")?.as_u64()? as u32,
        equipment_count: entity
            .data
            .get("equipmentIds")
            .and_then(|value| value.as_array())
            .map(|entries| entries.len())
            .unwrap_or(0),
        sequence_count: entity
            .data
            .get("sequenceIds")
            .and_then(|value| value.as_array())
            .map(|entries| entries.len())
            .unwrap_or(0),
        safety_zone_count: safety.get("zoneCount")?.as_u64()? as usize,
        signal_count: control
            .and_then(|value| value.get("signalCount"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0) as usize,
        controller_transition_count: control
            .and_then(|value| value.get("controllerTransitionCount"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0) as usize,
        blocked_sequence_detected: control
            .and_then(|value| value.get("blockedSequenceDetected"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        blocked_state_id: control
            .and_then(|value| value.get("blockedStateId"))
            .and_then(|value| value.as_str())
            .map(str::to_string),
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
    let metrics = entity.data.get("metrics").unwrap_or(summary);
    let job = entity.data.get("job");
    Some(SimulationRunEntitySummary {
        status: string_from_value(summary, "status")?,
        collision_count: metrics.get("collisionCount")?.as_u64()? as u32,
        cycle_time_ms: metrics.get("cycleTimeMs")?.as_u64()? as u32,
        max_tracking_error_mm: number_from_value(metrics, "maxTrackingErrorMm")?,
        energy_estimate_j: number_from_value(metrics, "energyEstimateJ")?,
        blocked_sequence_detected: summary
            .get("blockedSequenceDetected")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
        blocked_state_id: summary
            .get("blockedStateId")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        contact_count: summary
            .get("contactCount")
            .and_then(|value| value.as_u64())
            .unwrap_or_else(|| {
                entity
                    .data
                    .get("contacts")
                    .and_then(|value| value.as_array())
                    .map(|entries| entries.len() as u64)
                    .unwrap_or(0)
            }) as usize,
        signal_sample_count: summary
            .get("signalSampleCount")
            .and_then(|value| value.as_u64())
            .unwrap_or_else(|| {
                entity
                    .data
                    .get("signalSamples")
                    .and_then(|value| value.as_array())
                    .map(|entries| entries.len() as u64)
                    .unwrap_or(0)
            }) as usize,
        controller_state_sample_count: summary
            .get("controllerStateSampleCount")
            .and_then(|value| value.as_u64())
            .unwrap_or_else(|| {
                entity
                    .data
                    .get("controllerStateSamples")
                    .and_then(|value| value.as_array())
                    .map(|entries| entries.len() as u64)
                    .unwrap_or(0)
            }) as usize,
        timeline_sample_count: summary
            .get("timelineSampleCount")
            .and_then(|value| value.as_u64())
            .unwrap_or_else(|| {
                entity
                    .data
                    .get("timelineSamples")
                    .and_then(|value| value.as_array())
                    .map(|entries| entries.len() as u64)
                    .unwrap_or(0)
            }) as usize,
        job_status: job
            .and_then(|value| value.get("status"))
            .and_then(|value| value.as_str())
            .unwrap_or("completed")
            .to_string(),
        job_phase: job
            .and_then(|value| value.get("phase"))
            .and_then(|value| value.as_str())
            .unwrap_or("completed")
            .to_string(),
        job_progress: job
            .and_then(|value| value.get("progress"))
            .and_then(|value| value.as_f64())
            .unwrap_or(1.0),
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

fn sensor_rig_summary_from_entity(entity: &EntityRecord) -> Option<SensorRigEntitySummary> {
    if entity.entity_type != "SensorRig" {
        return None;
    }

    let mounts = entity.data.get("mounts")?.as_array()?;
    let sample_rate_hz = mounts
        .iter()
        .filter_map(|mount| mount.get("sampleRateHz").and_then(|value| value.as_f64()))
        .max_by(f64::total_cmp)
        .unwrap_or(0.0);
    Some(SensorRigEntitySummary {
        sensor_count: mounts.len(),
        lidar_count: mounts
            .iter()
            .filter(|mount| {
                mount
                    .get("sensorKind")
                    .and_then(|value| value.as_str())
                    .is_some_and(|kind| kind.contains("lidar"))
            })
            .count(),
        sample_rate_hz,
        calibration_status: entity
            .data
            .get("calibration")
            .and_then(|value| value.get("status"))
            .and_then(|value| value.as_str())
            .map(str::to_string),
    })
}

fn perception_run_summary_from_entity(entity: &EntityRecord) -> Option<PerceptionRunEntitySummary> {
    if entity.entity_type != "PerceptionRun" {
        return None;
    }

    let summary = entity.data.get("summary")?;
    Some(PerceptionRunEntitySummary {
        status: string_from_value(summary, "status")?,
        frame_count: summary.get("frameCount")?.as_u64()? as usize,
        average_coverage_ratio: number_from_value(summary, "averageCoverageRatio")?,
        unknown_obstacle_count: summary.get("unknownObstacleCount")?.as_u64()? as u32,
        deviation_count: summary.get("deviationCount")?.as_u64()? as usize,
    })
}

fn commissioning_session_summary_from_entity(
    entity: &EntityRecord,
) -> Option<CommissioningSessionEntitySummary> {
    if entity.entity_type != "CommissioningSession" {
        return None;
    }

    let summary = entity.data.get("summary")?;
    Some(CommissioningSessionEntitySummary {
        status: string_from_value(summary, "status")?,
        progress_ratio: number_from_value(summary, "progressRatio")?,
        capture_count: summary.get("captureCount")?.as_u64()? as usize,
        adjustment_count: summary.get("adjustmentCount")?.as_u64()? as usize,
    })
}

fn as_built_comparison_summary_from_entity(
    entity: &EntityRecord,
) -> Option<AsBuiltComparisonEntitySummary> {
    if entity.entity_type != "AsBuiltComparison" {
        return None;
    }

    let summary = entity.data.get("summary")?;
    Some(AsBuiltComparisonEntitySummary {
        accepted_count: summary.get("acceptedCount")?.as_u64()? as usize,
        rejected_count: summary.get("rejectedCount")?.as_u64()? as usize,
        average_deviation_mm: number_from_value(summary, "averageDeviationMm")?,
        max_deviation_mm: number_from_value(summary, "maxDeviationMm")?,
    })
}

fn optimization_study_summary_from_entity(
    entity: &EntityRecord,
) -> Option<OptimizationStudyEntitySummary> {
    if entity.entity_type != "OptimizationStudy" {
        return None;
    }

    let summary = entity.data.get("summary")?;
    Some(OptimizationStudyEntitySummary {
        candidate_count: summary.get("candidateCount")?.as_u64()? as usize,
        objective_count: summary.get("objectiveCount")?.as_u64()? as usize,
        best_candidate_id: summary
            .get("bestCandidateId")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        best_score: number_from_value(summary, "bestScore")?,
    })
}

fn format_signal_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn generic_entity_detail(entity: &EntityRecord) -> Option<String> {
    match entity.entity_type.as_str() {
        "Signal" => Some(format!(
            "{} | {}",
            entity
                .data
                .get("signalId")
                .and_then(|value| value.as_str())
                .unwrap_or("signal"),
            format_signal_value(
                entity
                    .data
                    .get("currentValue")
                    .unwrap_or(&serde_json::Value::Null)
            )
        )),
        "ControllerModel" => Some(format!(
            "{} states | {} transitions",
            entity
                .data
                .get("parameterSet")
                .and_then(|value| value.get("stateCount"))
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            entity
                .data
                .get("parameterSet")
                .and_then(|value| value.get("transitionCount"))
                .and_then(|value| value.as_u64())
                .unwrap_or(0)
        )),
        "RobotTarget" => robot_target_model_from_entity(entity)
            .map(|model| format_robot_target_entity_detail(&model)),
        "AiSession" => Some(format!(
            "{} | {} suggestion(s)",
            entity
                .data
                .get("mode")
                .and_then(|value| value.as_str())
                .unwrap_or("chat"),
            entity
                .data
                .get("createdSuggestionIds")
                .and_then(|value| value.as_array())
                .map(|entries| entries.len())
                .unwrap_or(0)
        )),
        "AiSuggestion" => Some(format!(
            "{} | conf {}",
            entity
                .data
                .get("riskLevel")
                .and_then(|value| value.as_str())
                .unwrap_or("low"),
            entity
                .data
                .get("confidence")
                .and_then(|value| value.as_f64())
                .unwrap_or(0.0)
        )),
        "IndustrialReplay" => Some(format!(
            "{} sample(s) | latency {:?} ms",
            entity
                .data
                .get("summary")
                .and_then(|value| value.get("sampleCount"))
                .and_then(|value| value.as_u64())
                .unwrap_or(0),
            entity
                .data
                .get("summary")
                .and_then(|value| value.get("latencyMs"))
                .and_then(|value| value.as_u64())
        )),
        _ => None,
    }
}

fn format_part_entity_detail(summary: &PartGeometrySummary) -> String {
    format!(
        "{:.1} x {:.1} x {:.1} mm | {:.1} g",
        summary.width_mm, summary.height_mm, summary.depth_mm, summary.estimated_mass_grams
    )
}

fn format_assembly_entity_detail(summary: &AssemblyEntitySummary) -> String {
    let joint_summary = summary
        .joint_state_summary
        .as_deref()
        .unwrap_or("no joint state");
    format!(
        "{} | {} occ | {} mates | {} joints | {} | {} ddl",
        summary.status,
        summary.occurrence_count,
        summary.mate_count,
        summary.joint_count,
        joint_summary,
        summary.degrees_of_freedom_estimate
    )
}

fn format_robot_cell_entity_detail(summary: &RobotCellEntitySummary) -> String {
    let scene = summary.scene_assembly_id.as_deref().unwrap_or("scene");
    let preview = summary
        .target_preview
        .as_deref()
        .map(|value| format!(" | {value}"))
        .unwrap_or_default();
    let blocked = if summary.blocked_sequence_detected {
        format!(
            " | blocked {}",
            summary.blocked_state_id.as_deref().unwrap_or("state")
        )
    } else {
        String::new()
    };
    format!(
        "{} pts | {} equip | {} sig | {} tr | {} | {} ms{}{}",
        summary.target_count,
        summary.equipment_count,
        summary.signal_count,
        summary.controller_transition_count,
        scene,
        summary.estimated_cycle_time_ms,
        blocked,
        preview
    )
}

fn format_simulation_run_entity_detail(summary: &SimulationRunEntitySummary) -> String {
    format!(
        "{} | {} {:.0}% | {} ms | {} coll | {} contact",
        summary.status,
        summary.job_phase,
        summary.job_progress * 100.0,
        summary.cycle_time_ms,
        summary.collision_count,
        summary.contact_count
    )
}

fn format_safety_report_entity_detail(summary: &SafetyReportEntitySummary) -> String {
    format!(
        "{} | {} active | {} block",
        summary.status, summary.active_zone_count, summary.blocking_interlock_count
    )
}

fn format_sensor_rig_entity_detail(summary: &SensorRigEntitySummary) -> String {
    format!(
        "{} sensor(s) | {} lidar | {} Hz",
        summary.sensor_count, summary.lidar_count, summary.sample_rate_hz
    )
}

fn format_perception_run_entity_detail(summary: &PerceptionRunEntitySummary) -> String {
    format!(
        "{} | {} frame(s) | {} unknown",
        summary.status, summary.frame_count, summary.unknown_obstacle_count
    )
}

fn format_commissioning_session_entity_detail(
    summary: &CommissioningSessionEntitySummary,
) -> String {
    format!(
        "{} | {:.0}% | {} capture(s)",
        summary.status,
        summary.progress_ratio * 100.0,
        summary.capture_count
    )
}

fn format_as_built_comparison_entity_detail(summary: &AsBuiltComparisonEntitySummary) -> String {
    format!(
        "{} ok | {} ko | {:.2} mm",
        summary.accepted_count, summary.rejected_count, summary.max_deviation_mm
    )
}

fn format_optimization_study_entity_detail(summary: &OptimizationStudyEntitySummary) -> String {
    format!(
        "{} obj | {} cand | {:.2}",
        summary.objective_count, summary.candidate_count, summary.best_score
    )
}

fn sample_endpoint_with_stream(index: usize) -> (ExternalEndpoint, TelemetryStream) {
    let endpoint_kind = (index - 1) % 6;
    let mut endpoint = match endpoint_kind {
        0 => stub_ros2_endpoint(),
        1 => stub_opcua_endpoint(),
        2 => stub_plc_endpoint(),
        3 => stub_robot_controller_endpoint(),
        4 => stub_wifi_endpoint(),
        _ => stub_bluetooth_endpoint(),
    };

    let endpoint_prefix = match &endpoint.endpoint_type {
        faero_types::EndpointType::Ros2 => "ros2",
        faero_types::EndpointType::Opcua => "opcua",
        faero_types::EndpointType::Plc => "plc",
        faero_types::EndpointType::RobotController => "robot",
        faero_types::EndpointType::WifiDevice => "wifi",
        _ => "ble",
    };
    endpoint.id = format!("ext_{endpoint_prefix}_{index:03}");
    endpoint.name = format!("{} {index:03}", endpoint.name);
    if endpoint.addressing.host.is_some() {
        endpoint.addressing.host = Some(format!("{endpoint_prefix}-{index:03}.local"));
    }
    if endpoint.addressing.device_id.is_some() {
        endpoint.addressing.device_id = Some(format!("{endpoint_prefix}-device-{index:03}"));
    }
    endpoint.signal_map_ids = vec![format!("sig_{endpoint_prefix}_{index:03}")];

    let stream = TelemetryStream {
        id: format!("str_{endpoint_prefix}_{index:03}"),
        name: format!("Telemetry-{endpoint_prefix}-{index:03}"),
        endpoint_id: endpoint.id.clone(),
        stream_type: match &endpoint.endpoint_type {
            faero_types::EndpointType::Ros2 => "ros2_topic".to_string(),
            faero_types::EndpointType::Opcua => "opcua_subscription".to_string(),
            faero_types::EndpointType::Plc => "plc_tag".to_string(),
            faero_types::EndpointType::RobotController => "robot_status".to_string(),
            faero_types::EndpointType::WifiDevice => "mqtt_topic".to_string(),
            _ => "gatt_characteristic".to_string(),
        },
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
        release_channel: "stable".to_string(),
        capabilities: vec!["panel".to_string(), "command".to_string()],
        permissions: vec!["project.read".to_string(), "plugin.ui.mount".to_string()],
        contributions: vec![
            PluginContribution {
                kind: "panel".to_string(),
                target: "workspace.right".to_string(),
                title: "Desktop Runtime".to_string(),
            },
            PluginContribution {
                kind: "command".to_string(),
                target: "project.export".to_string(),
                title: "Project Export Hook".to_string(),
            },
        ],
        entrypoints: vec!["plugins/desktop-runtime/index.js".to_string()],
        compatibility: vec!["faero-core@0.1".to_string()],
        signature: Some(format!("sha256:plugin-{index:03}")),
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
fn workspace_update_entity_properties(
    payload: EntityPropertyUpdateInput,
    state: State<'_, SharedWorkspace>,
) -> Result<CommandResponse, String> {
    let mut session = lock_workspace(&state)?;
    let result = session.update_entity_properties(&payload)?;
    Ok(CommandResponse {
        snapshot: session.snapshot(),
        result,
    })
}

#[tauri::command]
fn workspace_apply_ai_suggestion(
    suggestion_id: String,
    state: State<'_, SharedWorkspace>,
) -> Result<CommandResponse, String> {
    let mut session = lock_workspace(&state)?;
    let result = session.apply_ai_suggestion(&suggestion_id)?;
    Ok(CommandResponse {
        snapshot: session.snapshot(),
        result,
    })
}

#[tauri::command]
fn workspace_reject_ai_suggestion(
    suggestion_id: String,
    state: State<'_, SharedWorkspace>,
) -> Result<CommandResponse, String> {
    let mut session = lock_workspace(&state)?;
    let result = session.reject_ai_suggestion(&suggestion_id)?;
    Ok(CommandResponse {
        snapshot: session.snapshot(),
        result,
    })
}

#[tauri::command]
fn ai_runtime_status(selected_profile: Option<String>) -> AiRuntimeStatus {
    query_ai_runtime_status(selected_profile.as_deref())
}

#[tauri::command]
fn ai_chat_send_message(
    message: String,
    locale: String,
    history: Vec<AiConversationMessage>,
    selected_model: Option<String>,
    selected_profile: Option<String>,
    state: State<'_, SharedWorkspace>,
) -> Result<AiChatResponse, String> {
    let document = {
        let session = lock_workspace(&state)?;
        session.graph.document().clone()
    };
    let mut response = chat_with_project(
        &document,
        &locale,
        &history,
        &message,
        selected_model.as_deref(),
        selected_profile.as_deref(),
    )
    .map_err(|error| error.to_string())?;

    {
        let mut session = lock_workspace(&state)?;
        let suggestion_id = session.record_ai_response(&message, &response)?;
        response.suggestion_id = Some(suggestion_id);
    }

    Ok(response)
}

fn build_menu_item<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    text: &str,
    accelerator: Option<&str>,
) -> tauri::Result<tauri::menu::MenuItem<R>> {
    let builder = if let Some(accelerator) = accelerator {
        MenuItemBuilder::with_id(id, text).accelerator(accelerator)
    } else {
        MenuItemBuilder::with_id(id, text)
    };
    builder.build(app)
}

fn build_native_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<tauri::menu::Menu<R>> {
    let file_new = build_menu_item(
        app,
        "project.create",
        "New Project",
        Some("CmdOrCtrl+Shift+N"),
    )?;
    let file_open = build_menu_item(app, "project.open", "Open Project", Some("CmdOrCtrl+O"))?;
    let file_recent = build_menu_item(app, "project.open_recent", "Open Recent", None)?;
    let file_save = build_menu_item(app, "project.save", "Save", Some("CmdOrCtrl+S"))?;
    let file_save_all = build_menu_item(
        app,
        "project.save_all",
        "Save All",
        Some("CmdOrCtrl+Shift+S"),
    )?;
    let file_import = build_menu_item(app, "project.import", "Import", None)?;
    let file_export = build_menu_item(app, "project.export", "Export", None)?;
    let file_settings = build_menu_item(app, "app.settings", "Settings", None)?;
    let file_exit = build_menu_item(app, "app.exit", "Exit", None)?;
    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&file_new)
        .item(&file_open)
        .item(&file_recent)
        .separator()
        .item(&file_save)
        .item(&file_save_all)
        .separator()
        .item(&file_import)
        .item(&file_export)
        .separator()
        .item(&file_settings)
        .item(&file_exit)
        .build()?;

    let edit_undo = build_menu_item(app, "history.undo", "Undo", Some("CmdOrCtrl+Z"))?;
    let edit_redo = build_menu_item(app, "history.redo", "Redo", Some("CmdOrCtrl+Y"))?;
    let edit_cut = build_menu_item(app, "selection.cut", "Cut", Some("CmdOrCtrl+X"))?;
    let edit_copy = build_menu_item(app, "selection.copy", "Copy", Some("CmdOrCtrl+C"))?;
    let edit_paste = build_menu_item(app, "selection.paste", "Paste", Some("CmdOrCtrl+V"))?;
    let edit_delete = build_menu_item(app, "selection.delete", "Delete", Some("Delete"))?;
    let edit_find = build_menu_item(app, "workspace.find", "Find", Some("CmdOrCtrl+F"))?;
    let edit_palette = build_menu_item(
        app,
        "workspace.command_palette",
        "Command Palette",
        Some("CmdOrCtrl+Shift+P"),
    )?;
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&edit_undo)
        .item(&edit_redo)
        .separator()
        .item(&edit_cut)
        .item(&edit_copy)
        .item(&edit_paste)
        .item(&edit_delete)
        .separator()
        .item(&edit_find)
        .item(&edit_palette)
        .build()?;

    let view_project_explorer =
        build_menu_item(app, "view.project_explorer", "Project Explorer", None)?;
    let view_properties = build_menu_item(app, "view.properties", "Properties", Some("F4"))?;
    let view_output = build_menu_item(app, "view.output", "Output", None)?;
    let view_problems = build_menu_item(app, "view.problems", "Problems", None)?;
    let view_ai = build_menu_item(app, "view.ai_assistant", "AI Assistant", None)?;
    let view_viewport = build_menu_item(app, "view.viewport_3d", "3D Viewport", None)?;
    let view_timeline =
        build_menu_item(app, "view.simulation_timeline", "Simulation Timeline", None)?;
    let view_telemetry = build_menu_item(app, "view.telemetry_monitor", "Telemetry Monitor", None)?;
    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&view_project_explorer)
        .item(&view_properties)
        .item(&view_output)
        .item(&view_problems)
        .item(&view_ai)
        .separator()
        .item(&view_viewport)
        .item(&view_timeline)
        .item(&view_telemetry)
        .build()?;

    let insert_part = build_menu_item(app, "entity.create.part", "Add Part", None)?;
    let insert_assembly = build_menu_item(app, "entity.create.assembly", "Add Assembly", None)?;
    let insert_cell = build_menu_item(app, "entity.create.robot_cell", "Add Robot Cell", None)?;
    let insert_sensor = build_menu_item(app, "entity.create.sensor_rig", "Add Sensor Rig", None)?;
    let insert_endpoint = build_menu_item(
        app,
        "entity.create.external_endpoint",
        "Add External Endpoint",
        None,
    )?;
    let insert_properties = build_menu_item(
        app,
        "project.properties",
        "Project Properties",
        Some("Alt+Enter"),
    )?;
    let insert_menu = SubmenuBuilder::new(app, "Insert")
        .item(&insert_part)
        .item(&insert_assembly)
        .item(&insert_cell)
        .item(&insert_sensor)
        .item(&insert_endpoint)
        .separator()
        .item(&insert_properties)
        .build()?;

    let simulation_start =
        build_menu_item(app, "simulation.run.start", "Start Simulation", Some("F5"))?;
    let simulation_stop = build_menu_item(
        app,
        "simulation.run.cancel",
        "Stop Simulation",
        Some("Shift+F5"),
    )?;
    let simulation_step = build_menu_item(
        app,
        "simulation.timeline.step",
        "Step Simulation",
        Some("F10"),
    )?;
    let simulation_regenerate = build_menu_item(
        app,
        "build.regenerate_part",
        "Rebuild Geometry",
        Some("CmdOrCtrl+B"),
    )?;
    let simulation_safety = build_menu_item(app, "analyze.safety", "Safety Analysis", None)?;
    let simulation_perception =
        build_menu_item(app, "perception.run.start", "Run Perception", None)?;
    let simulation_replay = build_menu_item(
        app,
        "integration.replay.degraded",
        "Replay Degraded Link",
        None,
    )?;
    let simulation_commissioning = build_menu_item(
        app,
        "commissioning.session.start",
        "Start Commissioning",
        None,
    )?;
    let simulation_as_built = build_menu_item(
        app,
        "commissioning.compare.as_built",
        "Compare As-Built",
        None,
    )?;
    let simulation_optimization =
        build_menu_item(app, "optimization.run.start", "Run Optimization", None)?;
    let simulation_menu = SubmenuBuilder::new(app, "Simulation")
        .item(&simulation_start)
        .item(&simulation_stop)
        .item(&simulation_step)
        .separator()
        .item(&simulation_regenerate)
        .item(&simulation_safety)
        .separator()
        .item(&simulation_perception)
        .item(&simulation_replay)
        .item(&simulation_commissioning)
        .item(&simulation_as_built)
        .item(&simulation_optimization)
        .build()?;

    let ai_focus = build_menu_item(
        app,
        "ai.focus_input",
        "Focus Local AI Chat",
        Some("Ctrl+Space"),
    )?;
    let ai_panel = build_menu_item(app, "ai.show_panel", "Show AI Assistant", None)?;
    let ai_explain = build_menu_item(app, "ai.deep_explain.request", "AI Deep Explain", None)?;
    let ai_menu = SubmenuBuilder::new(app, "AI")
        .item(&ai_focus)
        .item(&ai_panel)
        .item(&ai_explain)
        .build()?;

    let help_docs = build_menu_item(app, "help.documentation", "Documentation", None)?;
    let help_openspec = build_menu_item(app, "help.openspec", "OpenSpec", None)?;
    let help_shortcuts =
        build_menu_item(app, "help.keyboard_shortcuts", "Keyboard Shortcuts", None)?;
    let help_about = build_menu_item(app, "help.about", "About FutureAero", None)?;
    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&help_docs)
        .item(&help_openspec)
        .item(&help_shortcuts)
        .item(&help_about)
        .build()?;

    MenuBuilder::new(app)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&insert_menu)
        .item(&simulation_menu)
        .item(&ai_menu)
        .item(&help_menu)
        .build()
}

fn main() {
    let workspace = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
        .unwrap_or_else(|_| WorkspaceSession::empty("FutureAero Session"));

    tauri::Builder::default()
        .manage(Mutex::new(workspace))
        .menu(build_native_menu)
        .on_menu_event(|app, event| {
            let _ = app.emit(
                NATIVE_MENU_EVENT_NAME,
                NativeMenuCommandPayload {
                    command_id: event.id().as_ref().to_string(),
                },
            );
        })
        .invoke_handler(tauri::generate_handler![
            backend_status,
            available_fixture_projects,
            load_fixture_project,
            load_project_snapshot,
            workspace_bootstrap,
            workspace_load_fixture,
            workspace_execute_command,
            workspace_regenerate_latest_part,
            workspace_update_entity_properties,
            workspace_apply_ai_suggestion,
            workspace_reject_ai_suggestion,
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
        assert_eq!(snapshot.open_spec_documents.len(), 2);
        assert_eq!(snapshot.endpoints[0].transport_kind, "robot_controller");
        assert!(snapshot.plugins[0].enabled);
        assert_eq!(snapshot.open_spec_documents[0].kind, "design_intent");
        assert_eq!(snapshot.open_spec_documents[0].excerpt, "Intent");
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
        assert_eq!(summary.joint_count, 1);
        assert!(
            summary
                .joint_state_summary
                .as_deref()
                .is_some_and(|value| value.contains("revolute"))
        );
        assert!(
            assembly.data["occurrences"]
                .as_array()
                .is_some_and(|items| items.len() >= 2)
        );
        assert!(
            assembly.data["joints"]
                .as_array()
                .is_some_and(|items| items.len() == 1)
        );
        assert!(
            assembly.data["mateConstraints"]
                .as_array()
                .is_some_and(|items| !items.is_empty())
        );
        assert_eq!(assembly.data["solveReport"]["status"], "solved");
        assert_eq!(assembly.data["joints"][0]["jointType"], "revolute");
        assert_eq!(assembly.data["joints"][0]["currentPosition"], 0.35);
        assert!(
            assembly
                .detail
                .as_ref()
                .is_some_and(|detail| detail.contains("joint_001"))
        );
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| { entry.channel == "command" && entry.kind == "assembly.mate.add" })
        );
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| { entry.channel == "command" && entry.kind == "joint.create" })
        );
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| { entry.channel == "command" && entry.kind == "joint.state.set" })
        );
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| { entry.channel == "event" && entry.kind == "joint.state.changed" })
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
        assert_eq!(
            summary.target_preview.as_deref(),
            Some("pick -> transfer -> place")
        );
        assert_eq!(summary.equipment_count, 3);
        assert_eq!(summary.sequence_count, 1);
        assert_eq!(summary.signal_count, 5);
        assert_eq!(summary.controller_transition_count, 3);
        assert!(summary.blocked_sequence_detected);
        assert_eq!(summary.blocked_state_id.as_deref(), Some("idle"));
        assert_eq!(
            summary.scene_assembly_id.as_deref(),
            Some("ent_asm_cell_001")
        );
        assert_eq!(robot_cell.data["sceneAssemblyId"], "ent_asm_cell_001");
        assert_eq!(
            robot_cell.data["equipmentIds"]
                .as_array()
                .map(|entries| entries.len()),
            Some(3)
        );
        assert_eq!(
            robot_cell.data["sequenceIds"]
                .as_array()
                .map(|entries| entries.len()),
            Some(1)
        );
        assert_eq!(
            robot_cell.data["targetIds"]
                .as_array()
                .map(|entries| entries.len()),
            Some(3)
        );
        assert_eq!(robot_cell.data["control"]["signalCount"], 5);
        assert_eq!(robot_cell.data["control"]["controllerTransitionCount"], 3);
        assert_eq!(robot_cell.data["control"]["blockedSequenceDetected"], true);
        assert_eq!(robot_cell.data["control"]["blockedStateId"], "idle");
        assert_eq!(
            robot_cell.data["control"]["states"]
                .as_array()
                .map(|entries| entries.len()),
            Some(4)
        );
        assert_eq!(
            robot_cell.data["control"]["transitions"]
                .as_array()
                .map(|entries| entries.len()),
            Some(3)
        );
        assert_eq!(
            robot_cell.data["targetPreview"],
            "pick -> transfer -> place"
        );
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.entity_type == "Assembly" && entity.id == "ent_asm_cell_001")
        );
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.entity_type == "RobotModel" && entity.id == "ent_robot_001")
        );
        assert_eq!(
            snapshot
                .entities
                .iter()
                .filter(|entity| entity.entity_type == "EquipmentModel")
                .count(),
            3
        );
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.entity_type == "RobotSequence" && entity.id == "ent_seq_001")
        );
        assert_eq!(
            snapshot
                .entities
                .iter()
                .filter(|entity| entity.entity_type == "RobotTarget")
                .count(),
            3
        );
        assert_eq!(
            snapshot
                .entities
                .iter()
                .filter(|entity| entity.entity_type == "SafetyZoneModel")
                .count(),
            2
        );
        assert!(
            robot_cell
                .detail
                .as_ref()
                .is_some_and(|detail| detail.contains("equip"))
        );
        assert!(snapshot.recent_activity.iter().any(|entry| {
            entry.channel == "event"
                && entry.kind == "entity.created"
                && entry.target_id.as_deref() == Some("ent_ctrl_001")
        }));
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| { entry.channel == "event" && entry.kind == "assembly.solved" })
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
        assert_eq!(summary.job_status, "completed");
        assert_eq!(summary.job_phase, "completed");
        assert_eq!(summary.job_progress, 1.0);
        assert_eq!(
            simulation
                .data
                .get("scenario")
                .and_then(|value| value.get("seed"))
                .and_then(|value| value.as_u64()),
            Some(308)
        );
        assert_eq!(
            simulation
                .data
                .get("scenario")
                .and_then(|value| value.get("stepCount"))
                .and_then(|value| value.as_u64()),
            Some(12)
        );
        assert_eq!(
            simulation
                .data
                .get("metrics")
                .and_then(|value| value.get("cycleTimeMs"))
                .and_then(|value| value.as_u64()),
            Some(summary.cycle_time_ms as u64)
        );
        assert_eq!(
            simulation
                .data
                .get("job")
                .and_then(|value| value.get("progressSamples"))
                .and_then(|value| value.as_array())
                .map(|samples| samples.len()),
            Some(4)
        );
        assert_eq!(
            simulation
                .data
                .get("report")
                .and_then(|value| value.get("status"))
                .and_then(|value| value.as_str()),
            Some("completed")
        );
        assert!(
            simulation
                .data
                .get("report")
                .and_then(|value| value.get("headline"))
                .and_then(|value| value.as_str())
                .is_some_and(|headline| headline.contains("Run nominal"))
        );
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| entry.kind == "simulation.run.queued")
        );
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| entry.kind == "simulation.run.trace_persisted")
        );
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
    fn help_openspec_reports_readable_documents_and_logs_activity() {
        let mut session = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
            .expect("fixture-backed session should load");

        let result = session
            .execute_command("help.openspec")
            .expect("help openspec should succeed");
        let snapshot = session.snapshot();

        assert_eq!(result.status, "applied");
        assert!(result.message.contains("document(s) OpenSpec"));
        assert_eq!(snapshot.open_spec_documents.len(), 2);
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| entry.kind == "help.openspec.reviewed")
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
        assert_eq!(
            reloaded.open_spec_documents.len(),
            document.open_spec_documents.len()
        );
        assert!(output_path.exists());

        fs::remove_dir_all(output_path).expect("saved test artifact should be removable");
    }

    #[test]
    fn simulation_run_round_trips_with_job_metrics_and_progress_samples() {
        let mut session = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
            .expect("fixture-backed session should load");
        session
            .execute_command("simulation.run.start")
            .expect("simulation command should run");

        let output_path = save_document_copy(
            "test-saves",
            "fixture:simulation-run",
            session.graph.document(),
        )
        .expect("save should work");
        let reloaded =
            faero_storage::load_project(&output_path).expect("saved project should reload");
        let simulation = reloaded
            .nodes
            .values()
            .find(|entity| entity.entity_type == "SimulationRun")
            .expect("simulation run should persist");

        assert_eq!(
            simulation
                .data
                .get("scenario")
                .and_then(|value| value.get("engineVersion"))
                .and_then(|value| value.as_str()),
            Some("faero-sim@0.2.0")
        );
        assert_eq!(
            simulation
                .data
                .get("metrics")
                .and_then(|value| value.get("collisionCount"))
                .and_then(|value| value.as_u64()),
            Some(0)
        );
        assert_eq!(
            simulation
                .data
                .get("job")
                .and_then(|value| value.get("progressSamples"))
                .and_then(|value| value.as_array())
                .map(|samples| samples.len()),
            Some(4)
        );
        assert_eq!(
            simulation
                .data
                .get("report")
                .and_then(|value| value.get("status"))
                .and_then(|value| value.as_str()),
            Some("completed")
        );
        assert!(
            simulation
                .data
                .get("report")
                .and_then(|value| value.get("recommendedActions"))
                .and_then(|value| value.as_array())
                .is_some_and(|actions| !actions.is_empty())
        );

        fs::remove_dir_all(output_path).expect("saved test artifact should be removable");
    }

    #[test]
    fn simulation_run_can_persist_localized_collision_contacts_and_report() {
        let mut session = WorkspaceSession::load_fixture(DEFAULT_FIXTURE_ID)
            .expect("fixture-backed session should load");

        session
            .execute_command("entity.create.external_endpoint")
            .expect("first endpoint should be created");
        session
            .execute_command("entity.create.external_endpoint")
            .expect("second endpoint should be created");
        session
            .execute_command("simulation.run.start")
            .expect("simulation command should run");

        let simulation = session
            .snapshot()
            .entities
            .into_iter()
            .find(|entity| entity.entity_type == "SimulationRun")
            .expect("simulation run should exist");
        let contacts = simulation
            .data
            .get("contacts")
            .and_then(|value| value.as_array())
            .expect("contacts should persist");
        let first_contact = contacts
            .first()
            .expect("collision run should expose contacts");

        assert_eq!(
            simulation
                .data
                .get("report")
                .and_then(|value| value.get("status"))
                .and_then(|value| value.as_str()),
            Some("collided")
        );
        assert!(
            simulation
                .data
                .get("report")
                .and_then(|value| value.get("criticalEventIds"))
                .and_then(|value| value.as_array())
                .is_some_and(|ids| !ids.is_empty())
        );
        assert!(
            first_contact
                .get("locationLabel")
                .and_then(|value| value.as_str())
                .is_some_and(|label| label.contains("pair_"))
        );
        assert_eq!(
            first_contact.get("phase").and_then(|value| value.as_str()),
            Some("running")
        );
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

    #[test]
    fn text_signal_property_updates_preserve_control_summary() {
        let mut session = WorkspaceSession::empty("Session");
        session
            .execute_command("entity.create.robot_cell")
            .expect("robot cell should be created");
        let signal = session
            .graph
            .document()
            .nodes
            .values()
            .find(|entity| entity.id == "ent_sig_001_operator_mode")
            .cloned()
            .expect("signal should exist");

        let result = session
            .update_entity_properties(&EntityPropertyUpdateInput {
                entity_id: signal.id.clone(),
                changes: serde_json::json!({
                    "name": "Operator Mode Updated",
                    "tags": ["control", "edited"],
                    "kind": "text",
                    "currentValue": "manual",
                    "parameterSet.unit": "label",
                    "parameterSet.thresholds": ["auto", "manual"]
                }),
            })
            .expect("signal should update");

        let snapshot = session.snapshot();
        let updated = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == signal.id)
            .expect("updated signal should exist");
        let robot_cell = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == "ent_cell_001")
            .expect("robot cell should still exist");

        assert_eq!(result.status, "applied");
        assert_eq!(updated.name, "Operator Mode Updated");
        assert_eq!(updated.data["kind"], "text");
        assert_eq!(updated.data["currentValue"], "manual");
        assert_eq!(updated.data["parameterSet"]["unit"], "label");
        assert_eq!(updated.data["parameterSet"]["thresholds"][0], "auto");
        assert_eq!(robot_cell.data["control"]["blockedStateId"], "idle");
    }

    #[test]
    fn boolean_and_scalar_signal_updates_recompute_robot_cell_control_summary() {
        let mut session = WorkspaceSession::empty("Session");
        session
            .execute_command("entity.create.robot_cell")
            .expect("robot cell should be created");

        session
            .update_entity_properties(&EntityPropertyUpdateInput {
                entity_id: "ent_sig_001_cycle_start".to_string(),
                changes: serde_json::json!({
                    "currentValue": true
                }),
            })
            .expect("boolean signal should update");

        let snapshot = session.snapshot();
        let robot_cell = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == "ent_cell_001")
            .expect("robot cell should exist");
        let summary = robot_cell
            .robot_cell_summary
            .as_ref()
            .expect("robot cell summary should exist");
        assert!(summary.blocked_sequence_detected);
        assert_eq!(summary.blocked_state_id.as_deref(), Some("place"));

        session
            .update_entity_properties(&EntityPropertyUpdateInput {
                entity_id: "ent_sig_001_progress_gate".to_string(),
                changes: serde_json::json!({
                    "currentValue": 1.0
                }),
            })
            .expect("scalar signal should update");

        let snapshot = session.snapshot();
        let robot_cell = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == "ent_cell_001")
            .expect("robot cell should exist");
        let summary = robot_cell
            .robot_cell_summary
            .as_ref()
            .expect("robot cell summary should exist");
        assert!(!summary.blocked_sequence_detected);
        assert_eq!(summary.blocked_state_id, None);
    }

    #[test]
    fn incompatible_signal_kind_updates_are_rejected_when_controller_refs_break() {
        let mut session = WorkspaceSession::empty("Session");
        session
            .execute_command("entity.create.robot_cell")
            .expect("robot cell should be created");

        let error = match session.update_entity_properties(&EntityPropertyUpdateInput {
            entity_id: "ent_sig_001_progress_gate".to_string(),
            changes: serde_json::json!({
                "kind": "text",
                "currentValue": "ready"
            }),
        }) {
            Ok(_) => panic!("invalid controller graph should be rejected"),
            Err(error) => error,
        };

        assert!(error.contains("incompatible"));
    }

    #[test]
    fn controller_updates_recompute_robot_cell_control_summary() {
        let mut session = WorkspaceSession::empty("Session");
        session
            .execute_command("entity.create.robot_cell")
            .expect("robot cell should be created");

        let controller = session
            .graph
            .document()
            .nodes
            .values()
            .find(|entity| entity.id == "ent_ctrl_001")
            .cloned()
            .expect("controller should exist");
        let mut state_machine: ControllerStateMachine =
            serde_json::from_value(controller.data["stateMachine"].clone())
                .expect("state machine should deserialize");
        state_machine.transitions.pop();

        let result = session
            .update_entity_properties(&EntityPropertyUpdateInput {
                entity_id: controller.id,
                changes: serde_json::json!({
                    "stateMachine": state_machine
                }),
            })
            .expect("controller update should apply");

        let snapshot = session.snapshot();
        let robot_cell = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == "ent_cell_001")
            .expect("robot cell should still exist");
        let summary = robot_cell
            .robot_cell_summary
            .as_ref()
            .expect("robot cell summary should exist");

        assert_eq!(result.status, "applied");
        assert_eq!(summary.controller_transition_count, 2);
        assert!(summary.blocked_sequence_detected);
        assert_eq!(summary.blocked_state_id.as_deref(), Some("idle"));
        assert_eq!(robot_cell.data["control"]["controllerTransitionCount"], 2);
    }

    #[test]
    fn robot_target_property_update_reorders_sequence_and_preview() {
        let mut session = WorkspaceSession::empty("Session");
        session
            .execute_command("entity.create.robot_cell")
            .expect("robot cell should be created");
        let target = session
            .graph
            .document()
            .nodes
            .values()
            .find(|entity| entity.id == "ent_target_001_transfer")
            .cloned()
            .expect("target should exist");

        let result = session
            .update_entity_properties(&EntityPropertyUpdateInput {
                entity_id: target.id.clone(),
                changes: serde_json::json!({
                    "parameterSet.orderIndex": 4
                }),
            })
            .expect("target order should update");

        let snapshot = session.snapshot();
        let robot_cell = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == "ent_cell_001")
            .expect("robot cell should still exist");
        let sequence = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == "ent_seq_001")
            .expect("robot sequence should still exist");

        assert_eq!(result.status, "applied");
        assert_eq!(
            robot_cell.data["targetPreview"],
            "pick -> place -> transfer"
        );
        assert_eq!(
            robot_cell.data["targetIds"].as_array().map(|entries| {
                entries
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<_>>()
            }),
            Some(vec![
                "ent_target_001_pick",
                "ent_target_001_place",
                "ent_target_001_transfer",
            ])
        );
        assert_eq!(sequence.data["targetPreview"], "pick -> place -> transfer");
        assert_eq!(
            sequence.data["targetIds"].as_array().map(|entries| {
                entries
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<_>>()
            }),
            Some(vec![
                "ent_target_001_pick",
                "ent_target_001_place",
                "ent_target_001_transfer",
            ])
        );
    }

    #[test]
    fn ai_suggestion_apply_and_reject_update_review_state() {
        let mut session = WorkspaceSession::empty("Session");
        session
            .execute_command("entity.create.part")
            .expect("part should be created");
        let part = session
            .latest_parametric_part()
            .expect("part should be available");

        let response = AiChatResponse {
            answer: "Suggestion structuree".to_string(),
            runtime: AiRuntimeStatus {
                available: true,
                provider: "ollama".to_string(),
                endpoint: "http://127.0.0.1:11434".to_string(),
                mode: "test".to_string(),
                local_only: true,
                active_profile: "furnace".to_string(),
                available_profiles: vec![
                    "balanced".to_string(),
                    "max".to_string(),
                    "furnace".to_string(),
                ],
                active_model: Some("gemma3:27b".to_string()),
                available_models: vec!["gemma3:27b".to_string()],
                gemma3_models: vec!["gemma3:27b".to_string()],
                warning: None,
            },
            references: vec![format!("entity:{}", part.id)],
            structured: Some(faero_types::AiStructuredExplain {
                summary: "Change la piece".to_string(),
                runtime_profile: "furnace".to_string(),
                context_refs: vec![faero_types::AiContextReference {
                    entity_id: Some(part.id.clone()),
                    role: "source".to_string(),
                    path: "parameterSet.widthMm".to_string(),
                }],
                confidence: 0.84,
                risk_level: faero_types::AiRiskLevel::Medium,
                limitations: vec!["Test backend".to_string()],
                critique_passes: vec![faero_types::AiCritiquePass {
                    stage: "critic".to_string(),
                    summary: "La suggestion reste plausible sur la base des artefacts locaux."
                        .to_string(),
                    confidence_delta: -0.02,
                    issues: vec!["validation manuelle recommandee".to_string()],
                    adjustments: vec!["review before apply".to_string()],
                }],
                proposed_commands: vec![faero_types::AiProposedCommand {
                    kind: "entity.properties.update".to_string(),
                    target_id: Some(part.id.clone()),
                    payload: serde_json::json!({
                        "changes": {
                            "name": "Bracket-Reviewed",
                            "parameterSet.widthMm": 150.0,
                            "parameterSet.heightMm": 90.0,
                            "parameterSet.depthMm": 12.0
                        }
                    }),
                }],
                explanation: vec!["La piece doit etre elargie.".to_string()],
            }),
            suggestion_id: None,
            warnings: Vec::new(),
            source: "test".to_string(),
        };

        let suggestion_id = session
            .record_ai_response("applique une correction", &response)
            .expect("suggestion should persist");
        let apply_result = session
            .apply_ai_suggestion(&suggestion_id)
            .expect("suggestion should apply");

        let snapshot = session.snapshot();
        let updated_part = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == part.id)
            .expect("updated part should exist");
        let applied_suggestion = snapshot
            .entities
            .iter()
            .find(|entity| entity.id == suggestion_id)
            .expect("applied suggestion should exist");

        assert_eq!(apply_result.command_id, "ai.suggestion.apply");
        assert_eq!(updated_part.name, "Bracket-Reviewed");
        assert_eq!(applied_suggestion.data["reviewStatus"], "accepted");
        assert!(
            snapshot
                .recent_activity
                .iter()
                .any(|entry| entry.kind == "ai.suggestion.applied")
        );

        let rejected_id = session
            .record_ai_response("rejette une correction", &response)
            .expect("second suggestion should persist");
        session
            .reject_ai_suggestion(&rejected_id)
            .expect("suggestion should reject");
        let rejected = session
            .snapshot()
            .entities
            .into_iter()
            .find(|entity| entity.id == rejected_id)
            .expect("rejected suggestion should exist");
        assert_eq!(rejected.data["reviewStatus"], "rejected");
    }

    #[test]
    fn perception_commissioning_and_optimization_commands_create_summaries() {
        let mut session = WorkspaceSession::empty("Session");

        let perception = session
            .execute_command("perception.run.start")
            .expect("perception run should execute");
        let commissioning = session
            .execute_command("commissioning.session.start")
            .expect("commissioning session should execute");
        let comparison = session
            .execute_command("commissioning.compare.as_built")
            .expect("as-built comparison should execute");
        let optimization = session
            .execute_command("optimization.run.start")
            .expect("optimization should execute");

        let snapshot = session.snapshot();
        assert_eq!(perception.status, "applied");
        assert_eq!(commissioning.status, "applied");
        assert_eq!(comparison.status, "applied");
        assert_eq!(optimization.status, "applied");
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.sensor_rig_summary.is_some())
        );
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.perception_run_summary.is_some())
        );
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.commissioning_session_summary.is_some())
        );
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.as_built_comparison_summary.is_some())
        );
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.optimization_study_summary.is_some())
        );
    }

    #[test]
    fn endpoint_creation_and_integration_replay_cover_multiple_endpoint_families() {
        let mut session = WorkspaceSession::empty("Session");
        for _ in 0..6 {
            session
                .execute_command("entity.create.external_endpoint")
                .expect("endpoint creation should succeed");
        }

        let replay = session
            .execute_command("integration.replay.degraded")
            .expect("degraded replay should execute");
        let snapshot = session.snapshot();

        assert_eq!(snapshot.endpoints.len(), 6);
        assert!(
            snapshot
                .endpoints
                .iter()
                .any(|endpoint| endpoint.endpoint_type == "ros2")
        );
        assert!(
            snapshot
                .endpoints
                .iter()
                .any(|endpoint| endpoint.endpoint_type == "opcua")
        );
        assert!(
            snapshot
                .endpoints
                .iter()
                .any(|endpoint| endpoint.endpoint_type == "bluetooth_le")
        );
        assert_eq!(replay.status, "applied");
        assert!(
            snapshot
                .entities
                .iter()
                .any(|entity| entity.entity_type == "IndustrialReplay")
        );
    }
}
