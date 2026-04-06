use std::collections::HashSet;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotTarget {
    pub id: String,
    pub nominal_speed_mm_s: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceValidation {
    pub target_count: usize,
    pub estimated_cycle_time_ms: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoboticsError {
    #[error("a robot sequence requires at least one target")]
    EmptySequence,
    #[error("target ids must be unique and speeds must be non-zero")]
    InvalidTargetSet,
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

    let estimated_cycle_time_ms = targets
        .iter()
        .map(|target| 1_000 / target.nominal_speed_mm_s.max(1))
        .sum();

    Ok(SequenceValidation {
        target_count: targets.len(),
        estimated_cycle_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_a_nominal_robot_sequence() {
        let validation = validate_sequence(&[
            RobotTarget {
                id: "pick".to_string(),
                nominal_speed_mm_s: 250,
            },
            RobotTarget {
                id: "place".to_string(),
                nominal_speed_mm_s: 200,
            },
        ])
        .expect("sequence should validate");

        assert_eq!(validation.target_count, 2);
        assert_eq!(validation.estimated_cycle_time_ms, 9);
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
                    nominal_speed_mm_s: 100,
                },
                RobotTarget {
                    id: "dup".to_string(),
                    nominal_speed_mm_s: 100,
                },
            ]),
            Err(RoboticsError::InvalidTargetSet)
        );
        assert_eq!(
            validate_sequence(&[RobotTarget {
                id: "bad".to_string(),
                nominal_speed_mm_s: 0,
            }]),
            Err(RoboticsError::InvalidTargetSet)
        );
    }
}
