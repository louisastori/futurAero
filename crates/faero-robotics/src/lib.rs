use std::collections::{BTreeMap, HashSet};

use faero_types::{
    ControlTransition, ControllerState, ControllerStateMachine, SignalAssignment, SignalComparator,
    SignalCondition, SignalDefinition, SignalKind, SignalValue,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotCellModel {
    pub id: String,
    pub scene_assembly_id: String,
    pub robot_ids: Vec<String>,
    pub equipment_ids: Vec<String>,
    pub safety_zone_ids: Vec<String>,
    pub sequence_ids: Vec<String>,
    pub controller_model_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotToolMountRef {
    pub equipment_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotWorkspaceBounds {
    pub reach_radius_mm: f64,
    pub vertical_span_mm: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotPayloadLimits {
    pub nominal_kg: f64,
    pub max_kg: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotModel {
    pub id: String,
    pub cell_id: String,
    pub kinematic_chain: Vec<String>,
    pub joint_ids: Vec<String>,
    pub tool_mount_ref: RobotToolMountRef,
    pub workspace_bounds: RobotWorkspaceBounds,
    pub payload_limits: RobotPayloadLimits,
    pub calibration_state: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentType {
    Conveyor,
    Workstation,
    Gripper,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentParameterSet {
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
    pub nominal_speed_mm_s: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentModel {
    pub id: String,
    pub cell_id: String,
    pub equipment_type: EquipmentType,
    pub assembly_occurrence_id: String,
    pub parameter_set: EquipmentParameterSet,
    pub io_port_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotSequenceModel {
    pub id: String,
    pub cell_id: String,
    pub robot_id: String,
    pub target_ids: Vec<String>,
    pub path_length_mm: f64,
    pub estimated_cycle_time_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotTargetModel {
    pub id: String,
    pub cell_id: String,
    pub sequence_id: String,
    pub target_key: String,
    pub order_index: u32,
    pub pose: CartesianPose,
    pub nominal_speed_mm_s: u32,
    pub dwell_time_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotCellStructureSummary {
    pub robot_count: usize,
    pub equipment_count: usize,
    pub safety_zone_count: usize,
    pub sequence_count: usize,
    pub controller_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CartesianPose {
    pub x_mm: f64,
    pub y_mm: f64,
    pub z_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotTarget {
    pub id: String,
    pub pose: CartesianPose,
    pub nominal_speed_mm_s: u32,
    pub dwell_time_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceValidation {
    pub target_count: usize,
    pub path_length_mm: f64,
    pub max_segment_mm: f64,
    pub estimated_cycle_time_ms: u32,
    pub warning_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotCellControlModel {
    pub cell_id: String,
    pub signals: Vec<SignalDefinition>,
    pub controller: ControllerStateMachine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotCellControlSummary {
    pub signal_count: usize,
    pub controller_transition_count: usize,
    pub blocked_sequence_detected: bool,
    pub blocked_state_id: Option<String>,
}

#[derive(Debug, Error, PartialEq)]
pub enum RoboticsError {
    #[error("a robot sequence requires at least one target")]
    EmptySequence,
    #[error("target ids must be unique and speeds must be non-zero")]
    InvalidTargetSet,
    #[error("target poses must contain finite coordinates")]
    InvalidPose,
    #[error(
        "robot cell structure requires non-empty unique ids for scene, robot, equipment, safety, sequence and controller refs"
    )]
    InvalidRobotCellStructure,
    #[error("robot models must belong to the cell and match the declared robot ids")]
    InvalidRobotModelSet,
    #[error(
        "equipment models must belong to the cell and expose valid scene occurrence references"
    )]
    InvalidEquipmentModelSet,
    #[error(
        "robot sequences must belong to the cell and target declared robots with non-empty targets"
    )]
    InvalidRobotSequenceSet,
    #[error(
        "robot target models must belong to the sequence and expose unique ids, keys and positive order indexes"
    )]
    InvalidRobotTargetModelSet,
    #[error(
        "robot cell control requires a non-empty cell id, non-empty unique signal ids and a non-empty controller id and name"
    )]
    InvalidRobotCellControlModel,
    #[error("controller state machine must expose non-empty unique state ids and transition ids")]
    InvalidControllerStateMachine,
    #[error("controller initial state `{0}` is missing")]
    UnknownInitialControllerState(String),
    #[error("controller transition `{transition_id}` references missing state `{state_id}`")]
    UnknownControllerTransitionState {
        transition_id: String,
        state_id: String,
    },
    #[error("controller transition `{transition_id}` references missing signal `{signal_id}`")]
    UnknownControllerSignal {
        transition_id: String,
        signal_id: String,
    },
    #[error(
        "signal `{signal_id}` value is incompatible with kind `{expected_kind:?}` in {context}"
    )]
    InvalidSignalValueForKind {
        signal_id: String,
        expected_kind: SignalKind,
        context: &'static str,
    },
    #[error(
        "controller transition `{transition_id}` uses comparator `{comparator:?}` incompatible with signal `{signal_id}` kind `{kind:?}`"
    )]
    InvalidSignalComparator {
        transition_id: String,
        signal_id: String,
        comparator: SignalComparator,
        kind: SignalKind,
    },
}

pub fn validate_sequence(targets: &[RobotTarget]) -> Result<SequenceValidation, RoboticsError> {
    if targets.is_empty() {
        return Err(RoboticsError::EmptySequence);
    }

    let mut ids = HashSet::new();
    if targets
        .iter()
        .any(|target| target.nominal_speed_mm_s == 0 || !ids.insert(target.id.as_str()))
    {
        return Err(RoboticsError::InvalidTargetSet);
    }
    if targets.iter().any(|target| {
        !target.pose.x_mm.is_finite()
            || !target.pose.y_mm.is_finite()
            || !target.pose.z_mm.is_finite()
    }) {
        return Err(RoboticsError::InvalidPose);
    }

    let (path_length_mm, max_segment_mm, motion_time_ms) = targets.windows(2).fold(
        (0.0_f64, 0.0_f64, 0_u32),
        |(path_acc, segment_acc, time_acc), pair| {
            let distance = distance_between(pair[0].pose, pair[1].pose);
            let average_speed =
                ((pair[0].nominal_speed_mm_s + pair[1].nominal_speed_mm_s) / 2).max(1);
            let travel_time_ms = (distance / f64::from(average_speed) * 1_000.0).ceil() as u32;
            (
                path_acc + distance,
                segment_acc.max(distance),
                time_acc + travel_time_ms,
            )
        },
    );

    let dwell_time_ms = targets
        .iter()
        .map(|target| target.dwell_time_ms)
        .sum::<u32>();
    let warning_count =
        usize::from(max_segment_mm > 1_000.0) + usize::from(path_length_mm > 1_800.0);

    Ok(SequenceValidation {
        target_count: targets.len(),
        path_length_mm,
        max_segment_mm,
        estimated_cycle_time_ms: motion_time_ms + dwell_time_ms,
        warning_count,
    })
}

pub fn validate_target_models(
    cell_id: &str,
    sequence_id: &str,
    target_models: &[RobotTargetModel],
) -> Result<Vec<RobotTarget>, RoboticsError> {
    if target_models.is_empty() {
        return Err(RoboticsError::EmptySequence);
    }

    let mut entity_ids = HashSet::new();
    let mut target_keys = HashSet::new();
    let mut order_indexes = HashSet::new();
    if target_models.iter().any(|target| {
        target.id.is_empty()
            || target.cell_id != cell_id
            || target.sequence_id != sequence_id
            || target.target_key.is_empty()
            || target.order_index == 0
            || target.nominal_speed_mm_s == 0
            || !entity_ids.insert(target.id.as_str())
            || !target_keys.insert(target.target_key.as_str())
            || !order_indexes.insert(target.order_index)
    }) {
        return Err(RoboticsError::InvalidRobotTargetModelSet);
    }

    if target_models.iter().any(|target| {
        !target.pose.x_mm.is_finite()
            || !target.pose.y_mm.is_finite()
            || !target.pose.z_mm.is_finite()
    }) {
        return Err(RoboticsError::InvalidPose);
    }

    let mut ordered = target_models.to_vec();
    ordered.sort_by_key(|target| target.order_index);
    Ok(ordered
        .into_iter()
        .map(|target| RobotTarget {
            id: target.target_key,
            pose: target.pose,
            nominal_speed_mm_s: target.nominal_speed_mm_s,
            dwell_time_ms: target.dwell_time_ms,
        })
        .collect())
}

pub fn signal_value_matches_kind(kind: &SignalKind, value: &SignalValue) -> bool {
    matches!(
        (kind, value),
        (SignalKind::Boolean, SignalValue::Bool(_))
            | (SignalKind::Scalar, SignalValue::Scalar(_))
            | (SignalKind::Text, SignalValue::Text(_))
    )
}

pub fn control_signal_values(signals: &[SignalDefinition]) -> BTreeMap<String, SignalValue> {
    signals
        .iter()
        .map(|signal| (signal.id.clone(), signal.initial_value.clone()))
        .collect()
}

pub fn signal_condition_matches(
    condition: &SignalCondition,
    signal_values: &BTreeMap<String, SignalValue>,
) -> bool {
    let Some(current_value) = signal_values.get(&condition.signal_id) else {
        return false;
    };
    match (
        &condition.comparator,
        current_value,
        &condition.expected_value,
    ) {
        (SignalComparator::Equal, left, right) => left == right,
        (SignalComparator::NotEqual, left, right) => left != right,
        (SignalComparator::GreaterThan, SignalValue::Scalar(left), SignalValue::Scalar(right)) => {
            left > right
        }
        (
            SignalComparator::GreaterThanOrEqual,
            SignalValue::Scalar(left),
            SignalValue::Scalar(right),
        ) => left >= right,
        (SignalComparator::LessThan, SignalValue::Scalar(left), SignalValue::Scalar(right)) => {
            left < right
        }
        (
            SignalComparator::LessThanOrEqual,
            SignalValue::Scalar(left),
            SignalValue::Scalar(right),
        ) => left <= right,
        _ => false,
    }
}

pub fn apply_control_assignments(
    signal_values: &mut BTreeMap<String, SignalValue>,
    assignments: &[SignalAssignment],
) {
    for assignment in assignments {
        signal_values.insert(assignment.signal_id.clone(), assignment.value.clone());
    }
}

pub fn resolve_initial_control_state(
    control: &RobotCellControlModel,
) -> Result<ControllerState, RoboticsError> {
    control
        .controller
        .states
        .iter()
        .find(|state| state.id == control.controller.initial_state_id)
        .cloned()
        .ok_or_else(|| {
            RoboticsError::UnknownInitialControllerState(
                control.controller.initial_state_id.clone(),
            )
        })
}

pub fn resolve_control_transition_state(
    controller: &ControllerStateMachine,
    transition: &ControlTransition,
) -> Result<ControllerState, RoboticsError> {
    controller
        .states
        .iter()
        .find(|state| state.id == transition.to_state_id)
        .cloned()
        .ok_or_else(|| RoboticsError::UnknownControllerTransitionState {
            transition_id: transition.id.clone(),
            state_id: transition.to_state_id.clone(),
        })
}

pub fn enabled_control_transition<'a>(
    control: &'a RobotCellControlModel,
    current_state_id: &str,
    signal_values: &BTreeMap<String, SignalValue>,
) -> Option<&'a ControlTransition> {
    control
        .controller
        .transitions
        .iter()
        .filter(|transition| transition.from_state_id == current_state_id)
        .find(|transition| {
            transition
                .conditions
                .iter()
                .all(|condition| signal_condition_matches(condition, signal_values))
        })
}

pub fn validate_robot_cell_control(control: &RobotCellControlModel) -> Result<(), RoboticsError> {
    if control.cell_id.trim().is_empty()
        || control.controller.id.trim().is_empty()
        || control.controller.name.trim().is_empty()
        || control.signals.is_empty()
    {
        return Err(RoboticsError::InvalidRobotCellControlModel);
    }

    let mut signal_ids = HashSet::new();
    let mut signal_kinds = BTreeMap::new();
    for signal in &control.signals {
        if signal.id.trim().is_empty()
            || signal.name.trim().is_empty()
            || !signal_ids.insert(signal.id.as_str())
        {
            return Err(RoboticsError::InvalidRobotCellControlModel);
        }
        if !signal_value_matches_kind(&signal.kind, &signal.initial_value) {
            return Err(RoboticsError::InvalidSignalValueForKind {
                signal_id: signal.id.clone(),
                expected_kind: signal.kind.clone(),
                context: "signal.initialValue",
            });
        }
        signal_kinds.insert(signal.id.clone(), signal.kind.clone());
    }

    if control.controller.initial_state_id.trim().is_empty() || control.controller.states.is_empty()
    {
        return Err(RoboticsError::InvalidControllerStateMachine);
    }

    let mut state_ids = HashSet::new();
    for state in &control.controller.states {
        if state.id.trim().is_empty()
            || state.name.trim().is_empty()
            || !state_ids.insert(state.id.as_str())
        {
            return Err(RoboticsError::InvalidControllerStateMachine);
        }
    }
    if !state_ids.contains(control.controller.initial_state_id.as_str()) {
        return Err(RoboticsError::UnknownInitialControllerState(
            control.controller.initial_state_id.clone(),
        ));
    }

    let mut transition_ids = HashSet::new();
    for transition in &control.controller.transitions {
        if transition.id.trim().is_empty()
            || transition.from_state_id.trim().is_empty()
            || transition.to_state_id.trim().is_empty()
            || !transition_ids.insert(transition.id.as_str())
        {
            return Err(RoboticsError::InvalidControllerStateMachine);
        }
        if !state_ids.contains(transition.from_state_id.as_str()) {
            return Err(RoboticsError::UnknownControllerTransitionState {
                transition_id: transition.id.clone(),
                state_id: transition.from_state_id.clone(),
            });
        }
        if !state_ids.contains(transition.to_state_id.as_str()) {
            return Err(RoboticsError::UnknownControllerTransitionState {
                transition_id: transition.id.clone(),
                state_id: transition.to_state_id.clone(),
            });
        }

        for condition in &transition.conditions {
            let Some(kind) = signal_kinds.get(&condition.signal_id) else {
                return Err(RoboticsError::UnknownControllerSignal {
                    transition_id: transition.id.clone(),
                    signal_id: condition.signal_id.clone(),
                });
            };
            if !signal_value_matches_kind(kind, &condition.expected_value) {
                return Err(RoboticsError::InvalidSignalValueForKind {
                    signal_id: condition.signal_id.clone(),
                    expected_kind: kind.clone(),
                    context: "condition.expectedValue",
                });
            }
            if !signal_comparator_supported(kind, &condition.comparator) {
                return Err(RoboticsError::InvalidSignalComparator {
                    transition_id: transition.id.clone(),
                    signal_id: condition.signal_id.clone(),
                    comparator: condition.comparator.clone(),
                    kind: kind.clone(),
                });
            }
        }

        for assignment in &transition.assignments {
            let Some(kind) = signal_kinds.get(&assignment.signal_id) else {
                return Err(RoboticsError::UnknownControllerSignal {
                    transition_id: transition.id.clone(),
                    signal_id: assignment.signal_id.clone(),
                });
            };
            if !signal_value_matches_kind(kind, &assignment.value) {
                return Err(RoboticsError::InvalidSignalValueForKind {
                    signal_id: assignment.signal_id.clone(),
                    expected_kind: kind.clone(),
                    context: "assignment.value",
                });
            }
        }
    }

    Ok(())
}

pub fn summarize_robot_cell_control(
    control: &RobotCellControlModel,
) -> Result<RobotCellControlSummary, RoboticsError> {
    validate_robot_cell_control(control)?;

    let mut signal_values = control_signal_values(&control.signals);
    let mut current_state = resolve_initial_control_state(control)?;
    let max_steps = control.controller.transitions.len() + control.controller.states.len() + 1;
    let mut steps = 0usize;

    while !current_state.terminal && steps < max_steps {
        let Some(transition) =
            enabled_control_transition(control, &current_state.id, &signal_values)
        else {
            break;
        };
        apply_control_assignments(&mut signal_values, &transition.assignments);
        current_state = resolve_control_transition_state(&control.controller, transition)?;
        steps += 1;
    }

    let blocked_sequence_detected = !current_state.terminal;
    Ok(RobotCellControlSummary {
        signal_count: control.signals.len(),
        controller_transition_count: control.controller.transitions.len(),
        blocked_sequence_detected,
        blocked_state_id: blocked_sequence_detected.then_some(current_state.id),
    })
}

pub fn validate_robot_cell_structure(
    cell: &RobotCellModel,
    robots: &[RobotModel],
    equipment: &[EquipmentModel],
    sequences: &[RobotSequenceModel],
) -> Result<RobotCellStructureSummary, RoboticsError> {
    if cell.id.is_empty()
        || cell.scene_assembly_id.is_empty()
        || cell.robot_ids.is_empty()
        || cell.equipment_ids.is_empty()
        || cell.safety_zone_ids.is_empty()
        || cell.sequence_ids.is_empty()
        || cell.controller_model_ids.is_empty()
        || !ids_are_unique(&cell.robot_ids)
        || !ids_are_unique(&cell.equipment_ids)
        || !ids_are_unique(&cell.safety_zone_ids)
        || !ids_are_unique(&cell.sequence_ids)
        || !ids_are_unique(&cell.controller_model_ids)
    {
        return Err(RoboticsError::InvalidRobotCellStructure);
    }

    let robot_ids = robots
        .iter()
        .map(|robot| robot.id.as_str())
        .collect::<Vec<_>>();
    if robots.len() != cell.robot_ids.len()
        || !same_id_set(&cell.robot_ids, &robot_ids)
        || robots.iter().any(|robot| {
            robot.cell_id != cell.id
                || robot.kinematic_chain.is_empty()
                || robot.joint_ids.is_empty()
                || robot.tool_mount_ref.equipment_id.is_empty()
                || robot.tool_mount_ref.role.is_empty()
                || !robot.workspace_bounds.reach_radius_mm.is_finite()
                || robot.workspace_bounds.reach_radius_mm <= 0.0
                || !robot.workspace_bounds.vertical_span_mm.is_finite()
                || robot.workspace_bounds.vertical_span_mm <= 0.0
                || !robot.payload_limits.nominal_kg.is_finite()
                || robot.payload_limits.nominal_kg <= 0.0
                || !robot.payload_limits.max_kg.is_finite()
                || robot.payload_limits.max_kg < robot.payload_limits.nominal_kg
                || robot.calibration_state.is_empty()
        })
    {
        return Err(RoboticsError::InvalidRobotModelSet);
    }

    let equipment_ids = equipment
        .iter()
        .map(|model| model.id.as_str())
        .collect::<Vec<_>>();
    if equipment.len() != cell.equipment_ids.len()
        || !same_id_set(&cell.equipment_ids, &equipment_ids)
        || equipment.iter().any(|model| {
            model.cell_id != cell.id
                || model.assembly_occurrence_id.is_empty()
                || !model.parameter_set.width_mm.is_finite()
                || model.parameter_set.width_mm <= 0.0
                || !model.parameter_set.height_mm.is_finite()
                || model.parameter_set.height_mm <= 0.0
                || !model.parameter_set.depth_mm.is_finite()
                || model.parameter_set.depth_mm <= 0.0
        })
    {
        return Err(RoboticsError::InvalidEquipmentModelSet);
    }

    let sequence_ids = sequences
        .iter()
        .map(|sequence| sequence.id.as_str())
        .collect::<Vec<_>>();
    if sequences.len() != cell.sequence_ids.len()
        || !same_id_set(&cell.sequence_ids, &sequence_ids)
        || sequences.iter().any(|sequence| {
            sequence.cell_id != cell.id
                || !cell
                    .robot_ids
                    .iter()
                    .any(|robot_id| robot_id == &sequence.robot_id)
                || sequence.target_ids.is_empty()
                || !sequence.path_length_mm.is_finite()
                || sequence.path_length_mm <= 0.0
                || sequence.estimated_cycle_time_ms == 0
        })
    {
        return Err(RoboticsError::InvalidRobotSequenceSet);
    }

    Ok(RobotCellStructureSummary {
        robot_count: cell.robot_ids.len(),
        equipment_count: cell.equipment_ids.len(),
        safety_zone_count: cell.safety_zone_ids.len(),
        sequence_count: cell.sequence_ids.len(),
        controller_count: cell.controller_model_ids.len(),
    })
}

fn distance_between(left: CartesianPose, right: CartesianPose) -> f64 {
    let dx = right.x_mm - left.x_mm;
    let dy = right.y_mm - left.y_mm;
    let dz = right.z_mm - left.z_mm;

    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn ids_are_unique(ids: &[String]) -> bool {
    let mut known = HashSet::new();
    ids.iter()
        .all(|id| !id.is_empty() && known.insert(id.as_str()))
}

fn same_id_set(expected: &[String], actual: &[&str]) -> bool {
    let expected_set = expected.iter().map(String::as_str).collect::<HashSet<_>>();
    let actual_set = actual.iter().copied().collect::<HashSet<_>>();
    expected_set == actual_set
}

fn signal_comparator_supported(kind: &SignalKind, comparator: &SignalComparator) -> bool {
    matches!(
        (kind, comparator),
        (SignalKind::Scalar, _)
            | (
                SignalKind::Boolean,
                SignalComparator::Equal | SignalComparator::NotEqual
            )
            | (
                SignalKind::Text,
                SignalComparator::Equal | SignalComparator::NotEqual
            )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nominal_targets() -> Vec<RobotTarget> {
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

    #[test]
    fn validates_a_nominal_robot_sequence_with_path_metrics() {
        let validation = validate_sequence(&nominal_targets()).expect("sequence should validate");

        assert_eq!(validation.target_count, 3);
        assert!(validation.path_length_mm > 850.0);
        assert!(validation.max_segment_mm > 400.0);
        assert_eq!(validation.estimated_cycle_time_ms, 3_491);
        assert_eq!(validation.warning_count, 0);
    }

    #[test]
    fn warns_when_reach_segments_are_long() {
        let mut targets = nominal_targets();
        targets[2].pose.x_mm = 1_900.0;

        let validation = validate_sequence(&targets).expect("sequence should validate");

        assert!(validation.max_segment_mm > 1_000.0);
        assert!(validation.warning_count >= 1);
    }

    #[test]
    fn rejects_empty_sequences() {
        assert_eq!(validate_sequence(&[]), Err(RoboticsError::EmptySequence));
    }

    #[test]
    fn rejects_duplicate_targets_or_zero_speed() {
        assert_eq!(
            validate_sequence(&[
                RobotTarget {
                    id: "dup".to_string(),
                    pose: CartesianPose::default(),
                    nominal_speed_mm_s: 100,
                    dwell_time_ms: 0,
                },
                RobotTarget {
                    id: "dup".to_string(),
                    pose: CartesianPose::default(),
                    nominal_speed_mm_s: 100,
                    dwell_time_ms: 0,
                },
            ]),
            Err(RoboticsError::InvalidTargetSet)
        );
        assert_eq!(
            validate_sequence(&[RobotTarget {
                id: "bad".to_string(),
                pose: CartesianPose::default(),
                nominal_speed_mm_s: 0,
                dwell_time_ms: 0,
            }]),
            Err(RoboticsError::InvalidTargetSet)
        );
    }

    #[test]
    fn rejects_non_finite_target_positions() {
        assert_eq!(
            validate_sequence(&[RobotTarget {
                id: "bad_pose".to_string(),
                pose: CartesianPose {
                    x_mm: f64::NAN,
                    y_mm: 0.0,
                    z_mm: 0.0,
                },
                nominal_speed_mm_s: 100,
                dwell_time_ms: 0,
            }]),
            Err(RoboticsError::InvalidPose)
        );
    }

    fn sample_robot_cell_model() -> RobotCellModel {
        RobotCellModel {
            id: "ent_cell_001".to_string(),
            scene_assembly_id: "ent_asm_cell_001".to_string(),
            robot_ids: vec!["ent_robot_001".to_string()],
            equipment_ids: vec![
                "ent_conveyor_001".to_string(),
                "ent_fixture_001".to_string(),
                "ent_tool_001".to_string(),
            ],
            safety_zone_ids: vec![
                "ent_zone_001_warning".to_string(),
                "ent_zone_001_protective".to_string(),
            ],
            sequence_ids: vec!["ent_seq_001".to_string()],
            controller_model_ids: vec!["ent_ctrl_001".to_string()],
        }
    }

    fn sample_robot_model() -> RobotModel {
        RobotModel {
            id: "ent_robot_001".to_string(),
            cell_id: "ent_cell_001".to_string(),
            kinematic_chain: vec![
                "base".to_string(),
                "shoulder".to_string(),
                "wrist".to_string(),
                "tool".to_string(),
            ],
            joint_ids: vec!["joint_axis_001".to_string()],
            tool_mount_ref: RobotToolMountRef {
                equipment_id: "ent_tool_001".to_string(),
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

    fn sample_equipment_models() -> Vec<EquipmentModel> {
        vec![
            EquipmentModel {
                id: "ent_conveyor_001".to_string(),
                cell_id: "ent_cell_001".to_string(),
                equipment_type: EquipmentType::Conveyor,
                assembly_occurrence_id: "occ_conveyor_001".to_string(),
                parameter_set: EquipmentParameterSet {
                    width_mm: 850.0,
                    height_mm: 220.0,
                    depth_mm: 600.0,
                    nominal_speed_mm_s: Some(320),
                },
                io_port_ids: vec!["sig_cycle_start".to_string()],
            },
            EquipmentModel {
                id: "ent_fixture_001".to_string(),
                cell_id: "ent_cell_001".to_string(),
                equipment_type: EquipmentType::Workstation,
                assembly_occurrence_id: "occ_fixture_001".to_string(),
                parameter_set: EquipmentParameterSet {
                    width_mm: 640.0,
                    height_mm: 180.0,
                    depth_mm: 480.0,
                    nominal_speed_mm_s: None,
                },
                io_port_ids: vec!["sig_progress_gate".to_string()],
            },
            EquipmentModel {
                id: "ent_tool_001".to_string(),
                cell_id: "ent_cell_001".to_string(),
                equipment_type: EquipmentType::Gripper,
                assembly_occurrence_id: "occ_tool_001".to_string(),
                parameter_set: EquipmentParameterSet {
                    width_mm: 110.0,
                    height_mm: 80.0,
                    depth_mm: 140.0,
                    nominal_speed_mm_s: None,
                },
                io_port_ids: vec!["sig_payload_released".to_string()],
            },
        ]
    }

    fn sample_robot_sequence_model() -> RobotSequenceModel {
        RobotSequenceModel {
            id: "ent_seq_001".to_string(),
            cell_id: "ent_cell_001".to_string(),
            robot_id: "ent_robot_001".to_string(),
            target_ids: vec![
                "ent_target_001_pick".to_string(),
                "ent_target_001_transfer".to_string(),
                "ent_target_001_place".to_string(),
            ],
            path_length_mm: 896.0,
            estimated_cycle_time_ms: 3_491,
        }
    }

    fn sample_target_models() -> Vec<RobotTargetModel> {
        vec![
            RobotTargetModel {
                id: "ent_target_001_pick".to_string(),
                cell_id: "ent_cell_001".to_string(),
                sequence_id: "ent_seq_001".to_string(),
                target_key: "pick".to_string(),
                order_index: 1,
                pose: CartesianPose {
                    x_mm: 0.0,
                    y_mm: 0.0,
                    z_mm: 120.0,
                },
                nominal_speed_mm_s: 250,
                dwell_time_ms: 120,
            },
            RobotTargetModel {
                id: "ent_target_001_transfer".to_string(),
                cell_id: "ent_cell_001".to_string(),
                sequence_id: "ent_seq_001".to_string(),
                target_key: "transfer".to_string(),
                order_index: 2,
                pose: CartesianPose {
                    x_mm: 450.0,
                    y_mm: 60.0,
                    z_mm: 240.0,
                },
                nominal_speed_mm_s: 320,
                dwell_time_ms: 40,
            },
            RobotTargetModel {
                id: "ent_target_001_place".to_string(),
                cell_id: "ent_cell_001".to_string(),
                sequence_id: "ent_seq_001".to_string(),
                target_key: "place".to_string(),
                order_index: 3,
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

    #[test]
    fn validates_robot_cell_structure_with_consistent_support_graph() {
        let summary = validate_robot_cell_structure(
            &sample_robot_cell_model(),
            &[sample_robot_model()],
            &sample_equipment_models(),
            &[sample_robot_sequence_model()],
        )
        .expect("structure should validate");

        assert_eq!(
            summary,
            RobotCellStructureSummary {
                robot_count: 1,
                equipment_count: 3,
                safety_zone_count: 2,
                sequence_count: 1,
                controller_count: 1,
            }
        );
    }

    #[test]
    fn rejects_robot_cell_structure_with_invalid_refs_or_counts() {
        let mut cell = sample_robot_cell_model();
        cell.equipment_ids.push("ent_fixture_001".to_string());
        assert_eq!(
            validate_robot_cell_structure(
                &cell,
                &[sample_robot_model()],
                &sample_equipment_models(),
                &[sample_robot_sequence_model()],
            ),
            Err(RoboticsError::InvalidRobotCellStructure)
        );

        let mut bad_sequence = sample_robot_sequence_model();
        bad_sequence.robot_id = "ent_robot_missing".to_string();
        assert_eq!(
            validate_robot_cell_structure(
                &sample_robot_cell_model(),
                &[sample_robot_model()],
                &sample_equipment_models(),
                &[bad_sequence],
            ),
            Err(RoboticsError::InvalidRobotSequenceSet)
        );
    }

    #[test]
    fn validates_target_models_and_preserves_ordered_robot_targets() {
        let ordered =
            validate_target_models("ent_cell_001", "ent_seq_001", &sample_target_models())
                .expect("target models should validate");

        assert_eq!(
            ordered
                .iter()
                .map(|target| target.id.as_str())
                .collect::<Vec<_>>(),
            vec!["pick", "transfer", "place"]
        );
        assert_eq!(ordered[1].pose.x_mm, 450.0);
    }

    #[test]
    fn rejects_target_models_with_duplicate_order_or_invalid_scope() {
        let mut models = sample_target_models();
        models[1].order_index = 1;
        assert_eq!(
            validate_target_models("ent_cell_001", "ent_seq_001", &models),
            Err(RoboticsError::InvalidRobotTargetModelSet)
        );

        let mut wrong_scope = sample_target_models();
        wrong_scope[0].sequence_id = "ent_seq_other".to_string();
        assert_eq!(
            validate_target_models("ent_cell_001", "ent_seq_001", &wrong_scope),
            Err(RoboticsError::InvalidRobotTargetModelSet)
        );
    }

    fn sample_control_model() -> RobotCellControlModel {
        RobotCellControlModel {
            cell_id: "ent_cell_001".to_string(),
            signals: vec![
                SignalDefinition {
                    id: "sig_cycle_start".to_string(),
                    name: "Cycle Start".to_string(),
                    kind: SignalKind::Boolean,
                    initial_value: SignalValue::Bool(false),
                    unit: None,
                    tags: vec!["control".to_string()],
                },
                SignalDefinition {
                    id: "sig_progress_gate".to_string(),
                    name: "Progress Gate".to_string(),
                    kind: SignalKind::Scalar,
                    initial_value: SignalValue::Scalar(0.62),
                    unit: Some("ratio".to_string()),
                    tags: vec!["control".to_string()],
                },
                SignalDefinition {
                    id: "sig_safety_clear".to_string(),
                    name: "Safety Clear".to_string(),
                    kind: SignalKind::Boolean,
                    initial_value: SignalValue::Bool(true),
                    unit: None,
                    tags: vec!["safety".to_string()],
                },
                SignalDefinition {
                    id: "sig_payload_released".to_string(),
                    name: "Payload Released".to_string(),
                    kind: SignalKind::Boolean,
                    initial_value: SignalValue::Bool(false),
                    unit: None,
                    tags: vec!["process".to_string()],
                },
                SignalDefinition {
                    id: "sig_operator_mode".to_string(),
                    name: "Operator Mode".to_string(),
                    kind: SignalKind::Text,
                    initial_value: SignalValue::Text("auto".to_string()),
                    unit: None,
                    tags: vec!["control".to_string(), "text".to_string()],
                },
            ],
            controller: ControllerStateMachine {
                id: "ctrl_001".to_string(),
                name: "Controller 001".to_string(),
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
            },
        }
    }

    #[test]
    fn control_summary_reports_blocked_initial_state_deterministically() {
        let summary =
            summarize_robot_cell_control(&sample_control_model()).expect("control should validate");

        assert_eq!(
            summary,
            RobotCellControlSummary {
                signal_count: 5,
                controller_transition_count: 3,
                blocked_sequence_detected: true,
                blocked_state_id: Some("idle".to_string()),
            }
        );
    }

    #[test]
    fn control_summary_reaches_terminal_state_when_conditions_are_met() {
        let mut control = sample_control_model();
        control.signals[0].initial_value = SignalValue::Bool(true);
        control.signals[1].initial_value = SignalValue::Scalar(1.0);

        let summary =
            summarize_robot_cell_control(&control).expect("control should reach terminal");

        assert_eq!(summary.signal_count, 5);
        assert_eq!(summary.controller_transition_count, 3);
        assert!(!summary.blocked_sequence_detected);
        assert_eq!(summary.blocked_state_id, None);
    }

    #[test]
    fn rejects_invalid_signal_values_for_declared_kind() {
        let mut control = sample_control_model();
        control.signals[0].initial_value = SignalValue::Scalar(1.0);

        assert_eq!(
            validate_robot_cell_control(&control),
            Err(RoboticsError::InvalidSignalValueForKind {
                signal_id: "sig_cycle_start".to_string(),
                expected_kind: SignalKind::Boolean,
                context: "signal.initialValue",
            })
        );

        control = sample_control_model();
        control.controller.transitions[2].assignments[0].value =
            SignalValue::Text("released".to_string());
        assert_eq!(
            validate_robot_cell_control(&control),
            Err(RoboticsError::InvalidSignalValueForKind {
                signal_id: "sig_payload_released".to_string(),
                expected_kind: SignalKind::Boolean,
                context: "assignment.value",
            })
        );
    }

    #[test]
    fn rejects_missing_controller_states_and_signal_references() {
        let mut control = sample_control_model();
        control.controller.initial_state_id = "missing".to_string();
        assert_eq!(
            validate_robot_cell_control(&control),
            Err(RoboticsError::UnknownInitialControllerState(
                "missing".to_string()
            ))
        );

        control = sample_control_model();
        control.controller.transitions[0].to_state_id = "missing".to_string();
        assert_eq!(
            validate_robot_cell_control(&control),
            Err(RoboticsError::UnknownControllerTransitionState {
                transition_id: "tr_start_cycle".to_string(),
                state_id: "missing".to_string(),
            })
        );

        control = sample_control_model();
        control.controller.transitions[1].conditions[0].signal_id = "sig_missing".to_string();
        assert_eq!(
            validate_robot_cell_control(&control),
            Err(RoboticsError::UnknownControllerSignal {
                transition_id: "tr_reach_place".to_string(),
                signal_id: "sig_missing".to_string(),
            })
        );
    }

    #[test]
    fn rejects_incompatible_signal_comparators() {
        let mut control = sample_control_model();
        control.controller.transitions[0].conditions[0].comparator = SignalComparator::GreaterThan;

        assert_eq!(
            validate_robot_cell_control(&control),
            Err(RoboticsError::InvalidSignalComparator {
                transition_id: "tr_start_cycle".to_string(),
                signal_id: "sig_cycle_start".to_string(),
                comparator: SignalComparator::GreaterThan,
                kind: SignalKind::Boolean,
            })
        );
    }
}
