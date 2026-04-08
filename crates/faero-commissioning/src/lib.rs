use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AsBuiltMeasurement {
    pub id: String,
    pub target_id: String,
    pub deviation_mm: f32,
    pub tolerance_mm: f32,
    pub source_capture_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommissioningCapture {
    pub id: String,
    pub source: String,
    pub capture_type: String,
    pub asset_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdjustmentLogEntry {
    pub id: String,
    pub target_id: String,
    pub action: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommissioningSession {
    pub session_id: String,
    pub status: String,
    pub progress_ratio: f32,
    pub captures: Vec<CommissioningCapture>,
    pub adjustments: Vec<AdjustmentLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AsBuiltComparison {
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub average_deviation_mm: f32,
    pub max_deviation_mm: f32,
    pub reportable: bool,
    pub measurements: Vec<AsBuiltMeasurement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommissioningReport {
    pub accepted_count: usize,
    pub rejected_count: usize,
}

pub fn start_commissioning_session(
    session_id: impl Into<String>,
    captures: Vec<CommissioningCapture>,
) -> CommissioningSession {
    let progress_ratio = if captures.is_empty() {
        0.2
    } else {
        (0.35 + captures.len() as f32 * 0.25).min(1.0)
    };

    CommissioningSession {
        session_id: session_id.into(),
        status: if captures.is_empty() {
            "open".to_string()
        } else {
            "capturing".to_string()
        },
        progress_ratio,
        captures,
        adjustments: vec![
            AdjustmentLogEntry {
                id: "adj_001".to_string(),
                target_id: "fixture.world".to_string(),
                action: "verify network bindings".to_string(),
                status: "done".to_string(),
            },
            AdjustmentLogEntry {
                id: "adj_002".to_string(),
                target_id: "sensor_rig.main".to_string(),
                action: "review lidar extrinsics".to_string(),
                status: "pending".to_string(),
            },
        ],
    }
}

#[must_use]
pub fn build_commissioning_report(measurements: &[AsBuiltMeasurement]) -> CommissioningReport {
    let accepted_count = measurements
        .iter()
        .filter(|measurement| measurement.deviation_mm.abs() <= measurement.tolerance_mm)
        .count();

    CommissioningReport {
        accepted_count,
        rejected_count: measurements.len().saturating_sub(accepted_count),
    }
}

#[must_use]
pub fn compare_as_built(measurements: Vec<AsBuiltMeasurement>) -> AsBuiltComparison {
    let report = build_commissioning_report(&measurements);
    let max_deviation_mm = measurements
        .iter()
        .map(|measurement| measurement.deviation_mm.abs())
        .fold(0.0, f32::max);
    let average_deviation_mm = if measurements.is_empty() {
        0.0
    } else {
        measurements
            .iter()
            .map(|measurement| measurement.deviation_mm.abs())
            .sum::<f32>()
            / measurements.len() as f32
    };

    AsBuiltComparison {
        accepted_count: report.accepted_count,
        rejected_count: report.rejected_count,
        average_deviation_mm,
        max_deviation_mm,
        reportable: !measurements.is_empty(),
        measurements,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_measurements() -> Vec<AsBuiltMeasurement> {
        vec![
            AsBuiltMeasurement {
                id: "m1".to_string(),
                target_id: "fixture_a".to_string(),
                deviation_mm: 0.5,
                tolerance_mm: 1.0,
                source_capture_id: "cap_001".to_string(),
            },
            AsBuiltMeasurement {
                id: "m2".to_string(),
                target_id: "fixture_b".to_string(),
                deviation_mm: 1.5,
                tolerance_mm: 1.0,
                source_capture_id: "cap_002".to_string(),
            },
        ]
    }

    #[test]
    fn starts_a_commissioning_session_with_adjustment_journal() {
        let session = start_commissioning_session(
            "com_001",
            vec![CommissioningCapture {
                id: "cap_001".to_string(),
                source: "wifi".to_string(),
                capture_type: "point_cloud".to_string(),
                asset_ref: "captures/cap_001.pcd".to_string(),
            }],
        );

        assert_eq!(session.session_id, "com_001");
        assert_eq!(session.status, "capturing");
        assert!(session.progress_ratio > 0.35);
        assert_eq!(session.adjustments.len(), 2);
    }

    #[test]
    fn counts_measurements_within_tolerance() {
        let report = build_commissioning_report(&sample_measurements());

        assert_eq!(report.accepted_count, 1);
        assert_eq!(report.rejected_count, 1);
    }

    #[test]
    fn compares_as_built_against_nominal_and_surfaces_metrics() {
        let comparison = compare_as_built(sample_measurements());

        assert_eq!(comparison.accepted_count, 1);
        assert_eq!(comparison.rejected_count, 1);
        assert!(comparison.average_deviation_mm > 0.0);
        assert!(comparison.max_deviation_mm >= comparison.average_deviation_mm);
        assert!(comparison.reportable);
    }

    #[test]
    fn handles_empty_measurement_sets() {
        let report = build_commissioning_report(&[]);
        assert_eq!(report.accepted_count, 0);
        assert_eq!(report.rejected_count, 0);

        let comparison = compare_as_built(Vec::new());
        assert!(!comparison.reportable);
        assert_eq!(comparison.average_deviation_mm, 0.0);
    }

    #[test]
    fn treats_negative_deviation_as_valid_when_within_tolerance() {
        let report = build_commissioning_report(&[AsBuiltMeasurement {
            id: "m1".to_string(),
            target_id: "fixture_a".to_string(),
            deviation_mm: -0.7,
            tolerance_mm: 1.0,
            source_capture_id: "cap_001".to_string(),
        }]);

        assert_eq!(report.accepted_count, 1);
        assert_eq!(report.rejected_count, 0);
    }
}
