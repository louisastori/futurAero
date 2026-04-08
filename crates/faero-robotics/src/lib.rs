use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotCellModel {
    pub id: String,
    pub scene_assembly_id: String,
    pub robot_ids: Vec<String>,
    pub equipment_ids: Vec<String>,
    pub safety_zone_ids: Vec<String>,
    pub sequence_ids: Vec<String>,
    pub controller_model_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotToolMountRef {
    pub equipment_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotWorkspaceBounds {
    pub reach_radius_mm: f64,
    pub vertical_span_mm: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotPayloadLimits {
    pub nominal_kg: f64,
    pub max_kg: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotModel {
    pub id: String,
    pub cell_id: String,
    pub kinematic_chain: Vec<String>,
    pub joint_ids: Vec<String>,
    pub tool_mount_ref: RobotToolMountRef,
    pub workspace_bounds: RobotWorkspaceBounds,
    pub payload_limits: RobotPayloadLimits,
    pub calibration_state: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentType {
    Conveyor,
    Workstation,
    Gripper,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentParameterSet {
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
    pub nominal_speed_mm_s: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentModel {
    pub id: String,
    pub cell_id: String,
    pub equipment_type: EquipmentType,
    pub assembly_occurrence_id: String,
    pub parameter_set: EquipmentParameterSet,
    pub io_port_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotSequenceModel {
    pub id: String,
    pub cell_id: String,
    pub robot_id: String,
    pub target_ids: Vec<String>,
    pub path_length_mm: f64,
    pub estimated_cycle_time_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotCellStructureSummary {
    pub robot_count: usize,
    pub equipment_count: usize,
    pub safety_zone_count: usize,
    pub sequence_count: usize,
    pub controller_count: usize,
}

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
    #[error(
        "robot cell structure requires non-empty unique ids for scene, robot, equipment, safety, sequence and controller refs"
    )]
    InvalidRobotCellStructure,
    #[error("robot models must belong to the cell and match the declared robot ids")]
    InvalidRobotModelSet,
    #[error(
        "equipment models must belong to the cell and expose valid scene occurrence references"
    )]
    InvalidEquipmentModelSet,
    #[error(
        "robot sequences must belong to the cell and target declared robots with non-empty targets"
    )]
    InvalidRobotSequenceSet,
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

pub fn validate_robot_cell_structure(
    cell: &RobotCellModel,
    robots: &[RobotModel],
    equipment: &[EquipmentModel],
    sequences: &[RobotSequenceModel],
) -> Result<RobotCellStructureSummary, RoboticsError> {
    if cell.id.is_empty()
        || cell.scene_assembly_id.is_empty()
        || cell.robot_ids.is_empty()
        || cell.equipment_ids.is_empty()
        || cell.safety_zone_ids.is_empty()
        || cell.sequence_ids.is_empty()
        || cell.controller_model_ids.is_empty()
        || !ids_are_unique(&cell.robot_ids)
        || !ids_are_unique(&cell.equipment_ids)
        || !ids_are_unique(&cell.safety_zone_ids)
        || !ids_are_unique(&cell.sequence_ids)
        || !ids_are_unique(&cell.controller_model_ids)
    {
        return Err(RoboticsError::InvalidRobotCellStructure);
    }

    let robot_ids = robots
        .iter()
        .map(|robot| robot.id.as_str())
        .collect::<Vec<_>>();
    if robots.len() != cell.robot_ids.len()
        || !same_id_set(&cell.robot_ids, &robot_ids)
        || robots.iter().any(|robot| {
            robot.cell_id != cell.id
                || robot.kinematic_chain.is_empty()
                || robot.joint_ids.is_empty()
                || robot.tool_mount_ref.equipment_id.is_empty()
                || robot.tool_mount_ref.role.is_empty()
                || !robot.workspace_bounds.reach_radius_mm.is_finite()
                || robot.workspace_bounds.reach_radius_mm <= 0.0
                || !robot.workspace_bounds.vertical_span_mm.is_finite()
                || robot.workspace_bounds.vertical_span_mm <= 0.0
                || !robot.payload_limits.nominal_kg.is_finite()
                || robot.payload_limits.nominal_kg <= 0.0
                || !robot.payload_limits.max_kg.is_finite()
                || robot.payload_limits.max_kg < robot.payload_limits.nominal_kg
                || robot.calibration_state.is_empty()
        })
    {
        return Err(RoboticsError::InvalidRobotModelSet);
    }

    let equipment_ids = equipment
        .iter()
        .map(|model| model.id.as_str())
        .collect::<Vec<_>>();
    if equipment.len() != cell.equipment_ids.len()
        || !same_id_set(&cell.equipment_ids, &equipment_ids)
        || equipment.iter().any(|model| {
            model.cell_id != cell.id
                || model.assembly_occurrence_id.is_empty()
                || !model.parameter_set.width_mm.is_finite()
                || model.parameter_set.width_mm <= 0.0
                || !model.parameter_set.height_mm.is_finite()
                || model.parameter_set.height_mm <= 0.0
                || !model.parameter_set.depth_mm.is_finite()
                || model.parameter_set.depth_mm <= 0.0
        })
    {
        return Err(RoboticsError::InvalidEquipmentModelSet);
    }

    let sequence_ids = sequences
        .iter()
        .map(|sequence| sequence.id.as_str())
        .collect::<Vec<_>>();
    if sequences.len() != cell.sequence_ids.len()
        || !same_id_set(&cell.sequence_ids, &sequence_ids)
        || sequences.iter().any(|sequence| {
            sequence.cell_id != cell.id
                || !cell
                    .robot_ids
                    .iter()
                    .any(|robot_id| robot_id == &sequence.robot_id)
                || sequence.target_ids.is_empty()
                || !sequence.path_length_mm.is_finite()
                || sequence.path_length_mm <= 0.0
                || sequence.estimated_cycle_time_ms == 0
        })
    {
        return Err(RoboticsError::InvalidRobotSequenceSet);
    }

    Ok(RobotCellStructureSummary {
        robot_count: cell.robot_ids.len(),
        equipment_count: cell.equipment_ids.len(),
        safety_zone_count: cell.safety_zone_ids.len(),
        sequence_count: cell.sequence_ids.len(),
        controller_count: cell.controller_model_ids.len(),
    })
}

fn distance_between(left: CartesianPose, right: CartesianPose) -> f64 {
    let dx = right.x_mm - left.x_mm;
    let dy = right.y_mm - left.y_mm;
    let dz = right.z_mm - left.z_mm;

    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn ids_are_unique(ids: &[String]) -> bool {
    let mut known = HashSet::new();
    ids.iter()
        .all(|id| !id.is_empty() && known.insert(id.as_str()))
}

fn same_id_set(expected: &[String], actual: &[&str]) -> bool {
    let expected_set = expected.iter().map(String::as_str).collect::<HashSet<_>>();
    let actual_set = actual.iter().copied().collect::<HashSet<_>>();
    expected_set == actual_set
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

    fn sample_robot_cell_model() -> RobotCellModel {
        RobotCellModel {
            id: "ent_cell_001".to_string(),
            scene_assembly_id: "ent_asm_cell_001".to_string(),
            robot_ids: vec!["ent_robot_001".to_string()],
            equipment_ids: vec![
                "ent_conveyor_001".to_string(),
                "ent_fixture_001".to_string(),
                "ent_tool_001".to_string(),
            ],
            safety_zone_ids: vec![
                "ent_zone_001_warning".to_string(),
                "ent_zone_001_protective".to_string(),
            ],
            sequence_ids: vec!["ent_seq_001".to_string()],
            controller_model_ids: vec!["ent_ctrl_001".to_string()],
        }
    }

    fn sample_robot_model() -> RobotModel {
        RobotModel {
            id: "ent_robot_001".to_string(),
            cell_id: "ent_cell_001".to_string(),
            kinematic_chain: vec![
                "base".to_string(),
                "shoulder".to_string(),
                "wrist".to_string(),
                "tool".to_string(),
            ],
            joint_ids: vec!["joint_axis_001".to_string()],
            tool_mount_ref: RobotToolMountRef {
                equipment_id: "ent_tool_001".to_string(),
                role: "tool".to_string(),
            },
            workspace_bounds: RobotWorkspaceBounds {
                reach_radius_mm: 1_450.0,
                vertical_span_mm: 1_900.0,
            },
            payload_limits: RobotPayloadLimits {
                nominal_kg: 8.0,
                max_kg: 12.0,
            },
            calibration_state: "seeded".to_string(),
        }
    }

    fn sample_equipment_models() -> Vec<EquipmentModel> {
        vec![
            EquipmentModel {
                id: "ent_conveyor_001".to_string(),
                cell_id: "ent_cell_001".to_string(),
                equipment_type: EquipmentType::Conveyor,
                assembly_occurrence_id: "occ_conveyor_001".to_string(),
                parameter_set: EquipmentParameterSet {
                    width_mm: 850.0,
                    height_mm: 220.0,
                    depth_mm: 600.0,
                    nominal_speed_mm_s: Some(320),
                },
                io_port_ids: vec!["sig_cycle_start".to_string()],
            },
            EquipmentModel {
                id: "ent_fixture_001".to_string(),
                cell_id: "ent_cell_001".to_string(),
                equipment_type: EquipmentType::Workstation,
                assembly_occurrence_id: "occ_fixture_001".to_string(),
                parameter_set: EquipmentParameterSet {
                    width_mm: 640.0,
                    height_mm: 180.0,
                    depth_mm: 480.0,
                    nominal_speed_mm_s: None,
                },
                io_port_ids: vec!["sig_progress_gate".to_string()],
            },
            EquipmentModel {
                id: "ent_tool_001".to_string(),
                cell_id: "ent_cell_001".to_string(),
                equipment_type: EquipmentType::Gripper,
                assembly_occurrence_id: "occ_tool_001".to_string(),
                parameter_set: EquipmentParameterSet {
                    width_mm: 110.0,
                    height_mm: 80.0,
                    depth_mm: 140.0,
                    nominal_speed_mm_s: None,
                },
                io_port_ids: vec!["sig_payload_released".to_string()],
            },
        ]
    }

    fn sample_robot_sequence_model() -> RobotSequenceModel {
        RobotSequenceModel {
            id: "ent_seq_001".to_string(),
            cell_id: "ent_cell_001".to_string(),
            robot_id: "ent_robot_001".to_string(),
            target_ids: vec![
                "target_pick".to_string(),
                "target_transfer".to_string(),
                "target_place".to_string(),
            ],
            path_length_mm: 896.0,
            estimated_cycle_time_ms: 3_491,
        }
    }

    #[test]
    fn validates_robot_cell_structure_with_consistent_support_graph() {
        let summary = validate_robot_cell_structure(
            &sample_robot_cell_model(),
            &[sample_robot_model()],
            &sample_equipment_models(),
            &[sample_robot_sequence_model()],
        )
        .expect("structure should validate");

        assert_eq!(
            summary,
            RobotCellStructureSummary {
                robot_count: 1,
                equipment_count: 3,
                safety_zone_count: 2,
                sequence_count: 1,
                controller_count: 1,
            }
        );
    }

    #[test]
    fn rejects_robot_cell_structure_with_invalid_refs_or_counts() {
        let mut cell = sample_robot_cell_model();
        cell.equipment_ids.push("ent_fixture_001".to_string());
        assert_eq!(
            validate_robot_cell_structure(
                &cell,
                &[sample_robot_model()],
                &sample_equipment_models(),
                &[sample_robot_sequence_model()],
            ),
            Err(RoboticsError::InvalidRobotCellStructure)
        );

        let mut bad_sequence = sample_robot_sequence_model();
        bad_sequence.robot_id = "ent_robot_missing".to_string();
        assert_eq!(
            validate_robot_cell_structure(
                &sample_robot_cell_model(),
                &[sample_robot_model()],
                &sample_equipment_models(),
                &[bad_sequence],
            ),
            Err(RoboticsError::InvalidRobotSequenceSet)
        );
    }
}
