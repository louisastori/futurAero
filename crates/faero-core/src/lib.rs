use std::collections::BTreeMap;

use faero_assembly::{AssemblyError, joint_degrees_of_freedom, solve_assembly};
use faero_types::{
    AssemblyData, AssemblyJoint, AssemblyJointAxis, AssemblyJointLimits, AssemblyJointType,
    AssemblyMateConstraint, AssemblyMateType, AssemblyOccurrence, AssemblySolveReport,
    AssemblySolveStatus, AssemblySolvedOccurrence, AssemblyTransform, CommandEnvelope,
    EntityRecord, EventEnvelope, ExternalEndpoint, PluginManifest, ProjectDocument, SignalValue,
    TelemetryStream,
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
    #[error("signal entity `{0}` does not exist")]
    SignalEntityNotFound(String),
    #[error("entity `{0}` is not a signal")]
    EntityIsNotSignal(String),
    #[error("entity `{0}` is not an assembly")]
    EntityIsNotAssembly(String),
    #[error("assembly payload is invalid: {0}")]
    AssemblyPayloadInvalid(String),
    #[error("assembly definition `{0}` does not exist")]
    AssemblyDefinitionNotFound(String),
    #[error("assembly definition `{0}` must be a Part or Assembly")]
    AssemblyDefinitionUnsupported(String),
    #[error("assembly occurrence `{0}` already exists")]
    AssemblyOccurrenceAlreadyExists(String),
    #[error("assembly occurrence `{0}` does not exist")]
    AssemblyOccurrenceNotFound(String),
    #[error("assembly mate `{0}` already exists")]
    AssemblyMateAlreadyExists(String),
    #[error("assembly mate `{0}` does not exist")]
    AssemblyMateNotFound(String),
    #[error("assembly joint `{0}` already exists")]
    AssemblyJointAlreadyExists(String),
    #[error("assembly joint `{0}` does not exist")]
    AssemblyJointNotFound(String),
    #[error("assembly joint must reference two distinct known occurrences")]
    AssemblyJointInvalidOccurrences,
    #[error("assembly joint axis must not be the zero vector")]
    AssemblyJointAxisInvalid,
    #[error("assembly joint limits are invalid")]
    AssemblyJointLimitsInvalid,
    #[error("assembly joint state is invalid: {0}")]
    AssemblyJointStateInvalid(String),
}

#[derive(Debug, Clone)]
pub enum CoreCommand {
    CreateEntity(EntityRecord),
    ReplaceEntity(EntityRecord),
    RegisterEndpoint(ExternalEndpoint),
    RegisterStream(TelemetryStream),
    InstallPlugin(PluginManifest),
    SetPluginEnabled {
        plugin_id: String,
        enabled: bool,
    },
    SetSignalValue {
        entity_id: String,
        value: SignalValue,
    },
    AddAssemblyOccurrence {
        assembly_id: String,
        occurrence: AssemblyOccurrence,
    },
    TransformAssemblyOccurrence {
        assembly_id: String,
        occurrence_id: String,
        transform: AssemblyTransform,
    },
    AddAssemblyMate {
        assembly_id: String,
        mate: AssemblyMateConstraint,
    },
    RemoveAssemblyMate {
        assembly_id: String,
        mate_id: String,
    },
    CreateAssemblyJoint {
        assembly_id: String,
        joint: AssemblyJoint,
    },
    SetAssemblyJointState {
        assembly_id: String,
        joint_id: String,
        current_position: f64,
    },
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
            CoreCommand::SetSignalValue { entity_id, value } => {
                let Some(existing) = self.document.nodes.get(&entity_id).cloned() else {
                    return Err(CoreError::SignalEntityNotFound(entity_id));
                };
                if existing.entity_type != "Signal" {
                    return Err(CoreError::EntityIsNotSignal(existing.id));
                }

                let mut next_entity = existing.clone();
                if let Some(data) = next_entity.data.as_object_mut() {
                    data.insert("currentValue".to_string(), serialize_payload(&value));
                } else {
                    next_entity.data = serde_json::json!({
                        "currentValue": value
                    });
                }
                let payload = serde_json::json!({
                    "entityId": entity_id,
                    "value": value
                });
                let command_id = self.record_command(
                    "signal.value.set",
                    Some(next_entity.id.clone()),
                    Some(existing.revision),
                    payload,
                );
                let revision = self.next_revision();
                next_entity.revision = revision.clone();
                let event_payload = serialize_payload(&next_entity);
                self.document
                    .nodes
                    .insert(next_entity.id.clone(), next_entity.clone());

                self.record_event(
                    "signal.value.updated",
                    Some(next_entity.id),
                    command_id,
                    revision,
                    event_payload,
                )
            }
            CoreCommand::AddAssemblyOccurrence {
                assembly_id,
                occurrence,
            } => {
                let definition = self
                    .document
                    .nodes
                    .get(&occurrence.definition_entity_id)
                    .ok_or_else(|| {
                        CoreError::AssemblyDefinitionNotFound(
                            occurrence.definition_entity_id.clone(),
                        )
                    })?;
                if !matches!(definition.entity_type.as_str(), "Part" | "Assembly") {
                    return Err(CoreError::AssemblyDefinitionUnsupported(
                        occurrence.definition_entity_id,
                    ));
                }
                let occurrence_id = occurrence.id.clone();
                let payload = serialize_payload(&occurrence);
                self.mutate_assembly_entity(
                    &assembly_id,
                    "assembly.occurrence.add",
                    None,
                    payload,
                    move |assembly| {
                        if assembly
                            .occurrences
                            .iter()
                            .any(|existing| existing.id == occurrence_id)
                        {
                            return Err(CoreError::AssemblyOccurrenceAlreadyExists(
                                occurrence_id.clone(),
                            ));
                        }
                        assembly.occurrences.push(occurrence);
                        Ok(())
                    },
                )
            }
            CoreCommand::TransformAssemblyOccurrence {
                assembly_id,
                occurrence_id,
                transform,
            } => {
                let payload = serde_json::json!({
                    "occurrenceId": occurrence_id,
                    "transform": transform,
                });
                self.mutate_assembly_entity(
                    &assembly_id,
                    "assembly.occurrence.transform",
                    None,
                    payload,
                    move |assembly| {
                        let occurrence = assembly
                            .occurrences
                            .iter_mut()
                            .find(|entry| entry.id == occurrence_id)
                            .ok_or_else(|| {
                                CoreError::AssemblyOccurrenceNotFound(occurrence_id.clone())
                            })?;
                        occurrence.transform = transform;
                        Ok(())
                    },
                )
            }
            CoreCommand::AddAssemblyMate { assembly_id, mate } => {
                let mate_id = mate.id.clone();
                let left_occurrence_id = mate.left_occurrence_id.clone();
                let right_occurrence_id = mate.right_occurrence_id.clone();
                if left_occurrence_id == right_occurrence_id {
                    return Err(CoreError::AssemblyPayloadInvalid(
                        "mate constraints must target two distinct occurrences".to_string(),
                    ));
                }
                if let AssemblyMateType::Offset { distance_mm } = mate.mate_type
                    && distance_mm < 0.0
                {
                    return Err(CoreError::AssemblyPayloadInvalid(
                        "offset mates must use a non-negative distance".to_string(),
                    ));
                }
                let payload = serialize_payload(&mate);
                self.mutate_assembly_entity(
                    &assembly_id,
                    "assembly.mate.add",
                    None,
                    payload,
                    move |assembly| {
                        if assembly
                            .mate_constraints
                            .iter()
                            .any(|existing| existing.id == mate_id)
                        {
                            return Err(CoreError::AssemblyMateAlreadyExists(mate_id.clone()));
                        }
                        if !assembly
                            .occurrences
                            .iter()
                            .any(|occurrence| occurrence.id == left_occurrence_id)
                        {
                            return Err(CoreError::AssemblyOccurrenceNotFound(
                                left_occurrence_id.clone(),
                            ));
                        }
                        if !assembly
                            .occurrences
                            .iter()
                            .any(|occurrence| occurrence.id == right_occurrence_id)
                        {
                            return Err(CoreError::AssemblyOccurrenceNotFound(
                                right_occurrence_id.clone(),
                            ));
                        }
                        assembly.mate_constraints.push(mate);
                        Ok(())
                    },
                )
            }
            CoreCommand::RemoveAssemblyMate {
                assembly_id,
                mate_id,
            } => {
                let payload = serde_json::json!({ "mateId": mate_id });
                self.mutate_assembly_entity(
                    &assembly_id,
                    "assembly.mate.remove",
                    None,
                    payload,
                    move |assembly| {
                        let before = assembly.mate_constraints.len();
                        assembly
                            .mate_constraints
                            .retain(|constraint| constraint.id != mate_id);
                        if assembly.mate_constraints.len() == before {
                            return Err(CoreError::AssemblyMateNotFound(mate_id.clone()));
                        }
                        Ok(())
                    },
                )
            }
            CoreCommand::CreateAssemblyJoint {
                assembly_id,
                mut joint,
            } => {
                let joint_id = joint.id.clone();
                let source_occurrence_id = joint.source_occurrence_id.clone();
                let target_occurrence_id = joint.target_occurrence_id.clone();

                validate_joint_axis(joint.axis)?;
                validate_joint_state(joint.joint_type, joint.limits, joint.current_position)?;
                joint.degrees_of_freedom = joint_degrees_of_freedom(joint.joint_type);

                let payload = serialize_payload(&joint);
                self.mutate_assembly_entity(
                    &assembly_id,
                    "joint.create",
                    Some("joint.state.changed"),
                    payload,
                    move |assembly| {
                        if assembly
                            .joints
                            .iter()
                            .any(|existing| existing.id == joint_id)
                        {
                            return Err(CoreError::AssemblyJointAlreadyExists(joint_id.clone()));
                        }
                        if source_occurrence_id == target_occurrence_id {
                            return Err(CoreError::AssemblyJointInvalidOccurrences);
                        }
                        if !assembly
                            .occurrences
                            .iter()
                            .any(|occurrence| occurrence.id == source_occurrence_id)
                            || !assembly
                                .occurrences
                                .iter()
                                .any(|occurrence| occurrence.id == target_occurrence_id)
                        {
                            return Err(CoreError::AssemblyJointInvalidOccurrences);
                        }
                        assembly.joints.push(joint);
                        Ok(())
                    },
                )
            }
            CoreCommand::SetAssemblyJointState {
                assembly_id,
                joint_id,
                current_position,
            } => {
                let payload = serde_json::json!({
                    "jointId": joint_id,
                    "currentPosition": current_position,
                });
                self.mutate_assembly_entity(
                    &assembly_id,
                    "joint.state.set",
                    Some("joint.state.changed"),
                    payload,
                    move |assembly| {
                        let joint = assembly
                            .joints
                            .iter_mut()
                            .find(|entry| entry.id == joint_id)
                            .ok_or_else(|| CoreError::AssemblyJointNotFound(joint_id.clone()))?;
                        validate_joint_state(joint.joint_type, joint.limits, current_position)?;
                        joint.current_position = current_position;
                        joint.degrees_of_freedom = joint_degrees_of_freedom(joint.joint_type);
                        Ok(())
                    },
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

    fn mutate_assembly_entity<F>(
        &mut self,
        assembly_id: &str,
        command_kind: &str,
        event_kind_override: Option<&str>,
        payload: Value,
        mutate: F,
    ) -> Result<EventEnvelope, CoreError>
    where
        F: FnOnce(&mut AssemblyData) -> Result<(), CoreError>,
    {
        let existing = self
            .document
            .nodes
            .get(assembly_id)
            .cloned()
            .ok_or_else(|| CoreError::EntityNotFound(assembly_id.to_string()))?;
        if existing.entity_type != "Assembly" {
            return Err(CoreError::EntityIsNotAssembly(existing.id));
        }

        let mut assembly = deserialize_assembly_data(&existing)?;
        mutate(&mut assembly)?;
        assembly.parameter_set.occurrence_count = assembly.occurrences.len();
        assembly.parameter_set.mate_count = assembly.mate_constraints.len();
        assembly.parameter_set.joint_count = assembly.joints.len();
        let solve_report = compute_assembly_solve_report(&assembly)?;
        let event_kind = event_kind_override.unwrap_or(match solve_report.status {
            AssemblySolveStatus::Solved => "assembly.solved",
            AssemblySolveStatus::Conflicting => "assembly.unsolved",
        });
        assembly.solve_report = Some(solve_report);

        let command_id = self.record_command(
            command_kind,
            Some(existing.id.clone()),
            Some(existing.revision.clone()),
            payload,
        );
        let revision = self.next_revision();
        let mut next_entity = existing.clone();
        next_entity.revision = revision.clone();
        next_entity.data = serialize_payload(&assembly);
        let event_payload = next_entity.data.clone();
        self.document
            .nodes
            .insert(next_entity.id.clone(), next_entity.clone());

        self.record_event(
            event_kind,
            Some(next_entity.id),
            command_id,
            revision,
            event_payload,
        )
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

fn deserialize_assembly_data(entity: &EntityRecord) -> Result<AssemblyData, CoreError> {
    serde_json::from_value(entity.data.clone())
        .map_err(|error| CoreError::AssemblyPayloadInvalid(format!("{}: {error}", entity.id)))
}

fn validate_joint_axis(axis: AssemblyJointAxis) -> Result<(), CoreError> {
    if [axis.x, axis.y, axis.z]
        .into_iter()
        .all(|component| component.abs() <= f64::EPSILON)
    {
        return Err(CoreError::AssemblyJointAxisInvalid);
    }

    Ok(())
}

fn validate_joint_state(
    joint_type: AssemblyJointType,
    limits: Option<AssemblyJointLimits>,
    current_position: f64,
) -> Result<(), CoreError> {
    if !current_position.is_finite() {
        return Err(CoreError::AssemblyJointStateInvalid(
            "current position must be finite".to_string(),
        ));
    }

    if let Some(limits) = limits {
        if !limits.min.is_finite() || !limits.max.is_finite() || limits.min > limits.max {
            return Err(CoreError::AssemblyJointLimitsInvalid);
        }
        if joint_type == AssemblyJointType::Fixed
            && (limits.min.abs() > f64::EPSILON || limits.max.abs() > f64::EPSILON)
        {
            return Err(CoreError::AssemblyJointLimitsInvalid);
        }
    }

    if joint_type == AssemblyJointType::Fixed && current_position.abs() > f64::EPSILON {
        return Err(CoreError::AssemblyJointStateInvalid(
            "fixed joints must stay at position 0".to_string(),
        ));
    }

    if let Some(limits) = limits
        && (current_position < limits.min || current_position > limits.max)
    {
        return Err(CoreError::AssemblyJointStateInvalid(
            "current position must stay within joint limits".to_string(),
        ));
    }

    Ok(())
}

fn compute_assembly_solve_report(
    assembly: &AssemblyData,
) -> Result<AssemblySolveReport, CoreError> {
    match solve_assembly(&assembly.occurrences, &assembly.mate_constraints) {
        Ok(report) => Ok(report),
        Err(AssemblyError::NotEnoughOccurrences) => Ok(AssemblySolveReport {
            status: AssemblySolveStatus::Conflicting,
            constrained_occurrence_count: 0,
            total_mate_count: assembly.mate_constraints.len(),
            degrees_of_freedom_estimate: 0,
            solved_occurrences: assembly
                .occurrences
                .iter()
                .map(|occurrence| AssemblySolvedOccurrence {
                    occurrence_id: occurrence.id.clone(),
                    transform: occurrence.transform,
                })
                .collect(),
            warnings: vec!["assembly requires at least two occurrences to solve".to_string()],
        }),
        Err(error) => Err(CoreError::AssemblyPayloadInvalid(error.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use faero_types::{
        Addressing, AssemblyData, AssemblyJoint, AssemblyJointAxis, AssemblyJointLimits,
        AssemblyJointType, AssemblyMateConstraint, AssemblyMateType, AssemblyOccurrence,
        AssemblyTransform, ConnectionMode, EndpointType, LinkMetrics, PluginContribution,
        QosProfile, SignalValue, StreamDirection, TimingProfile, TransportProfile,
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
            release_channel: "stable".to_string(),
            capabilities: vec!["panel".to_string()],
            permissions: vec![
                "project.read".to_string(),
                "integration.observe".to_string(),
            ],
            contributions: vec![PluginContribution {
                kind: "panel".to_string(),
                target: "workspace.right".to_string(),
                title: "Integration Viewer".to_string(),
            }],
            entrypoints: vec!["plugins/integration-viewer/index.js".to_string()],
            compatibility: vec!["faero-core@0.1".to_string()],
            signature: Some("sha256:demo".to_string()),
            status: "installed".to_string(),
        }
    }

    fn sample_signal() -> EntityRecord {
        EntityRecord {
            id: "ent_sig_001".to_string(),
            entity_type: "Signal".to_string(),
            name: "CycleStart".to_string(),
            revision: "rev_seed".to_string(),
            status: "active".to_string(),
            data: serde_json::json!({
                "signalId": "sig_cycle_start",
                "kind": "boolean",
                "initialValue": false,
                "currentValue": false,
                "tags": ["control", "mvp"]
            }),
        }
    }

    fn sample_entity_with_id(id: &str, name: &str) -> EntityRecord {
        EntityRecord {
            id: id.to_string(),
            name: name.to_string(),
            ..sample_entity()
        }
    }

    fn sample_assembly_entity() -> EntityRecord {
        EntityRecord {
            id: "ent_asm_001".to_string(),
            entity_type: "Assembly".to_string(),
            name: "Assembly-001".to_string(),
            revision: "rev_seed".to_string(),
            status: "active".to_string(),
            data: serde_json::to_value(AssemblyData {
                tags: vec!["assembly".to_string()],
                ..AssemblyData::default()
            })
            .expect("assembly payload should serialize"),
        }
    }

    fn sample_occurrence(id: &str, definition_entity_id: &str, x_mm: f64) -> AssemblyOccurrence {
        AssemblyOccurrence {
            id: id.to_string(),
            definition_entity_id: definition_entity_id.to_string(),
            transform: AssemblyTransform {
                x_mm,
                ..AssemblyTransform::default()
            },
        }
    }

    fn sample_mate(id: &str, left: &str, right: &str) -> AssemblyMateConstraint {
        AssemblyMateConstraint {
            id: id.to_string(),
            left_occurrence_id: left.to_string(),
            right_occurrence_id: right.to_string(),
            mate_type: AssemblyMateType::Coincident,
        }
    }

    fn sample_joint(id: &str, joint_type: AssemblyJointType) -> AssemblyJoint {
        AssemblyJoint {
            id: id.to_string(),
            joint_type,
            source_occurrence_id: "occ_001".to_string(),
            target_occurrence_id: "occ_002".to_string(),
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
    fn sets_signal_values_through_a_core_command() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_signal()))
            .expect("signal should be created");

        let event = graph
            .apply_command(CoreCommand::SetSignalValue {
                entity_id: "ent_sig_001".to_string(),
                value: SignalValue::Bool(true),
            })
            .expect("signal value should update");

        assert_eq!(event.kind, "signal.value.updated");
        assert_eq!(
            graph.document().nodes["ent_sig_001"].data["currentValue"],
            serde_json::json!(true)
        );
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

    #[test]
    fn assembly_commands_persist_occurrences_mates_and_emit_solve_status() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity_with_id(
                "ent_part_001",
                "Part-A",
            )))
            .expect("part a should exist");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity_with_id(
                "ent_part_002",
                "Part-B",
            )))
            .expect("part b should exist");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_assembly_entity()))
            .expect("assembly should exist");

        let first = graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_001", "ent_part_001", 0.0),
            })
            .expect("first occurrence should be added");
        let second = graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_002", "ent_part_002", 80.0),
            })
            .expect("second occurrence should be added");
        let solved = graph
            .apply_command(CoreCommand::AddAssemblyMate {
                assembly_id: "ent_asm_001".to_string(),
                mate: sample_mate("mate_001", "occ_001", "occ_002"),
            })
            .expect("mate should solve the assembly");

        assert_eq!(first.kind, "assembly.unsolved");
        assert_eq!(second.kind, "assembly.unsolved");
        assert_eq!(solved.kind, "assembly.solved");
        assert_eq!(graph.document().commands[3].kind, "assembly.occurrence.add");
        assert_eq!(graph.document().commands[5].kind, "assembly.mate.add");

        let assembly: AssemblyData =
            serde_json::from_value(graph.document().nodes["ent_asm_001"].data.clone())
                .expect("assembly payload should deserialize");
        assert_eq!(assembly.occurrences.len(), 2);
        assert_eq!(assembly.mate_constraints.len(), 1);
        assert_eq!(
            assembly.solve_report.as_ref().map(|report| report.status),
            Some(AssemblySolveStatus::Solved)
        );
    }

    #[test]
    fn assembly_commands_reject_invalid_references_and_duplicates() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity()))
            .expect("part should exist");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_assembly_entity()))
            .expect("assembly should exist");
        graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_001", "ent_part_001", 0.0),
            })
            .expect("occurrence should be added");

        let duplicate = graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_001", "ent_part_001", 10.0),
            })
            .expect_err("duplicate occurrence should fail");
        assert_eq!(
            duplicate,
            CoreError::AssemblyOccurrenceAlreadyExists("occ_001".to_string())
        );

        let missing_definition = graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_002", "missing", 10.0),
            })
            .expect_err("missing definition should fail");
        assert_eq!(
            missing_definition,
            CoreError::AssemblyDefinitionNotFound("missing".to_string())
        );

        let missing_mate = graph
            .apply_command(CoreCommand::RemoveAssemblyMate {
                assembly_id: "ent_asm_001".to_string(),
                mate_id: "mate_missing".to_string(),
            })
            .expect_err("missing mate should fail");
        assert_eq!(
            missing_mate,
            CoreError::AssemblyMateNotFound("mate_missing".to_string())
        );
    }

    #[test]
    fn joint_commands_persist_state_and_record_auditable_activity() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity_with_id(
                "ent_part_001",
                "Part-A",
            )))
            .expect("part a should exist");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity_with_id(
                "ent_part_002",
                "Part-B",
            )))
            .expect("part b should exist");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_assembly_entity()))
            .expect("assembly should exist");
        graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_001", "ent_part_001", 0.0),
            })
            .expect("first occurrence should be added");
        graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_002", "ent_part_002", 80.0),
            })
            .expect("second occurrence should be added");

        let created = graph
            .apply_command(CoreCommand::CreateAssemblyJoint {
                assembly_id: "ent_asm_001".to_string(),
                joint: sample_joint("joint_001", AssemblyJointType::Revolute),
            })
            .expect("joint should be created");
        let updated = graph
            .apply_command(CoreCommand::SetAssemblyJointState {
                assembly_id: "ent_asm_001".to_string(),
                joint_id: "joint_001".to_string(),
                current_position: 0.5,
            })
            .expect("joint state should update");

        assert_eq!(created.kind, "joint.state.changed");
        assert_eq!(updated.kind, "joint.state.changed");

        let command_kinds = graph
            .document()
            .commands
            .iter()
            .map(|entry| entry.kind.as_str())
            .collect::<Vec<_>>();
        assert!(command_kinds.contains(&"joint.create"));
        assert!(command_kinds.contains(&"joint.state.set"));

        let assembly: AssemblyData =
            serde_json::from_value(graph.document().nodes["ent_asm_001"].data.clone())
                .expect("assembly payload should deserialize");
        assert_eq!(assembly.parameter_set.joint_count, 1);
        assert_eq!(assembly.joints.len(), 1);
        assert_eq!(assembly.joints[0].degrees_of_freedom, 1);
        assert_eq!(assembly.joints[0].current_position, 0.5);
    }

    #[test]
    fn joint_commands_reject_invalid_limits_and_state() {
        let mut graph = ProjectGraph::new("Demo");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity_with_id(
                "ent_part_001",
                "Part-A",
            )))
            .expect("part a should exist");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_entity_with_id(
                "ent_part_002",
                "Part-B",
            )))
            .expect("part b should exist");
        graph
            .apply_command(CoreCommand::CreateEntity(sample_assembly_entity()))
            .expect("assembly should exist");
        graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_001", "ent_part_001", 0.0),
            })
            .expect("first occurrence should be added");
        graph
            .apply_command(CoreCommand::AddAssemblyOccurrence {
                assembly_id: "ent_asm_001".to_string(),
                occurrence: sample_occurrence("occ_002", "ent_part_002", 80.0),
            })
            .expect("second occurrence should be added");

        let mut invalid_occurrence_joint =
            sample_joint("joint_missing", AssemblyJointType::Revolute);
        invalid_occurrence_joint.target_occurrence_id = "missing".to_string();
        let invalid_occurrences = graph
            .apply_command(CoreCommand::CreateAssemblyJoint {
                assembly_id: "ent_asm_001".to_string(),
                joint: invalid_occurrence_joint,
            })
            .expect_err("missing occurrence should fail");
        assert_eq!(
            invalid_occurrences,
            CoreError::AssemblyJointInvalidOccurrences
        );

        let mut invalid_limits_joint = sample_joint("joint_invalid", AssemblyJointType::Prismatic);
        invalid_limits_joint.limits = Some(AssemblyJointLimits { min: 2.0, max: 1.0 });
        let invalid_limits = graph
            .apply_command(CoreCommand::CreateAssemblyJoint {
                assembly_id: "ent_asm_001".to_string(),
                joint: invalid_limits_joint,
            })
            .expect_err("invalid limits should fail");
        assert_eq!(invalid_limits, CoreError::AssemblyJointLimitsInvalid);

        let mut fixed_joint = sample_joint("joint_fixed", AssemblyJointType::Fixed);
        fixed_joint.limits = Some(AssemblyJointLimits { min: 0.0, max: 0.0 });
        let created = graph
            .apply_command(CoreCommand::CreateAssemblyJoint {
                assembly_id: "ent_asm_001".to_string(),
                joint: fixed_joint,
            })
            .expect("fixed joint should be created");
        assert_eq!(created.kind, "joint.state.changed");

        let invalid_state = graph
            .apply_command(CoreCommand::SetAssemblyJointState {
                assembly_id: "ent_asm_001".to_string(),
                joint_id: "joint_fixed".to_string(),
                current_position: 0.1,
            })
            .expect_err("fixed joints should reject non-zero state");
        assert_eq!(
            invalid_state,
            CoreError::AssemblyJointStateInvalid(
                "fixed joints must stay at position 0".to_string(),
            )
        );
    }
}
