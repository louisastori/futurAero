use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Error, PartialEq)]
pub enum RoboticsError {
    #[error("a robot sequence requires at least one target")]
    EmptySequence,
    #[error("target ids must be unique and speeds must be non-zero")]
    InvalidTargetSet,
    #[error("target poses must contain finite coordinates")]
    InvalidPose,
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

fn distance_between(left: CartesianPose, right: CartesianPose) -> f64 {
    let dx = right.x_mm - left.x_mm;
    let dy = right.y_mm - left.y_mm;
    let dz = right.z_mm - left.z_mm;

    (dx * dx + dy * dy + dz * dz).sqrt()
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
}
