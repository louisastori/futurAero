use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SketchConstraintState {
    UnderConstrained,
    WellConstrained,
    OverConstrained,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SketchProfile {
    pub point_count: usize,
    pub solved_constraint_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartRegeneration {
    pub state: SketchConstraintState,
    pub estimated_mass_grams: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GeometryError {
    #[error("sketch profile must contain at least one point")]
    EmptySketch,
    #[error("extrusion depth must be greater than zero")]
    InvalidDepth,
}

#[must_use]
pub fn evaluate_sketch_state(profile: &SketchProfile) -> SketchConstraintState {
    if profile.solved_constraint_count < profile.point_count {
        SketchConstraintState::UnderConstrained
    } else if profile.solved_constraint_count == profile.point_count {
        SketchConstraintState::WellConstrained
    } else {
        SketchConstraintState::OverConstrained
    }
}

pub fn regenerate_extrusion(
    profile: &SketchProfile,
    extrusion_depth_mm: u32,
) -> Result<PartRegeneration, GeometryError> {
    if profile.point_count == 0 {
        return Err(GeometryError::EmptySketch);
    }
    if extrusion_depth_mm == 0 {
        return Err(GeometryError::InvalidDepth);
    }

    let state = evaluate_sketch_state(profile);
    let effective_constraints = profile.solved_constraint_count.max(profile.point_count);
    let estimated_mass_grams =
        (profile.point_count as u32 * extrusion_depth_mm) + (effective_constraints as u32 * 5);

    Ok(PartRegeneration {
        state,
        estimated_mass_grams,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_sketch_state_across_all_constraint_levels() {
        assert_eq!(
            evaluate_sketch_state(&SketchProfile {
                point_count: 4,
                solved_constraint_count: 2,
            }),
            SketchConstraintState::UnderConstrained
        );
        assert_eq!(
            evaluate_sketch_state(&SketchProfile {
                point_count: 4,
                solved_constraint_count: 4,
            }),
            SketchConstraintState::WellConstrained
        );
        assert_eq!(
            evaluate_sketch_state(&SketchProfile {
                point_count: 4,
                solved_constraint_count: 5,
            }),
            SketchConstraintState::OverConstrained
        );
    }

    #[test]
    fn regenerates_extrusion_with_deterministic_mass_estimate() {
        let result = regenerate_extrusion(
            &SketchProfile {
                point_count: 6,
                solved_constraint_count: 6,
            },
            20,
        )
        .expect("valid extrusion should regenerate");

        assert_eq!(result.state, SketchConstraintState::WellConstrained);
        assert_eq!(result.estimated_mass_grams, 150);
    }

    #[test]
    fn rejects_empty_sketches_and_zero_depth() {
        assert_eq!(
            regenerate_extrusion(
                &SketchProfile {
                    point_count: 0,
                    solved_constraint_count: 0,
                },
                10,
            ),
            Err(GeometryError::EmptySketch)
        );
        assert_eq!(
            regenerate_extrusion(
                &SketchProfile {
                    point_count: 2,
                    solved_constraint_count: 2,
                },
                0,
            ),
            Err(GeometryError::InvalidDepth)
        );
    }
}
