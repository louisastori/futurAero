use std::collections::BTreeMap;

use faero_types::{
    ControllerState, ControllerStateMachine, ControllerStateSample, ScheduledSignalChange,
    SignalComparator, SignalCondition, SignalDefinition, SignalSample, SignalValue,
    SimulationContact, SimulationContactPair, SimulationProgressSample,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_ENGINE_VERSION: &str = "faero-sim@0.2.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRequest {
    pub scenario_name: String,
    pub seed: u64,
    pub engine_version: String,
    pub step_count: u32,
    pub planned_cycle_time_ms: u32,
    pub path_length_mm: f64,
    pub endpoint_count: u32,
    pub safety_zone_count: u32,
    pub signals: Vec<SignalDefinition>,
    pub controller: Option<ControllerStateMachine>,
    pub scheduled_signal_changes: Vec<ScheduledSignalChange>,
    pub contact_pairs: Vec<SimulationContactPair>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationStatus {
    Completed,
    Warning,
    Collided,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineSample {
    pub step_index: u32,
    pub timestamp_ms: u32,
    pub tracking_error_mm: f64,
    pub speed_scale: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationSummary {
    pub status: SimulationStatus,
    pub seed: u64,
    pub engine_version: String,
    pub collision_count: u32,
    pub cycle_time_ms: u32,
    pub max_tracking_error_mm: f64,
    pub energy_estimate_j: f64,
    pub blocked_sequence_detected: bool,
    pub blocked_state_id: Option<String>,
    pub timeline_samples: Vec<TimelineSample>,
    pub signal_samples: Vec<SignalSample>,
    pub controller_state_samples: Vec<ControllerStateSample>,
    pub contacts: Vec<SimulationContact>,
    pub progress_samples: Vec<SimulationProgressSample>,
}

#[derive(Debug, Error, PartialEq)]
pub enum SimulationError {
    #[error("simulation step count must be greater than zero")]
    InvalidStepCount,
    #[error("planned cycle time must be greater than zero")]
    InvalidCycleTime,
    #[error("path length must be finite and non-negative")]
    InvalidPathLength,
    #[error("controller initial state `{0}` is missing")]
    UnknownInitialState(String),
    #[error("controller transition `{transition_id}` references missing state `{state_id}`")]
    UnknownTransitionState {
        transition_id: String,
        state_id: String,
    },
}

pub fn run_simulation(request: &SimulationRequest) -> Result<SimulationSummary, SimulationError> {
    if request.step_count == 0 {
        return Err(SimulationError::InvalidStepCount);
    }
    if request.planned_cycle_time_ms == 0 {
        return Err(SimulationError::InvalidCycleTime);
    }
    if !request.path_length_mm.is_finite() || request.path_length_mm < 0.0 {
        return Err(SimulationError::InvalidPathLength);
    }

    let latency_penalty_ms = request.endpoint_count * 6;
    let seed_jitter_ms = (request.seed % 11) as u32;
    let cycle_time_ms = request.planned_cycle_time_ms + latency_penalty_ms + seed_jitter_ms;
    let base_tracking_error_mm =
        (request.path_length_mm / f64::from(request.step_count.max(1))) / 400.0;
    let seed_tracking_error_mm = (request.seed % 17) as f64 * 0.04;
    let max_tracking_error_mm = round_two_decimals(base_tracking_error_mm + seed_tracking_error_mm);
    let energy_estimate_j = round_two_decimals(
        request.path_length_mm * 0.035
            + f64::from(cycle_time_ms) * 0.012
            + f64::from(request.endpoint_count) * 1.5,
    );

    let mut signal_values = request
        .signals
        .iter()
        .map(|signal| (signal.id.clone(), signal.initial_value.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut signal_samples = request
        .signals
        .iter()
        .map(|signal| SignalSample {
            step_index: 0,
            timestamp_ms: 0,
            signal_id: signal.id.clone(),
            value: signal.initial_value.clone(),
            reason: "initial_value".to_string(),
        })
        .collect::<Vec<_>>();

    let mut controller_state_samples = Vec::new();
    let mut current_state = resolve_initial_state(request.controller.as_ref())?;
    if let Some(state) = current_state.as_ref() {
        controller_state_samples.push(ControllerStateSample {
            step_index: 0,
            timestamp_ms: 0,
            state_id: state.id.clone(),
            state_name: state.name.clone(),
            reason: "initial_state".to_string(),
        });
    }

    let mut timeline_samples = Vec::new();
    let sampled_step_count = request.step_count.min(12);
    let mut sorted_changes = request.scheduled_signal_changes.clone();
    sorted_changes.sort_by(|left, right| {
        left.step_index
            .cmp(&right.step_index)
            .then_with(|| left.signal_id.cmp(&right.signal_id))
    });

    let mut change_cursor = 0usize;
    for step_index in 0..request.step_count {
        let timestamp_ms = step_timestamp_ms(step_index, request.step_count, cycle_time_ms);

        while change_cursor < sorted_changes.len()
            && sorted_changes[change_cursor].step_index == step_index
        {
            let change = &sorted_changes[change_cursor];
            signal_values.insert(change.signal_id.clone(), change.value.clone());
            signal_samples.push(SignalSample {
                step_index,
                timestamp_ms,
                signal_id: change.signal_id.clone(),
                value: change.value.clone(),
                reason: change.reason.clone(),
            });
            change_cursor += 1;
        }

        if let (Some(controller), Some(state)) = (request.controller.as_ref(), current_state.clone())
        {
            if let Some(transition) = controller
                .transitions
                .iter()
                .filter(|transition| transition.from_state_id == state.id)
                .find(|transition| {
                    transition
                        .conditions
                        .iter()
                        .all(|condition| signal_condition_matches(condition, &signal_values))
                })
            {
                for assignment in &transition.assignments {
                    signal_values.insert(assignment.signal_id.clone(), assignment.value.clone());
                    signal_samples.push(SignalSample {
                        step_index,
                        timestamp_ms,
                        signal_id: assignment.signal_id.clone(),
                        value: assignment.value.clone(),
                        reason: format!("transition:{}", transition.id),
                    });
                }
                let next_state = resolve_transition_state(controller, transition)?;
                controller_state_samples.push(ControllerStateSample {
                    step_index,
                    timestamp_ms,
                    state_id: next_state.id.clone(),
                    state_name: next_state.name.clone(),
                    reason: transition
                        .description
                        .clone()
                        .unwrap_or_else(|| transition.id.clone()),
                });
                current_state = Some(next_state);
            }
        }

        if step_index < sampled_step_count {
            let progress = if request.step_count == 1 {
                0.0
            } else {
                f64::from(step_index) / f64::from(request.step_count - 1)
            };
            timeline_samples.push(TimelineSample {
                step_index,
                timestamp_ms,
                tracking_error_mm: round_two_decimals(
                    max_tracking_error_mm * (0.45 + progress * 0.55),
                ),
                speed_scale: round_two_decimals((0.82 + progress * 0.18).min(1.0)),
            });
        }
    }

    let blocked_sequence_detected = current_state
        .as_ref()
        .map(|state| !state.terminal)
        .unwrap_or(false);
    let blocked_state_id = current_state
        .as_ref()
        .filter(|state| !state.terminal)
        .map(|state| state.id.clone());
    if let Some(state) = current_state.as_ref().filter(|state| !state.terminal) {
        controller_state_samples.push(ControllerStateSample {
            step_index: request.step_count - 1,
            timestamp_ms: cycle_time_ms,
            state_id: state.id.clone(),
            state_name: state.name.clone(),
            reason: "sequence_blocked".to_string(),
        });
    }

    let collision_count = compute_collision_count(request);
    let contacts = build_contacts(request, collision_count, cycle_time_ms);
    let status = if collision_count > 0 {
        SimulationStatus::Collided
    } else if blocked_sequence_detected || max_tracking_error_mm > 2.5 {
        SimulationStatus::Warning
    } else {
        SimulationStatus::Completed
    };
    let progress_samples = vec![
        SimulationProgressSample {
            phase: "queued".to_string(),
            progress: 0.0,
            message: format!("job queued for {}", request.scenario_name),
        },
        SimulationProgressSample {
            phase: "running".to_string(),
            progress: 0.35,
            message: format!("{} steps scheduled", request.step_count),
        },
        SimulationProgressSample {
            phase: "trace_persisted".to_string(),
            progress: 0.78,
            message: format!(
                "{} timeline | {} signal changes | {} controller states",
                timeline_samples.len(),
                signal_samples.len(),
                controller_state_samples.len()
            ),
        },
        SimulationProgressSample {
            phase: "completed".to_string(),
            progress: 1.0,
            message: if blocked_sequence_detected {
                "run completed with blocked sequence".to_string()
            } else {
                "run completed successfully".to_string()
            },
        },
    ];

    Ok(SimulationSummary {
        status,
        seed: request.seed,
        engine_version: if request.engine_version.trim().is_empty() {
            DEFAULT_ENGINE_VERSION.to_string()
        } else {
            request.engine_version.clone()
        },
        collision_count,
        cycle_time_ms,
        max_tracking_error_mm,
        energy_estimate_j,
        blocked_sequence_detected,
        blocked_state_id,
        timeline_samples,
        signal_samples,
        controller_state_samples,
        contacts,
        progress_samples,
    })
}

fn resolve_initial_state(
    controller: Option<&ControllerStateMachine>,
) -> Result<Option<ControllerState>, SimulationError> {
    let Some(controller) = controller else {
        return Ok(None);
    };
    controller
        .states
        .iter()
        .find(|state| state.id == controller.initial_state_id)
        .cloned()
        .map(Some)
        .ok_or_else(|| SimulationError::UnknownInitialState(controller.initial_state_id.clone()))
}

fn resolve_transition_state(
    controller: &ControllerStateMachine,
    transition: &faero_types::ControlTransition,
) -> Result<ControllerState, SimulationError> {
    controller
        .states
        .iter()
        .find(|state| state.id == transition.to_state_id)
        .cloned()
        .ok_or_else(|| SimulationError::UnknownTransitionState {
            transition_id: transition.id.clone(),
            state_id: transition.to_state_id.clone(),
        })
}

fn signal_condition_matches(
    condition: &SignalCondition,
    signal_values: &BTreeMap<String, SignalValue>,
) -> bool {
    let Some(current_value) = signal_values.get(&condition.signal_id) else {
        return false;
    };
    match (&condition.comparator, current_value, &condition.expected_value) {
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

fn compute_collision_count(request: &SimulationRequest) -> u32 {
    if request.safety_zone_count == 0 {
        ((request.seed + u64::from(request.step_count)) % 3) as u32
    } else if request.endpoint_count > 2 {
        ((request.seed + u64::from(request.endpoint_count)) % 2) as u32
    } else {
        0
    }
}

fn build_contacts(
    request: &SimulationRequest,
    collision_count: u32,
    cycle_time_ms: u32,
) -> Vec<SimulationContact> {
    if collision_count == 0 {
        return Vec::new();
    }

    let default_pairs = vec![SimulationContactPair {
        id: "pair_default".to_string(),
        left_entity_id: "ent_tool_001".to_string(),
        right_entity_id: "ent_fixture_001".to_string(),
        base_clearance_mm: 0.8,
    }];
    let pairs = if request.contact_pairs.is_empty() {
        &default_pairs
    } else {
        &request.contact_pairs
    };

    (0..collision_count)
        .map(|index| {
            let pair = &pairs[index as usize % pairs.len()];
            let step_index = (((index + 1) * request.step_count)
                / (collision_count + 1))
                .min(request.step_count.saturating_sub(1));
            let timestamp_ms = step_timestamp_ms(step_index, request.step_count, cycle_time_ms);
            SimulationContact {
                step_index,
                timestamp_ms,
                pair_id: pair.id.clone(),
                left_entity_id: pair.left_entity_id.clone(),
                right_entity_id: pair.right_entity_id.clone(),
                overlap_mm: round_two_decimals(
                    pair.base_clearance_mm + (request.seed % 7) as f64 * 0.11 + f64::from(index) * 0.07,
                ),
                severity: "collision".to_string(),
            }
        })
        .collect()
}

fn step_timestamp_ms(step_index: u32, step_count: u32, cycle_time_ms: u32) -> u32 {
    if step_count <= 1 {
        return cycle_time_ms;
    }
    ((u64::from(cycle_time_ms) * u64::from(step_index + 1)) / u64::from(step_count)) as u32
}

fn round_two_decimals(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use faero_types::{
        ControlTransition, ControllerStateMachine, ScheduledSignalChange, SignalAssignment,
        SignalComparator, SignalCondition, SignalDefinition, SignalKind, SignalValue,
    };

    use super::*;

    fn nominal_request() -> SimulationRequest {
        SimulationRequest {
            scenario_name: "pick-and-place".to_string(),
            seed: 42,
            engine_version: DEFAULT_ENGINE_VERSION.to_string(),
            step_count: 10,
            planned_cycle_time_ms: 3_640,
            path_length_mm: 890.0,
            endpoint_count: 1,
            safety_zone_count: 1,
            signals: Vec::new(),
            controller: None,
            scheduled_signal_changes: Vec::new(),
            contact_pairs: Vec::new(),
        }
    }

    fn simple_controller() -> ControllerStateMachine {
        ControllerStateMachine {
            id: "ctrl_pick".to_string(),
            name: "Pick Controller".to_string(),
            initial_state_id: "idle".to_string(),
            states: vec![
                ControllerState {
                    id: "idle".to_string(),
                    name: "Idle".to_string(),
                    terminal: false,
                },
                ControllerState {
                    id: "done".to_string(),
                    name: "Done".to_string(),
                    terminal: true,
                },
            ],
            transitions: vec![ControlTransition {
                id: "tr_cycle_start".to_string(),
                from_state_id: "idle".to_string(),
                to_state_id: "done".to_string(),
                conditions: vec![SignalCondition {
                    signal_id: "sig_cycle_start".to_string(),
                    comparator: SignalComparator::Equal,
                    expected_value: SignalValue::Bool(true),
                }],
                assignments: vec![SignalAssignment {
                    signal_id: "sig_part_present".to_string(),
                    value: SignalValue::Bool(false),
                }],
                description: Some("cycle completed".to_string()),
            }],
        }
    }

    #[test]
    fn run_simulation_is_deterministic_for_a_seed() {
        let request = nominal_request();

        let first = run_simulation(&request).expect("simulation should run");
        let second = run_simulation(&request).expect("simulation should run twice");

        assert_eq!(first, second);
        assert_eq!(first.status, SimulationStatus::Completed);
        assert_eq!(first.seed, 42);
        assert_eq!(first.engine_version, DEFAULT_ENGINE_VERSION);
        assert_eq!(first.collision_count, 0);
        assert_eq!(first.cycle_time_ms, 3_655);
        assert_eq!(first.max_tracking_error_mm, 0.54);
        assert_eq!(first.timeline_samples.len(), 10);
        assert_eq!(first.progress_samples.last().map(|sample| sample.progress), Some(1.0));
    }

    #[test]
    fn different_seeds_produce_different_results() {
        let first = run_simulation(&SimulationRequest {
            seed: 1,
            ..nominal_request()
        })
        .expect("simulation should run");
        let second = run_simulation(&SimulationRequest {
            seed: 2,
            ..nominal_request()
        })
        .expect("simulation should run");

        assert_ne!(first, second);
    }

    #[test]
    fn controller_can_advance_to_a_terminal_state() {
        let summary = run_simulation(&SimulationRequest {
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
                    id: "sig_part_present".to_string(),
                    name: "Part Present".to_string(),
                    kind: SignalKind::Boolean,
                    initial_value: SignalValue::Bool(true),
                    unit: None,
                    tags: vec!["process".to_string()],
                },
            ],
            controller: Some(simple_controller()),
            scheduled_signal_changes: vec![ScheduledSignalChange {
                step_index: 2,
                signal_id: "sig_cycle_start".to_string(),
                value: SignalValue::Bool(true),
                reason: "operator_start".to_string(),
            }],
            ..nominal_request()
        })
        .expect("simulation should run");

        assert!(!summary.blocked_sequence_detected);
        assert_eq!(
            summary
                .controller_state_samples
                .last()
                .map(|sample| sample.state_id.as_str()),
            Some("done")
        );
        assert!(summary.signal_samples.iter().any(|sample| {
            sample.signal_id == "sig_part_present" && sample.value == SignalValue::Bool(false)
        }));
    }

    #[test]
    fn detects_a_blocked_sequence_when_transition_never_fires() {
        let summary = run_simulation(&SimulationRequest {
            signals: vec![SignalDefinition {
                id: "sig_cycle_start".to_string(),
                name: "Cycle Start".to_string(),
                kind: SignalKind::Boolean,
                initial_value: SignalValue::Bool(false),
                unit: None,
                tags: vec!["control".to_string()],
            }],
            controller: Some(simple_controller()),
            ..nominal_request()
        })
        .expect("simulation should run");

        assert_eq!(summary.status, SimulationStatus::Warning);
        assert!(summary.blocked_sequence_detected);
        assert_eq!(summary.blocked_state_id.as_deref(), Some("idle"));
    }

    #[test]
    fn missing_safety_can_produce_a_collision_state() {
        let summary = run_simulation(&SimulationRequest {
            safety_zone_count: 0,
            seed: 6,
            contact_pairs: vec![SimulationContactPair {
                id: "pair_conveyor".to_string(),
                left_entity_id: "ent_tool_001".to_string(),
                right_entity_id: "ent_conveyor_001".to_string(),
                base_clearance_mm: 0.4,
            }],
            ..nominal_request()
        })
        .expect("simulation should run");

        assert_eq!(summary.status, SimulationStatus::Collided);
        assert!(summary.collision_count > 0);
        assert_eq!(summary.contacts.len(), summary.collision_count as usize);
        assert_eq!(summary.contacts[0].pair_id, "pair_conveyor");
    }

    #[test]
    fn rejects_zero_step_runs() {
        assert_eq!(
            run_simulation(&SimulationRequest {
                scenario_name: "invalid".to_string(),
                seed: 0,
                engine_version: DEFAULT_ENGINE_VERSION.to_string(),
                step_count: 0,
                planned_cycle_time_ms: 100,
                path_length_mm: 10.0,
                endpoint_count: 0,
                safety_zone_count: 0,
                signals: Vec::new(),
                controller: None,
                scheduled_signal_changes: Vec::new(),
                contact_pairs: Vec::new(),
            }),
            Err(SimulationError::InvalidStepCount)
        );
    }

    #[test]
    fn rejects_invalid_cycle_time_and_path_length() {
        assert_eq!(
            run_simulation(&SimulationRequest {
                planned_cycle_time_ms: 0,
                ..nominal_request()
            }),
            Err(SimulationError::InvalidCycleTime)
        );
        assert_eq!(
            run_simulation(&SimulationRequest {
                path_length_mm: -1.0,
                ..nominal_request()
            }),
            Err(SimulationError::InvalidPathLength)
        );
    }
}
