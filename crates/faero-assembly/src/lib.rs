use std::collections::HashSet;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MateConstraint {
    pub left_occurrence_id: String,
    pub right_occurrence_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssemblySolveStatus {
    Solved,
    Conflicting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblySolveReport {
    pub status: AssemblySolveStatus,
    pub constrained_occurrence_count: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssemblyError {
    #[error("an assembly requires at least two occurrences")]
    NotEnoughOccurrences,
    #[error("mate constraints must target distinct known occurrences without duplicates")]
    InvalidConstraintGraph,
}

pub fn solve_assembly(
    occurrences: &[Occurrence],
    constraints: &[MateConstraint],
) -> Result<AssemblySolveReport, AssemblyError> {
    if occurrences.len() < 2 {
        return Err(AssemblyError::NotEnoughOccurrences);
    }

    let known_ids = occurrences
        .iter()
        .map(|occurrence| occurrence.id.as_str())
        .collect::<HashSet<_>>();
    let mut seen_pairs = HashSet::new();

    for constraint in constraints {
        if constraint.left_occurrence_id == constraint.right_occurrence_id {
            return Err(AssemblyError::InvalidConstraintGraph);
        }
        if !known_ids.contains(constraint.left_occurrence_id.as_str())
            || !known_ids.contains(constraint.right_occurrence_id.as_str())
        {
            return Err(AssemblyError::InvalidConstraintGraph);
        }

        let pair = if constraint.left_occurrence_id < constraint.right_occurrence_id {
            (
                constraint.left_occurrence_id.as_str(),
                constraint.right_occurrence_id.as_str(),
            )
        } else {
            (
                constraint.right_occurrence_id.as_str(),
                constraint.left_occurrence_id.as_str(),
            )
        };

        if !seen_pairs.insert(pair) {
            return Err(AssemblyError::InvalidConstraintGraph);
        }
    }

    let status = if constraints.len() + 1 >= occurrences.len() {
        AssemblySolveStatus::Solved
    } else {
        AssemblySolveStatus::Conflicting
    };

    Ok(AssemblySolveReport {
        status,
        constrained_occurrence_count: constraints.len() * 2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn occurrences() -> Vec<Occurrence> {
        vec![
            Occurrence {
                id: "occ_a".to_string(),
            },
            Occurrence {
                id: "occ_b".to_string(),
            },
            Occurrence {
                id: "occ_c".to_string(),
            },
        ]
    }

    #[test]
    fn solves_when_occurrences_are_connected() {
        let report = solve_assembly(
            &occurrences(),
            &[
                MateConstraint {
                    left_occurrence_id: "occ_a".to_string(),
                    right_occurrence_id: "occ_b".to_string(),
                },
                MateConstraint {
                    left_occurrence_id: "occ_b".to_string(),
                    right_occurrence_id: "occ_c".to_string(),
                },
            ],
        )
        .expect("connected occurrences should solve");

        assert_eq!(report.status, AssemblySolveStatus::Solved);
        assert_eq!(report.constrained_occurrence_count, 4);
    }

    #[test]
    fn marks_underconnected_assemblies_as_conflicting() {
        let report = solve_assembly(
            &occurrences(),
            &[MateConstraint {
                left_occurrence_id: "occ_a".to_string(),
                right_occurrence_id: "occ_b".to_string(),
            }],
        )
        .expect("partial graph should still report");

        assert_eq!(report.status, AssemblySolveStatus::Conflicting);
    }

    #[test]
    fn rejects_invalid_constraints() {
        assert_eq!(
            solve_assembly(
                &[Occurrence {
                    id: "occ_a".to_string(),
                }],
                &[],
            ),
            Err(AssemblyError::NotEnoughOccurrences)
        );
        assert_eq!(
            solve_assembly(
                &occurrences(),
                &[MateConstraint {
                    left_occurrence_id: "occ_a".to_string(),
                    right_occurrence_id: "occ_a".to_string(),
                }],
            ),
            Err(AssemblyError::InvalidConstraintGraph)
        );
    }
}
