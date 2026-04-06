use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct PointCloudFrame {
    pub point_count: u32,
    pub coverage_ratio: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerceptionReplaySummary {
    pub frame_count: usize,
    pub average_coverage_ratio: f32,
    pub total_point_count: u32,
}

#[derive(Debug, Error, PartialEq)]
pub enum PerceptionError {
    #[error("a perception replay requires at least one frame")]
    EmptyDataset,
    #[error("coverage ratios must stay between 0.0 and 1.0")]
    InvalidCoverage,
}

pub fn replay_dataset(
    frames: &[PointCloudFrame],
) -> Result<PerceptionReplaySummary, PerceptionError> {
    if frames.is_empty() {
        return Err(PerceptionError::EmptyDataset);
    }
    if frames
        .iter()
        .any(|frame| !(0.0..=1.0).contains(&frame.coverage_ratio))
    {
        return Err(PerceptionError::InvalidCoverage);
    }

    let total_point_count = frames.iter().map(|frame| frame.point_count).sum();
    let average_coverage_ratio =
        frames.iter().map(|frame| frame.coverage_ratio).sum::<f32>() / frames.len() as f32;

    Ok(PerceptionReplaySummary {
        frame_count: frames.len(),
        average_coverage_ratio,
        total_point_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replays_a_dataset_and_aggregates_metrics() {
        let summary = replay_dataset(&[
            PointCloudFrame {
                point_count: 1200,
                coverage_ratio: 0.80,
            },
            PointCloudFrame {
                point_count: 1500,
                coverage_ratio: 0.90,
            },
        ])
        .expect("dataset should replay");

        assert_eq!(summary.frame_count, 2);
        assert_eq!(summary.total_point_count, 2700);
        assert!((summary.average_coverage_ratio - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_empty_datasets() {
        assert_eq!(replay_dataset(&[]), Err(PerceptionError::EmptyDataset));
    }

    #[test]
    fn rejects_invalid_coverage_values() {
        assert_eq!(
            replay_dataset(&[PointCloudFrame {
                point_count: 10,
                coverage_ratio: 1.2,
            }]),
            Err(PerceptionError::InvalidCoverage)
        );
    }
}
