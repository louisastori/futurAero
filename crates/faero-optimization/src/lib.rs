use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationObjective {
    pub id: String,
    pub label: String,
    pub goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationConstraint {
    pub id: String,
    pub label: String,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationVariable {
    pub id: String,
    pub label: String,
    pub minimum: f32,
    pub maximum: f32,
    pub current: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationCandidate {
    pub id: String,
    pub cycle_time_ms: u32,
    pub energy_wh: u32,
    pub safety_margin_mm: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RankedCandidate {
    pub id: String,
    pub score: f32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationStudy {
    pub id: String,
    pub objectives: Vec<OptimizationObjective>,
    pub constraints: Vec<OptimizationConstraint>,
    pub variables: Vec<OptimizationVariable>,
    pub candidates: Vec<OptimizationCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationRunReport {
    pub candidate_count: usize,
    pub ranked_candidates: Vec<RankedCandidate>,
    pub best_candidate_id: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OptimizationError {
    #[error("an optimization study requires at least one candidate")]
    EmptyStudy,
    #[error("an optimization study requires at least one objective")]
    EmptyObjectives,
    #[error("optimization variables must keep min <= current <= max")]
    InvalidVariableBounds,
}

pub fn seeded_study(id: impl Into<String>) -> OptimizationStudy {
    OptimizationStudy {
        id: id.into(),
        objectives: vec![
            OptimizationObjective {
                id: "obj_cycle".to_string(),
                label: "Cycle Time".to_string(),
                goal: "minimize".to_string(),
            },
            OptimizationObjective {
                id: "obj_energy".to_string(),
                label: "Energy".to_string(),
                goal: "minimize".to_string(),
            },
            OptimizationObjective {
                id: "obj_safety".to_string(),
                label: "Safety Margin".to_string(),
                goal: "maximize".to_string(),
            },
        ],
        constraints: vec![
            OptimizationConstraint {
                id: "cst_safety".to_string(),
                label: "Minimum safety margin".to_string(),
                expression: "safety_margin_mm >= 18".to_string(),
            },
            OptimizationConstraint {
                id: "cst_clearance".to_string(),
                label: "Perception deviation".to_string(),
                expression: "max_perception_deviation_mm <= 6".to_string(),
            },
        ],
        variables: vec![
            OptimizationVariable {
                id: "var_speed".to_string(),
                label: "Robot speed scale".to_string(),
                minimum: 0.6,
                maximum: 1.0,
                current: 0.82,
            },
            OptimizationVariable {
                id: "var_buffer".to_string(),
                label: "Conveyor buffer".to_string(),
                minimum: 0.0,
                maximum: 120.0,
                current: 35.0,
            },
        ],
        candidates: vec![
            OptimizationCandidate {
                id: "candidate_fast".to_string(),
                cycle_time_ms: 900,
                energy_wh: 70,
                safety_margin_mm: 15,
            },
            OptimizationCandidate {
                id: "candidate_balanced".to_string(),
                cycle_time_ms: 1_050,
                energy_wh: 40,
                safety_margin_mm: 20,
            },
            OptimizationCandidate {
                id: "candidate_safe".to_string(),
                cycle_time_ms: 1_200,
                energy_wh: 35,
                safety_margin_mm: 24,
            },
        ],
    }
}

pub fn run_study(study: &OptimizationStudy) -> Result<OptimizationRunReport, OptimizationError> {
    if study.objectives.is_empty() {
        return Err(OptimizationError::EmptyObjectives);
    }
    if study.candidates.is_empty() {
        return Err(OptimizationError::EmptyStudy);
    }
    if study.variables.iter().any(|variable| {
        variable.minimum > variable.current || variable.current > variable.maximum
    }) {
        return Err(OptimizationError::InvalidVariableBounds);
    }

    let ranked_candidates = rank_candidates(&study.candidates)?;
    Ok(OptimizationRunReport {
        candidate_count: study.candidates.len(),
        best_candidate_id: ranked_candidates.first().map(|candidate| candidate.id.clone()),
        ranked_candidates,
    })
}

pub fn rank_candidates(
    candidates: &[OptimizationCandidate],
) -> Result<Vec<RankedCandidate>, OptimizationError> {
    if candidates.is_empty() {
        return Err(OptimizationError::EmptyStudy);
    }

    let mut ranked = candidates
        .iter()
        .map(|candidate| {
            let score = candidate.safety_margin_mm as f32 * 2.0
                - candidate.cycle_time_ms as f32 * 0.01
                - candidate.energy_wh as f32 * 0.05;
            RankedCandidate {
                id: candidate.id.clone(),
                score,
                recommendation: if score >= 33.0 {
                    "apply".to_string()
                } else if score >= 26.0 {
                    "review".to_string()
                } else {
                    "reject".to_string()
                },
            }
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.score.total_cmp(&left.score));
    Ok(ranked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_study_is_populated_with_objectives_constraints_and_variables() {
        let study = seeded_study("study_001");

        assert_eq!(study.objectives.len(), 3);
        assert_eq!(study.constraints.len(), 2);
        assert_eq!(study.variables.len(), 2);
        assert_eq!(study.candidates.len(), 3);
    }

    #[test]
    fn ranks_candidates_by_weighted_score() {
        let ranked = rank_candidates(&[
            OptimizationCandidate {
                id: "fast".to_string(),
                cycle_time_ms: 900,
                energy_wh: 70,
                safety_margin_mm: 15,
            },
            OptimizationCandidate {
                id: "balanced".to_string(),
                cycle_time_ms: 1_050,
                energy_wh: 40,
                safety_margin_mm: 20,
            },
        ])
        .expect("study should rank");

        assert_eq!(ranked[0].id, "balanced");
        assert!(ranked[0].score > ranked[1].score);
    }

    #[test]
    fn runs_study_and_selects_a_best_candidate() {
        let study = seeded_study("study_001");
        let report = run_study(&study).expect("study should run");

        assert_eq!(report.candidate_count, 3);
        assert!(report.best_candidate_id.is_some());
        assert_eq!(report.ranked_candidates.len(), 3);
        assert!(report
            .ranked_candidates
            .iter()
            .any(|candidate| candidate.recommendation == "apply"));
    }

    #[test]
    fn rejects_empty_or_invalid_studies() {
        assert_eq!(rank_candidates(&[]), Err(OptimizationError::EmptyStudy));
        assert_eq!(
            run_study(&OptimizationStudy {
                id: "study_001".to_string(),
                objectives: Vec::new(),
                constraints: Vec::new(),
                variables: Vec::new(),
                candidates: vec![OptimizationCandidate {
                    id: "only".to_string(),
                    cycle_time_ms: 1_000,
                    energy_wh: 50,
                    safety_margin_mm: 20,
                }],
            }),
            Err(OptimizationError::EmptyObjectives)
        );

        let mut invalid = seeded_study("study_invalid");
        invalid.variables[0].current = 2.0;
        assert_eq!(
            run_study(&invalid),
            Err(OptimizationError::InvalidVariableBounds)
        );
    }
}
