#[derive(Debug, Clone, PartialEq)]
pub struct AsBuiltMeasurement {
    pub id: String,
    pub deviation_mm: f32,
    pub tolerance_mm: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommissioningReport {
    pub accepted_count: usize,
    pub rejected_count: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_measurements_within_tolerance() {
        let report = build_commissioning_report(&[
            AsBuiltMeasurement {
                id: "m1".to_string(),
                deviation_mm: 0.5,
                tolerance_mm: 1.0,
            },
            AsBuiltMeasurement {
                id: "m2".to_string(),
                deviation_mm: 1.5,
                tolerance_mm: 1.0,
            },
        ]);

        assert_eq!(report.accepted_count, 1);
        assert_eq!(report.rejected_count, 1);
    }

    #[test]
    fn handles_empty_measurement_sets() {
        let report = build_commissioning_report(&[]);
        assert_eq!(report.accepted_count, 0);
        assert_eq!(report.rejected_count, 0);
    }

    #[test]
    fn treats_negative_deviation_as_valid_when_within_tolerance() {
        let report = build_commissioning_report(&[AsBuiltMeasurement {
            id: "m1".to_string(),
            deviation_mm: -0.7,
            tolerance_mm: 1.0,
        }]);

        assert_eq!(report.accepted_count, 1);
        assert_eq!(report.rejected_count, 0);
    }
}
