use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRequest {
    pub scenario_name: String,
    pub seed: u64,
    pub step_count: u32,
    pub planned_cycle_time_ms: u32,
    pub path_length_mm: f64,
    pub endpoint_count: u32,
    pub safety_zone_count: u32,
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
    pub collision_count: u32,
    pub cycle_time_ms: u32,
    pub max_tracking_error_mm: f64,
    pub energy_estimate_j: f64,
    pub timeline_samples: Vec<TimelineSample>,
}

#[derive(Debug, Error, PartialEq)]
pub enum SimulationError {
    #[error("simulation step count must be greater than zero")]
    InvalidStepCount,
    #[error("planned cycle time must be greater than zero")]
    InvalidCycleTime,
    #[error("path length must be finite and non-negative")]
    InvalidPathLength,
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
    let collision_count = if request.safety_zone_count == 0 {
        ((request.seed + u64::from(request.step_count)) % 3) as u32
    } else if request.endpoint_count > 2 {
        ((request.seed + u64::from(request.endpoint_count)) % 2) as u32
    } else {
        0
    };
    let status = if collision_count > 0 {
        SimulationStatus::Collided
    } else if max_tracking_error_mm > 2.5 {
        SimulationStatus::Warning
    } else {
        SimulationStatus::Completed
    };
    let energy_estimate_j = round_two_decimals(
        request.path_length_mm * 0.035
            + f64::from(cycle_time_ms) * 0.012
            + f64::from(request.endpoint_count) * 1.5,
    );
    let timeline_samples = (0..request.step_count.min(12))
        .map(|step_index| {
            let progress = if request.step_count == 1 {
                0.0
            } else {
                f64::from(step_index) / f64::from(request.step_count - 1)
            };
            TimelineSample {
                step_index,
                timestamp_ms: ((u64::from(cycle_time_ms) * u64::from(step_index + 1))
                    / u64::from(request.step_count.min(12))) as u32,
                tracking_error_mm: round_two_decimals(
                    max_tracking_error_mm * (0.45 + progress * 0.55),
                ),
                speed_scale: round_two_decimals((0.82 + progress * 0.18).min(1.0)),
            }
        })
        .collect::<Vec<_>>();

    Ok(SimulationSummary {
        status,
        collision_count,
        cycle_time_ms,
        max_tracking_error_mm,
        energy_estimate_j,
        timeline_samples,
    })
}

fn round_two_decimals(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nominal_request() -> SimulationRequest {
        SimulationRequest {
            scenario_name: "pick-and-place".to_string(),
            seed: 42,
            step_count: 10,
            planned_cycle_time_ms: 3_640,
            path_length_mm: 890.0,
            endpoint_count: 1,
            safety_zone_count: 1,
        }
    }

    #[test]
    fn run_simulation_is_deterministic_for_a_seed() {
        let request = nominal_request();

        let first = run_simulation(&request).expect("simulation should run");
        let second = run_simulation(&request).expect("simulation should run twice");

        assert_eq!(first, second);
        assert_eq!(first.status, SimulationStatus::Completed);
        assert_eq!(first.collision_count, 0);
        assert_eq!(first.cycle_time_ms, 3_655);
        assert_eq!(first.max_tracking_error_mm, 0.54);
        assert_eq!(first.timeline_samples.len(), 10);
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
    fn missing_safety_can_produce_a_collision_state() {
        let summary = run_simulation(&SimulationRequest {
            safety_zone_count: 0,
            seed: 6,
            ..nominal_request()
        })
        .expect("simulation should run");

        assert_eq!(summary.status, SimulationStatus::Collided);
        assert!(summary.collision_count > 0);
    }

    #[test]
    fn rejects_zero_step_runs() {
        assert_eq!(
            run_simulation(&SimulationRequest {
                scenario_name: "invalid".to_string(),
                seed: 0,
                step_count: 0,
                planned_cycle_time_ms: 100,
                path_length_mm: 10.0,
                endpoint_count: 0,
                safety_zone_count: 0,
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
