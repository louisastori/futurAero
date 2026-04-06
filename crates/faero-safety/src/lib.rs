#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyZone {
    pub id: String,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyInterlock {
    pub source_zone_id: String,
    pub inhibited_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyEvaluation {
    pub inhibited: bool,
    pub cause_zone_ids: Vec<String>,
}

#[must_use]
pub fn evaluate_safety(
    zones: &[SafetyZone],
    interlocks: &[SafetyInterlock],
    attempted_action: &str,
) -> SafetyEvaluation {
    let cause_zone_ids = interlocks
        .iter()
        .filter(|interlock| interlock.inhibited_action == attempted_action)
        .filter_map(|interlock| {
            zones.iter().find_map(|zone| {
                (zone.id == interlock.source_zone_id && zone.active).then(|| zone.id.clone())
            })
        })
        .collect::<Vec<_>>();

    SafetyEvaluation {
        inhibited: !cause_zone_ids.is_empty(),
        cause_zone_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inhibits_an_action_when_an_active_zone_matches() {
        let evaluation = evaluate_safety(
            &[SafetyZone {
                id: "zone_stop".to_string(),
                active: true,
            }],
            &[SafetyInterlock {
                source_zone_id: "zone_stop".to_string(),
                inhibited_action: "robot.move".to_string(),
            }],
            "robot.move",
        );

        assert!(evaluation.inhibited);
        assert_eq!(evaluation.cause_zone_ids, vec!["zone_stop".to_string()]);
    }

    #[test]
    fn ignores_inactive_zones() {
        let evaluation = evaluate_safety(
            &[SafetyZone {
                id: "zone_stop".to_string(),
                active: false,
            }],
            &[SafetyInterlock {
                source_zone_id: "zone_stop".to_string(),
                inhibited_action: "robot.move".to_string(),
            }],
            "robot.move",
        );

        assert!(!evaluation.inhibited);
        assert!(evaluation.cause_zone_ids.is_empty());
    }

    #[test]
    fn ignores_interlocks_for_other_actions() {
        let evaluation = evaluate_safety(
            &[SafetyZone {
                id: "zone_warn".to_string(),
                active: true,
            }],
            &[SafetyInterlock {
                source_zone_id: "zone_warn".to_string(),
                inhibited_action: "robot.stop".to_string(),
            }],
            "robot.move",
        );

        assert!(!evaluation.inhibited);
    }
}
