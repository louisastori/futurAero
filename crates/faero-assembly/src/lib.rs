use std::collections::{HashMap, HashSet, VecDeque};

pub use faero_types::{
    AssemblyJoint as Joint, AssemblyJointType as JointType,
    AssemblyMateConstraint as MateConstraint, AssemblyMateType as MateType,
    AssemblyOccurrence as Occurrence, AssemblySolveReport, AssemblySolveStatus,
    AssemblySolvedOccurrence as SolvedOccurrence, AssemblyTransform as Transform3D,
};
use thiserror::Error;

const TRANSFORM_EPSILON: f64 = 1e-6;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssemblyError {
    #[error("an assembly requires at least two occurrences")]
    NotEnoughOccurrences,
    #[error("occurrence ids must be unique and reference a part entity")]
    InvalidOccurrenceSet,
    #[error("mate constraints must target distinct known occurrences without duplicates")]
    InvalidConstraintGraph,
    #[error("offset mates must use a non-negative distance")]
    NegativeOffset,
}

pub fn joint_degrees_of_freedom(joint_type: JointType) -> usize {
    match joint_type {
        JointType::Fixed => 0,
        JointType::Revolute | JointType::Prismatic => 1,
    }
}

pub fn solve_assembly(
    occurrences: &[Occurrence],
    constraints: &[MateConstraint],
) -> Result<AssemblySolveReport, AssemblyError> {
    if occurrences.len() < 2 {
        return Err(AssemblyError::NotEnoughOccurrences);
    }

    let mut known_ids = HashSet::new();
    let occurrence_by_id = occurrences
        .iter()
        .map(|occurrence| {
            if occurrence.id.is_empty()
                || occurrence.definition_entity_id.is_empty()
                || !known_ids.insert(occurrence.id.as_str())
            {
                return Err(AssemblyError::InvalidOccurrenceSet);
            }

            Ok((occurrence.id.as_str(), occurrence))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let mut seen_pairs = HashSet::new();
    let mut adjacency: HashMap<&str, Vec<(&MateConstraint, &str)>> = HashMap::new();
    let mut constrained_ids = HashSet::new();

    for constraint in constraints {
        if constraint.left_occurrence_id == constraint.right_occurrence_id {
            return Err(AssemblyError::InvalidConstraintGraph);
        }
        if !occurrence_by_id.contains_key(constraint.left_occurrence_id.as_str())
            || !occurrence_by_id.contains_key(constraint.right_occurrence_id.as_str())
        {
            return Err(AssemblyError::InvalidConstraintGraph);
        }
        if let MateType::Offset { distance_mm } = constraint.mate_type
            && distance_mm < 0.0
        {
            return Err(AssemblyError::NegativeOffset);
        }

        let pair = normalized_pair(
            constraint.left_occurrence_id.as_str(),
            constraint.right_occurrence_id.as_str(),
        );
        if !seen_pairs.insert(pair) {
            return Err(AssemblyError::InvalidConstraintGraph);
        }

        adjacency
            .entry(constraint.left_occurrence_id.as_str())
            .or_default()
            .push((constraint, constraint.right_occurrence_id.as_str()));
        adjacency
            .entry(constraint.right_occurrence_id.as_str())
            .or_default()
            .push((constraint, constraint.left_occurrence_id.as_str()));
        constrained_ids.insert(constraint.left_occurrence_id.as_str());
        constrained_ids.insert(constraint.right_occurrence_id.as_str());
    }

    let root = occurrences.first().expect("validated occurrence list");
    let mut solved_transforms = HashMap::from([(root.id.as_str(), root.transform)]);
    let mut queue = VecDeque::from([root.id.as_str()]);
    let mut warnings = Vec::new();
    let mut conflict_detected = false;

    while let Some(current_id) = queue.pop_front() {
        let current_transform = solved_transforms[&current_id];
        for (constraint, neighbor_id) in adjacency.get(current_id).into_iter().flatten() {
            let expected_transform =
                propagated_transform(current_id, neighbor_id, current_transform, constraint);
            match solved_transforms.get(neighbor_id) {
                Some(existing) => {
                    if !transforms_approximately_equal(*existing, expected_transform) {
                        conflict_detected = true;
                        warnings.push(format!(
                            "constraint `{}` over-constrains occurrence `{}`",
                            constraint.id, neighbor_id
                        ));
                    }
                }
                None => {
                    solved_transforms.insert(neighbor_id, expected_transform);
                    queue.push_back(neighbor_id);
                }
            }
        }
    }

    let all_connected = solved_transforms.len() == occurrences.len();
    if !all_connected {
        warnings.push("assembly graph is under-connected".to_string());
    }

    let status = if all_connected && !conflict_detected {
        AssemblySolveStatus::Solved
    } else {
        AssemblySolveStatus::Conflicting
    };

    let solved_occurrences = occurrences
        .iter()
        .map(|occurrence| SolvedOccurrence {
            occurrence_id: occurrence.id.clone(),
            transform: *solved_transforms
                .get(occurrence.id.as_str())
                .unwrap_or(&occurrence.transform),
        })
        .collect::<Vec<_>>();

    Ok(AssemblySolveReport {
        status,
        constrained_occurrence_count: constrained_ids.len(),
        total_mate_count: constraints.len(),
        degrees_of_freedom_estimate: occurrences.len().saturating_sub(constraints.len() + 1) * 3,
        solved_occurrences,
        warnings,
    })
}

fn normalized_pair<'a>(left: &'a str, right: &'a str) -> (&'a str, &'a str) {
    if left < right {
        (left, right)
    } else {
        (right, left)
    }
}

fn propagated_transform(
    current_id: &str,
    neighbor_id: &str,
    current_transform: Transform3D,
    constraint: &MateConstraint,
) -> Transform3D {
    let right_is_neighbor = constraint.right_occurrence_id == neighbor_id;
    let offset = match constraint.mate_type {
        MateType::Coincident => 0.0,
        MateType::Offset { distance_mm } => {
            if right_is_neighbor {
                distance_mm
            } else {
                -distance_mm
            }
        }
    };

    let _ = current_id;
    Transform3D {
        x_mm: current_transform.x_mm + offset,
        ..current_transform
    }
}

fn transforms_approximately_equal(left: Transform3D, right: Transform3D) -> bool {
    (left.x_mm - right.x_mm).abs() <= TRANSFORM_EPSILON
        && (left.y_mm - right.y_mm).abs() <= TRANSFORM_EPSILON
        && (left.z_mm - right.z_mm).abs() <= TRANSFORM_EPSILON
        && (left.yaw_deg - right.yaw_deg).abs() <= TRANSFORM_EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    fn occurrences() -> Vec<Occurrence> {
        vec![
            Occurrence {
                id: "occ_a".to_string(),
                definition_entity_id: "ent_part_001".to_string(),
                transform: Transform3D::default(),
            },
            Occurrence {
                id: "occ_b".to_string(),
                definition_entity_id: "ent_part_002".to_string(),
                transform: Transform3D::default(),
            },
            Occurrence {
                id: "occ_c".to_string(),
                definition_entity_id: "ent_part_003".to_string(),
                transform: Transform3D::default(),
            },
        ]
    }

    #[test]
    fn solves_when_occurrences_are_connected_and_propagates_offsets() {
        let report = solve_assembly(
            &occurrences(),
            &[
                MateConstraint {
                    id: "mate_ab".to_string(),
                    left_occurrence_id: "occ_a".to_string(),
                    right_occurrence_id: "occ_b".to_string(),
                    mate_type: MateType::Coincident,
                },
                MateConstraint {
                    id: "mate_bc".to_string(),
                    left_occurrence_id: "occ_b".to_string(),
                    right_occurrence_id: "occ_c".to_string(),
                    mate_type: MateType::Offset { distance_mm: 25.0 },
                },
            ],
        )
        .expect("connected occurrences should solve");

        assert_eq!(report.status, AssemblySolveStatus::Solved);
        assert_eq!(report.constrained_occurrence_count, 3);
        assert_eq!(report.total_mate_count, 2);
        assert_eq!(report.degrees_of_freedom_estimate, 0);
        assert_eq!(report.solved_occurrences[2].transform.x_mm, 25.0);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn marks_underconnected_assemblies_as_conflicting() {
        let report = solve_assembly(
            &occurrences(),
            &[MateConstraint {
                id: "mate_ab".to_string(),
                left_occurrence_id: "occ_a".to_string(),
                right_occurrence_id: "occ_b".to_string(),
                mate_type: MateType::Coincident,
            }],
        )
        .expect("partial graph should still report");

        assert_eq!(report.status, AssemblySolveStatus::Conflicting);
        assert!(!report.warnings.is_empty());
        assert_eq!(report.degrees_of_freedom_estimate, 3);
    }

    #[test]
    fn rejects_invalid_occurrences_and_constraints() {
        assert_eq!(
            solve_assembly(
                &[Occurrence {
                    id: "occ_a".to_string(),
                    definition_entity_id: "ent_part_001".to_string(),
                    transform: Transform3D::default(),
                }],
                &[],
            ),
            Err(AssemblyError::NotEnoughOccurrences)
        );
        assert_eq!(
            solve_assembly(
                &[
                    Occurrence {
                        id: "occ_a".to_string(),
                        definition_entity_id: "ent_part_001".to_string(),
                        transform: Transform3D::default(),
                    },
                    Occurrence {
                        id: "occ_a".to_string(),
                        definition_entity_id: "ent_part_002".to_string(),
                        transform: Transform3D::default(),
                    },
                ],
                &[],
            ),
            Err(AssemblyError::InvalidOccurrenceSet)
        );
        assert_eq!(
            solve_assembly(
                &occurrences(),
                &[MateConstraint {
                    id: "mate_bad".to_string(),
                    left_occurrence_id: "occ_a".to_string(),
                    right_occurrence_id: "occ_a".to_string(),
                    mate_type: MateType::Coincident,
                }],
            ),
            Err(AssemblyError::InvalidConstraintGraph)
        );
        assert_eq!(
            solve_assembly(
                &occurrences(),
                &[MateConstraint {
                    id: "mate_offset".to_string(),
                    left_occurrence_id: "occ_a".to_string(),
                    right_occurrence_id: "occ_b".to_string(),
                    mate_type: MateType::Offset { distance_mm: -1.0 },
                }],
            ),
            Err(AssemblyError::NegativeOffset)
        );
    }

    #[test]
    fn rejects_unknown_occurrences_and_duplicate_reverse_pairs() {
        assert_eq!(
            solve_assembly(
                &occurrences(),
                &[MateConstraint {
                    id: "mate_unknown".to_string(),
                    left_occurrence_id: "occ_a".to_string(),
                    right_occurrence_id: "missing".to_string(),
                    mate_type: MateType::Coincident,
                }],
            ),
            Err(AssemblyError::InvalidConstraintGraph)
        );

        assert_eq!(
            solve_assembly(
                &occurrences(),
                &[
                    MateConstraint {
                        id: "mate_ba".to_string(),
                        left_occurrence_id: "occ_b".to_string(),
                        right_occurrence_id: "occ_a".to_string(),
                        mate_type: MateType::Coincident,
                    },
                    MateConstraint {
                        id: "mate_ab".to_string(),
                        left_occurrence_id: "occ_a".to_string(),
                        right_occurrence_id: "occ_b".to_string(),
                        mate_type: MateType::Coincident,
                    },
                ],
            ),
            Err(AssemblyError::InvalidConstraintGraph)
        );
    }

    #[test]
    fn joint_types_expose_expected_mvp_degrees_of_freedom() {
        assert_eq!(joint_degrees_of_freedom(JointType::Fixed), 0);
        assert_eq!(joint_degrees_of_freedom(JointType::Revolute), 1);
        assert_eq!(joint_degrees_of_freedom(JointType::Prismatic), 1);
    }
}
