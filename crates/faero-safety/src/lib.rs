use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyZoneKind {
    ProtectiveStop,
    Warning,
    LidarProtective,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyZone {
    pub id: String,
    pub kind: SafetyZoneKind,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyInterlock {
    pub id: String,
    pub source_zone_id: String,
    pub inhibited_action: String,
    pub requires_manual_reset: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyStatus {
    Clear,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyEvaluation {
    pub status: SafetyStatus,
    pub inhibited: bool,
    pub cause_zone_ids: Vec<String>,
    pub active_zone_count: usize,
    pub blocking_interlock_count: usize,
    pub advisory_zone_count: usize,
}

#[must_use]
pub fn evaluate_safety(
    zones: &[SafetyZone],
    interlocks: &[SafetyInterlock],
    attempted_action: &str,
) -> SafetyEvaluation {
    let active_zones = zones.iter().filter(|zone| zone.active).collect::<Vec<_>>();
    let cause_zone_ids = interlocks
        .iter()
        .filter(|interlock| interlock.inhibited_action == attempted_action)
        .filter_map(|interlock| {
            active_zones
                .iter()
                .find(|zone| zone.id == interlock.source_zone_id)
                .map(|zone| zone.id.clone())
        })
        .collect::<Vec<_>>();
    let advisory_zone_count = active_zones
        .iter()
        .filter(|zone| matches!(zone.kind, SafetyZoneKind::Warning))
        .count();
    let blocking_interlock_count = cause_zone_ids.len();
    let status = if blocking_interlock_count > 0 {
        SafetyStatus::Blocked
    } else if advisory_zone_count > 0 {
        SafetyStatus::Warning
    } else {
        SafetyStatus::Clear
    };

    SafetyEvaluation {
        status,
        inhibited: blocking_interlock_count > 0,
        cause_zone_ids,
        active_zone_count: active_zones.len(),
        blocking_interlock_count,
        advisory_zone_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_an_action_when_an_active_protective_zone_matches() {
        let evaluation = evaluate_safety(
            &[SafetyZone {
                id: "zone_stop".to_string(),
                kind: SafetyZoneKind::ProtectiveStop,
                active: true,
            }],
            &[SafetyInterlock {
                id: "int_stop".to_string(),
                source_zone_id: "zone_stop".to_string(),
                inhibited_action: "robot.move".to_string(),
                requires_manual_reset: true,
            }],
            "robot.move",
        );

        assert_eq!(evaluation.status, SafetyStatus::Blocked);
        assert!(evaluation.inhibited);
        assert_eq!(evaluation.cause_zone_ids, vec!["zone_stop".to_string()]);
        assert_eq!(evaluation.blocking_interlock_count, 1);
    }

    #[test]
    fn ignores_inactive_zones() {
        let evaluation = evaluate_safety(
            &[SafetyZone {
                id: "zone_stop".to_string(),
                kind: SafetyZoneKind::ProtectiveStop,
                active: false,
            }],
            &[SafetyInterlock {
                id: "int_stop".to_string(),
                source_zone_id: "zone_stop".to_string(),
                inhibited_action: "robot.move".to_string(),
                requires_manual_reset: true,
            }],
            "robot.move",
        );

        assert_eq!(evaluation.status, SafetyStatus::Clear);
        assert!(!evaluation.inhibited);
        assert!(evaluation.cause_zone_ids.is_empty());
    }

    #[test]
    fn warns_when_only_advisory_zones_are_active() {
        let evaluation = evaluate_safety(
            &[SafetyZone {
                id: "zone_warn".to_string(),
                kind: SafetyZoneKind::Warning,
                active: true,
            }],
            &[],
            "robot.move",
        );

        assert_eq!(evaluation.status, SafetyStatus::Warning);
        assert!(!evaluation.inhibited);
        assert_eq!(evaluation.advisory_zone_count, 1);
    }

    #[test]
    fn ignores_interlocks_for_other_actions() {
        let evaluation = evaluate_safety(
            &[SafetyZone {
                id: "zone_warn".to_string(),
                kind: SafetyZoneKind::LidarProtective,
                active: true,
            }],
            &[SafetyInterlock {
                id: "int_stop".to_string(),
                source_zone_id: "zone_warn".to_string(),
                inhibited_action: "robot.stop".to_string(),
                requires_manual_reset: false,
            }],
            "robot.move",
        );

        assert_eq!(evaluation.status, SafetyStatus::Clear);
        assert!(!evaluation.inhibited);
    }
}
