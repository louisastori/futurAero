use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationCandidate {
    pub id: String,
    pub cycle_time_ms: u32,
    pub energy_wh: u32,
    pub safety_margin_mm: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedCandidate {
    pub id: String,
    pub score: f32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OptimizationError {
    #[error("an optimization study requires at least one candidate")]
    EmptyStudy,
}

pub fn rank_candidates(
    candidates: &[OptimizationCandidate],
) -> Result<Vec<RankedCandidate>, OptimizationError> {
    if candidates.is_empty() {
        return Err(OptimizationError::EmptyStudy);
    }

    let mut ranked = candidates
        .iter()
        .map(|candidate| RankedCandidate {
            id: candidate.id.clone(),
            score: candidate.safety_margin_mm as f32 * 2.0
                - candidate.cycle_time_ms as f32 * 0.01
                - candidate.energy_wh as f32 * 0.05,
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.score.total_cmp(&left.score));
    Ok(ranked)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn rejects_empty_studies() {
        assert_eq!(rank_candidates(&[]), Err(OptimizationError::EmptyStudy));
    }

    #[test]
    fn preserves_all_candidates_in_ranked_output() {
        let ranked = rank_candidates(&[
            OptimizationCandidate {
                id: "a".to_string(),
                cycle_time_ms: 1_000,
                energy_wh: 50,
                safety_margin_mm: 20,
            },
            OptimizationCandidate {
                id: "b".to_string(),
                cycle_time_ms: 1_100,
                energy_wh: 30,
                safety_margin_mm: 25,
            },
        ])
        .expect("study should rank");

        assert_eq!(ranked.len(), 2);
        assert!(ranked.iter().any(|candidate| candidate.id == "a"));
        assert!(ranked.iter().any(|candidate| candidate.id == "b"));
    }
}
