use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensorKind {
    Lidar2d,
    Lidar3d,
    Camera,
    SafetyLidar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SensorMount {
    pub sensor_id: String,
    pub sensor_kind: SensorKind,
    pub frame_id: String,
    pub offset_mm: [f32; 3],
    pub yaw_pitch_roll_deg: [f32; 3],
    pub sample_rate_hz: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SensorRigDefinition {
    pub id: String,
    pub name: String,
    pub base_frame_id: String,
    pub mounts: Vec<SensorMount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PointCloudFrame {
    pub point_count: u32,
    pub coverage_ratio: f32,
    pub timestamp_ms: u32,
    pub observed_obstacle_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationProfile {
    pub rig_id: String,
    pub status: String,
    pub sync_skew_ms: f32,
    pub residual_mm: f32,
    pub quality_score: f32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PerceptionProgressSample {
    pub phase: String,
    pub progress: f32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OccupancyCell {
    pub cell_id: String,
    pub x_mm: f32,
    pub y_mm: f32,
    pub z_mm: f32,
    pub occupancy_ratio: f32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NominalSceneTarget {
    pub id: String,
    pub label: String,
    pub expected_clearance_mm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SceneDeviation {
    pub id: String,
    pub target_id: String,
    pub deviation_mm: f32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ObservedSceneComparison {
    pub deviation_count: usize,
    pub unknown_obstacle_count: u32,
    pub max_deviation_mm: f32,
    pub deviations: Vec<SceneDeviation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PerceptionRunArtifacts {
    pub status: String,
    pub frame_count: usize,
    pub average_coverage_ratio: f32,
    pub total_point_count: u32,
    pub occupancy_cells: Vec<OccupancyCell>,
    pub unknown_obstacle_count: u32,
    pub progress_samples: Vec<PerceptionProgressSample>,
    pub comparison: ObservedSceneComparison,
}

#[derive(Debug, Error, PartialEq)]
pub enum PerceptionError {
    #[error("a sensor rig requires at least one mounted sensor")]
    EmptyRig,
    #[error("a perception replay requires at least one frame")]
    EmptyDataset,
    #[error("coverage ratios must stay between 0.0 and 1.0")]
    InvalidCoverage,
    #[error("sync skew must stay between 0.0 and 50.0 ms")]
    InvalidSyncSkew,
    #[error("residual mm must stay non-negative")]
    InvalidResidual,
}

pub fn seeded_sensor_rig(id: impl Into<String>, name: impl Into<String>) -> SensorRigDefinition {
    SensorRigDefinition {
        id: id.into(),
        name: name.into(),
        base_frame_id: "cell.world".to_string(),
        mounts: vec![
            SensorMount {
                sensor_id: "sensor_lidar_front".to_string(),
                sensor_kind: SensorKind::Lidar3d,
                frame_id: "cell.lidar.front".to_string(),
                offset_mm: [420.0, 120.0, 1650.0],
                yaw_pitch_roll_deg: [0.0, -5.0, 0.0],
                sample_rate_hz: 15.0,
            },
            SensorMount {
                sensor_id: "sensor_cam_overhead".to_string(),
                sensor_kind: SensorKind::Camera,
                frame_id: "cell.camera.overhead".to_string(),
                offset_mm: [0.0, 0.0, 2400.0],
                yaw_pitch_roll_deg: [0.0, -90.0, 0.0],
                sample_rate_hz: 30.0,
            },
            SensorMount {
                sensor_id: "sensor_safety_gate".to_string(),
                sensor_kind: SensorKind::SafetyLidar,
                frame_id: "cell.lidar.safety".to_string(),
                offset_mm: [850.0, -300.0, 700.0],
                yaw_pitch_roll_deg: [0.0, 0.0, 180.0],
                sample_rate_hz: 25.0,
            },
        ],
    }
}

pub fn calibrate_rig(
    rig: &SensorRigDefinition,
    sync_skew_ms: f32,
    residual_mm: f32,
) -> Result<CalibrationProfile, PerceptionError> {
    if rig.mounts.is_empty() {
        return Err(PerceptionError::EmptyRig);
    }
    if !(0.0..=50.0).contains(&sync_skew_ms) {
        return Err(PerceptionError::InvalidSyncSkew);
    }
    if residual_mm < 0.0 {
        return Err(PerceptionError::InvalidResidual);
    }

    let mut warnings = Vec::new();
    if sync_skew_ms > 8.0 {
        warnings.push("synchronization drift approaching warning threshold".to_string());
    }
    if residual_mm > 1.5 {
        warnings.push("extrinsic residual above nominal target".to_string());
    }

    let quality_score = (1.0 - sync_skew_ms / 50.0 - residual_mm / 10.0).clamp(0.0, 1.0);

    Ok(CalibrationProfile {
        rig_id: rig.id.clone(),
        status: if warnings.is_empty() {
            "stable".to_string()
        } else {
            "warning".to_string()
        },
        sync_skew_ms,
        residual_mm,
        quality_score,
        warnings,
    })
}

pub fn run_perception(
    rig: &SensorRigDefinition,
    calibration: &CalibrationProfile,
    frames: &[PointCloudFrame],
    nominal_targets: &[NominalSceneTarget],
) -> Result<PerceptionRunArtifacts, PerceptionError> {
    if rig.mounts.is_empty() {
        return Err(PerceptionError::EmptyRig);
    }
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
    let unknown_obstacle_count = frames
        .iter()
        .map(|frame| frame.observed_obstacle_count)
        .max()
        .unwrap_or(0)
        .saturating_sub(nominal_targets.len() as u32);
    let occupancy_cells = frames
        .iter()
        .take(6)
        .enumerate()
        .map(|(index, frame)| OccupancyCell {
            cell_id: format!("occ_{index:03}"),
            x_mm: index as f32 * 250.0,
            y_mm: (index as f32 * 80.0) - 120.0,
            z_mm: 0.0,
            occupancy_ratio: frame.coverage_ratio.clamp(0.0, 1.0),
            source: "lidar_fused".to_string(),
        })
        .collect::<Vec<_>>();
    let comparison = compare_observed_scene(frames, nominal_targets);

    Ok(PerceptionRunArtifacts {
        status: if calibration.status == "stable" && comparison.deviation_count == 0 {
            "clear".to_string()
        } else if comparison.unknown_obstacle_count > 0 {
            "warning".to_string()
        } else {
            "review".to_string()
        },
        frame_count: frames.len(),
        average_coverage_ratio,
        total_point_count,
        occupancy_cells,
        unknown_obstacle_count,
        progress_samples: vec![
            PerceptionProgressSample {
                phase: "capture".to_string(),
                progress: 0.25,
                message: format!("{} mounted sensors synchronized", rig.mounts.len()),
            },
            PerceptionProgressSample {
                phase: "fusion".to_string(),
                progress: 0.65,
                message: format!(
                    "average coverage {:.2} with calibration {}",
                    average_coverage_ratio, calibration.status
                ),
            },
            PerceptionProgressSample {
                phase: "compare".to_string(),
                progress: 1.0,
                message: format!(
                    "{} deviation(s), {} unknown obstacle(s)",
                    comparison.deviation_count, comparison.unknown_obstacle_count
                ),
            },
        ],
        comparison,
    })
}

pub fn compare_observed_scene(
    frames: &[PointCloudFrame],
    nominal_targets: &[NominalSceneTarget],
) -> ObservedSceneComparison {
    let average_coverage = if frames.is_empty() {
        0.0
    } else {
        frames.iter().map(|frame| frame.coverage_ratio).sum::<f32>() / frames.len() as f32
    };
    let average_obstacles = if frames.is_empty() {
        0.0
    } else {
        frames
            .iter()
            .map(|frame| frame.observed_obstacle_count as f32)
            .sum::<f32>()
            / frames.len() as f32
    };

    let deviations = nominal_targets
        .iter()
        .enumerate()
        .map(|(index, target)| {
            let deviation_mm =
                ((1.0 - average_coverage) * 12.0 + average_obstacles + index as f32 * 0.75)
                    .max(0.0);
            SceneDeviation {
                id: format!("dev_{index:03}"),
                target_id: target.id.clone(),
                deviation_mm,
                status: if deviation_mm <= target.expected_clearance_mm * 0.2 {
                    "within_tolerance".to_string()
                } else {
                    "out_of_tolerance".to_string()
                },
            }
        })
        .collect::<Vec<_>>();

    ObservedSceneComparison {
        deviation_count: deviations
            .iter()
            .filter(|deviation| deviation.status == "out_of_tolerance")
            .count(),
        unknown_obstacle_count: average_obstacles
            .max(0.0)
            .round()
            .max(nominal_targets.len() as f32)
            .round() as u32
            - nominal_targets.len() as u32,
        max_deviation_mm: deviations
            .iter()
            .map(|deviation| deviation.deviation_mm)
            .fold(0.0, f32::max),
        deviations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_frames() -> Vec<PointCloudFrame> {
        vec![
            PointCloudFrame {
                point_count: 1200,
                coverage_ratio: 0.80,
                timestamp_ms: 0,
                observed_obstacle_count: 2,
            },
            PointCloudFrame {
                point_count: 1500,
                coverage_ratio: 0.90,
                timestamp_ms: 80,
                observed_obstacle_count: 3,
            },
        ]
    }

    fn sample_targets() -> Vec<NominalSceneTarget> {
        vec![
            NominalSceneTarget {
                id: "fixture_a".to_string(),
                label: "Fixture A".to_string(),
                expected_clearance_mm: 12.0,
            },
            NominalSceneTarget {
                id: "fixture_b".to_string(),
                label: "Fixture B".to_string(),
                expected_clearance_mm: 12.0,
            },
        ]
    }

    #[test]
    fn seeded_rig_contains_multiple_sensor_kinds() {
        let rig = seeded_sensor_rig("rig_001", "Demo Rig");
        assert_eq!(rig.mounts.len(), 3);
        assert!(
            rig.mounts
                .iter()
                .any(|mount| mount.sensor_kind == SensorKind::Lidar3d)
        );
        assert!(
            rig.mounts
                .iter()
                .any(|mount| mount.sensor_kind == SensorKind::SafetyLidar)
        );
    }

    #[test]
    fn calibrates_rig_and_surfaces_warning_thresholds() {
        let rig = seeded_sensor_rig("rig_001", "Demo Rig");
        let profile = calibrate_rig(&rig, 9.0, 1.8).expect("calibration should succeed");

        assert_eq!(profile.status, "warning");
        assert_eq!(profile.rig_id, "rig_001");
        assert_eq!(profile.warnings.len(), 2);
        assert!(profile.quality_score < 1.0);
    }

    #[test]
    fn runs_perception_and_produces_occupancy_and_comparison_artifacts() {
        let rig = seeded_sensor_rig("rig_001", "Demo Rig");
        let calibration = calibrate_rig(&rig, 2.5, 0.4).expect("calibration should succeed");
        let run = run_perception(&rig, &calibration, &sample_frames(), &sample_targets())
            .expect("perception run should succeed");

        assert_eq!(run.frame_count, 2);
        assert_eq!(run.total_point_count, 2700);
        assert_eq!(run.progress_samples.len(), 3);
        assert!(!run.occupancy_cells.is_empty());
        assert!(run.comparison.max_deviation_mm >= 0.0);
    }

    #[test]
    fn rejects_invalid_rigs_frames_and_calibration_values() {
        let empty_rig = SensorRigDefinition {
            id: "rig_empty".to_string(),
            name: "Empty".to_string(),
            base_frame_id: "world".to_string(),
            mounts: Vec::new(),
        };
        assert_eq!(
            calibrate_rig(&empty_rig, 1.0, 0.1),
            Err(PerceptionError::EmptyRig)
        );
        let rig = seeded_sensor_rig("rig_001", "Demo Rig");
        assert_eq!(
            calibrate_rig(&rig, 90.0, 0.1),
            Err(PerceptionError::InvalidSyncSkew)
        );
        assert_eq!(
            run_perception(
                &rig,
                &calibrate_rig(&rig, 1.0, 0.1).expect("profile"),
                &[],
                &[]
            ),
            Err(PerceptionError::EmptyDataset)
        );
        assert_eq!(
            run_perception(
                &rig,
                &calibrate_rig(&rig, 1.0, 0.1).expect("profile"),
                &[PointCloudFrame {
                    point_count: 10,
                    coverage_ratio: 1.2,
                    timestamp_ms: 0,
                    observed_obstacle_count: 0,
                }],
                &[]
            ),
            Err(PerceptionError::InvalidCoverage)
        );
    }

    #[test]
    fn compares_observed_scene_and_flags_out_of_tolerance_targets() {
        let comparison = compare_observed_scene(&sample_frames(), &sample_targets());

        assert_eq!(comparison.deviations.len(), 2);
        assert!(
            comparison
                .deviations
                .iter()
                .all(|deviation| !deviation.target_id.is_empty())
        );
    }
}
