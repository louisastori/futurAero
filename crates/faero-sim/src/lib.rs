use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationRequest {
    pub scenario_name: String,
    pub seed: u64,
    pub step_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationSummary {
    pub collision_count: u32,
    pub cycle_time_ms: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SimulationError {
    #[error("simulation step count must be greater than zero")]
    InvalidStepCount,
}

pub fn run_simulation(request: &SimulationRequest) -> Result<SimulationSummary, SimulationError> {
    if request.step_count == 0 {
        return Err(SimulationError::InvalidStepCount);
    }

    let collision_count = ((request.seed % 7) as u32 + request.step_count) % 5;
    let cycle_time_ms = request.step_count * 40 + (request.seed % 13) as u32;

    Ok(SimulationSummary {
        collision_count,
        cycle_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_simulation_is_deterministic_for_a_seed() {
        let request = SimulationRequest {
            scenario_name: "pick-and-place".to_string(),
            seed: 42,
            step_count: 10,
        };

        let first = run_simulation(&request).expect("simulation should run");
        let second = run_simulation(&request).expect("simulation should run twice");

        assert_eq!(first, second);
        assert_eq!(first.collision_count, 0);
        assert_eq!(first.cycle_time_ms, 400 + (42 % 13) as u32);
    }

    #[test]
    fn different_seeds_produce_different_results() {
        let first = run_simulation(&SimulationRequest {
            scenario_name: "pick-and-place".to_string(),
            seed: 1,
            step_count: 8,
        })
        .expect("simulation should run");
        let second = run_simulation(&SimulationRequest {
            scenario_name: "pick-and-place".to_string(),
            seed: 2,
            step_count: 8,
        })
        .expect("simulation should run");

        assert_ne!(first, second);
    }

    #[test]
    fn rejects_zero_step_runs() {
        assert_eq!(
            run_simulation(&SimulationRequest {
                scenario_name: "invalid".to_string(),
                seed: 0,
                step_count: 0,
            }),
            Err(SimulationError::InvalidStepCount)
        );
    }
}
