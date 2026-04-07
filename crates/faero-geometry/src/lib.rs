use serde::{Deserialize, Serialize};
use thiserror::Error;

const GEOMETRY_EPSILON: f64 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SketchConstraintState {
    UnderConstrained,
    WellConstrained,
    OverConstrained,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SketchPoint {
    pub x_mm: f64,
    pub y_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SketchProfile {
    pub points: Vec<SketchPoint>,
    pub solved_constraint_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SketchMetrics {
    pub state: SketchConstraintState,
    pub point_count: usize,
    pub perimeter_mm: f64,
    pub area_mm2: f64,
    pub centroid_x_mm: f64,
    pub centroid_y_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtrusionDefinition {
    pub depth_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialProfile {
    pub name: String,
    pub density_kg_m3: f64,
}

impl MaterialProfile {
    #[must_use]
    pub fn aluminum_6061() -> Self {
        Self {
            name: "Aluminum 6061".to_string(),
            density_kg_m3: 2_700.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartRegeneration {
    pub state: SketchConstraintState,
    pub point_count: usize,
    pub closed_profile: bool,
    pub perimeter_mm: f64,
    pub area_mm2: f64,
    pub centroid_x_mm: f64,
    pub centroid_y_mm: f64,
    pub depth_mm: f64,
    pub volume_mm3: f64,
    pub estimated_mass_grams: f64,
    pub material_name: String,
}

#[derive(Debug, Error, PartialEq)]
pub enum GeometryError {
    #[error("sketch profile must contain at least one point")]
    EmptySketch,
    #[error("sketch profile must contain at least three distinct points")]
    NotEnoughPoints,
    #[error("sketch profile contains a non-finite coordinate")]
    NonFiniteCoordinate,
    #[error("sketch profile area must be greater than zero")]
    DegenerateProfile,
    #[error("extrusion depth must be greater than zero")]
    InvalidDepth,
    #[error("material density must be greater than zero")]
    InvalidDensity,
    #[error("rectangle dimensions must be greater than zero")]
    InvalidRectangleDimensions,
}

#[must_use]
pub fn evaluate_sketch_state(
    point_count: usize,
    solved_constraint_count: usize,
) -> SketchConstraintState {
    if solved_constraint_count < point_count {
        SketchConstraintState::UnderConstrained
    } else if solved_constraint_count == point_count {
        SketchConstraintState::WellConstrained
    } else {
        SketchConstraintState::OverConstrained
    }
}

pub fn rectangular_profile(
    width_mm: f64,
    height_mm: f64,
    solved_constraint_count: usize,
) -> Result<SketchProfile, GeometryError> {
    if width_mm <= 0.0 || height_mm <= 0.0 {
        return Err(GeometryError::InvalidRectangleDimensions);
    }

    Ok(SketchProfile {
        points: vec![
            SketchPoint {
                x_mm: 0.0,
                y_mm: 0.0,
            },
            SketchPoint {
                x_mm: width_mm,
                y_mm: 0.0,
            },
            SketchPoint {
                x_mm: width_mm,
                y_mm: height_mm,
            },
            SketchPoint {
                x_mm: 0.0,
                y_mm: height_mm,
            },
        ],
        solved_constraint_count,
    })
}

pub fn analyze_sketch_profile(profile: &SketchProfile) -> Result<SketchMetrics, GeometryError> {
    let points = normalized_points(profile)?;
    let point_count = points.len();
    let signed_twice_area = signed_twice_area(&points);
    let area_twice_abs = signed_twice_area.abs();
    if area_twice_abs <= GEOMETRY_EPSILON {
        return Err(GeometryError::DegenerateProfile);
    }

    let perimeter_mm = polygon_perimeter(&points);
    let area_mm2 = area_twice_abs / 2.0;
    let centroid_factor = 3.0 * signed_twice_area;
    let (centroid_x_sum, centroid_y_sum) = centroid_accumulators(&points);
    let centroid_x_mm = centroid_x_sum / centroid_factor;
    let centroid_y_mm = centroid_y_sum / centroid_factor;

    Ok(SketchMetrics {
        state: evaluate_sketch_state(point_count, profile.solved_constraint_count),
        point_count,
        perimeter_mm,
        area_mm2,
        centroid_x_mm,
        centroid_y_mm,
    })
}

pub fn regenerate_extrusion(
    profile: &SketchProfile,
    extrusion: &ExtrusionDefinition,
    material: &MaterialProfile,
) -> Result<PartRegeneration, GeometryError> {
    if extrusion.depth_mm <= 0.0 || !extrusion.depth_mm.is_finite() {
        return Err(GeometryError::InvalidDepth);
    }
    if material.density_kg_m3 <= 0.0 || !material.density_kg_m3.is_finite() {
        return Err(GeometryError::InvalidDensity);
    }

    let metrics = analyze_sketch_profile(profile)?;
    let volume_mm3 = metrics.area_mm2 * extrusion.depth_mm;
    let estimated_mass_grams = volume_mm3 * material.density_kg_m3 * 1e-6;

    Ok(PartRegeneration {
        state: metrics.state,
        point_count: metrics.point_count,
        closed_profile: true,
        perimeter_mm: metrics.perimeter_mm,
        area_mm2: metrics.area_mm2,
        centroid_x_mm: metrics.centroid_x_mm,
        centroid_y_mm: metrics.centroid_y_mm,
        depth_mm: extrusion.depth_mm,
        volume_mm3,
        estimated_mass_grams,
        material_name: material.name.clone(),
    })
}

fn normalized_points(profile: &SketchProfile) -> Result<Vec<SketchPoint>, GeometryError> {
    if profile.points.is_empty() {
        return Err(GeometryError::EmptySketch);
    }

    let mut points = profile.points.clone();
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }

    if points.len() < 3 {
        return Err(GeometryError::NotEnoughPoints);
    }

    if points
        .iter()
        .any(|point| !point.x_mm.is_finite() || !point.y_mm.is_finite())
    {
        return Err(GeometryError::NonFiniteCoordinate);
    }

    Ok(points)
}

fn polygon_perimeter(points: &[SketchPoint]) -> f64 {
    polygon_edges(points)
        .map(|(left, right)| distance(left, right))
        .sum()
}

fn signed_twice_area(points: &[SketchPoint]) -> f64 {
    polygon_edges(points)
        .map(|(left, right)| (left.x_mm * right.y_mm) - (right.x_mm * left.y_mm))
        .sum()
}

fn centroid_accumulators(points: &[SketchPoint]) -> (f64, f64) {
    polygon_edges(points).fold((0.0, 0.0), |(x_sum, y_sum), (left, right)| {
        let cross = (left.x_mm * right.y_mm) - (right.x_mm * left.y_mm);
        (
            x_sum + ((left.x_mm + right.x_mm) * cross),
            y_sum + ((left.y_mm + right.y_mm) * cross),
        )
    })
}

fn polygon_edges(points: &[SketchPoint]) -> impl Iterator<Item = (SketchPoint, SketchPoint)> + '_ {
    points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
}

fn distance(left: SketchPoint, right: SketchPoint) -> f64 {
    let dx = right.x_mm - left.x_mm;
    let dy = right.y_mm - left.y_mm;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta < 1e-6,
            "expected {expected}, got {actual} (delta {delta})"
        );
    }

    #[test]
    fn evaluates_sketch_state_across_all_constraint_levels() {
        assert_eq!(
            evaluate_sketch_state(4, 2),
            SketchConstraintState::UnderConstrained
        );
        assert_eq!(
            evaluate_sketch_state(4, 4),
            SketchConstraintState::WellConstrained
        );
        assert_eq!(
            evaluate_sketch_state(4, 5),
            SketchConstraintState::OverConstrained
        );
    }

    #[test]
    fn analyzes_rectangular_profile_with_real_metrics() {
        let profile =
            rectangular_profile(120.0, 80.0, 4).expect("rectangle profile should be valid");

        let metrics = analyze_sketch_profile(&profile).expect("profile should analyze");

        assert_eq!(metrics.state, SketchConstraintState::WellConstrained);
        assert_eq!(metrics.point_count, 4);
        assert_close(metrics.perimeter_mm, 400.0);
        assert_close(metrics.area_mm2, 9_600.0);
        assert_close(metrics.centroid_x_mm, 60.0);
        assert_close(metrics.centroid_y_mm, 40.0);
    }

    #[test]
    fn regenerates_extrusion_with_volume_and_mass() {
        let profile =
            rectangular_profile(120.0, 80.0, 4).expect("rectangle profile should be valid");
        let extrusion = ExtrusionDefinition { depth_mm: 15.0 };
        let material = MaterialProfile::aluminum_6061();

        let result =
            regenerate_extrusion(&profile, &extrusion, &material).expect("extrusion should work");

        assert_eq!(result.state, SketchConstraintState::WellConstrained);
        assert!(result.closed_profile);
        assert_close(result.volume_mm3, 144_000.0);
        assert_close(result.estimated_mass_grams, 388.8);
        assert_eq!(result.material_name, "Aluminum 6061");
    }

    #[test]
    fn accepts_profiles_with_redundant_closing_point() {
        let profile = SketchProfile {
            points: vec![
                SketchPoint {
                    x_mm: 0.0,
                    y_mm: 0.0,
                },
                SketchPoint {
                    x_mm: 50.0,
                    y_mm: 0.0,
                },
                SketchPoint {
                    x_mm: 50.0,
                    y_mm: 50.0,
                },
                SketchPoint {
                    x_mm: 0.0,
                    y_mm: 50.0,
                },
                SketchPoint {
                    x_mm: 0.0,
                    y_mm: 0.0,
                },
            ],
            solved_constraint_count: 4,
        };

        let metrics = analyze_sketch_profile(&profile).expect("closed profile should analyze");

        assert_eq!(metrics.point_count, 4);
        assert_close(metrics.area_mm2, 2_500.0);
    }

    #[test]
    fn rejects_invalid_profiles_and_parameters() {
        assert_eq!(
            analyze_sketch_profile(&SketchProfile {
                points: vec![],
                solved_constraint_count: 0,
            }),
            Err(GeometryError::EmptySketch)
        );
        assert_eq!(
            analyze_sketch_profile(&SketchProfile {
                points: vec![
                    SketchPoint {
                        x_mm: 0.0,
                        y_mm: 0.0
                    },
                    SketchPoint {
                        x_mm: 10.0,
                        y_mm: 0.0
                    },
                ],
                solved_constraint_count: 1,
            }),
            Err(GeometryError::NotEnoughPoints)
        );
        assert_eq!(
            analyze_sketch_profile(&SketchProfile {
                points: vec![
                    SketchPoint {
                        x_mm: 0.0,
                        y_mm: 0.0
                    },
                    SketchPoint {
                        x_mm: 10.0,
                        y_mm: 10.0
                    },
                    SketchPoint {
                        x_mm: 20.0,
                        y_mm: 20.0
                    },
                ],
                solved_constraint_count: 3,
            }),
            Err(GeometryError::DegenerateProfile)
        );
        assert_eq!(
            analyze_sketch_profile(&SketchProfile {
                points: vec![
                    SketchPoint {
                        x_mm: 0.0,
                        y_mm: 0.0
                    },
                    SketchPoint {
                        x_mm: f64::NAN,
                        y_mm: 10.0,
                    },
                    SketchPoint {
                        x_mm: 20.0,
                        y_mm: 20.0
                    },
                ],
                solved_constraint_count: 3,
            }),
            Err(GeometryError::NonFiniteCoordinate)
        );
        assert_eq!(
            rectangular_profile(0.0, 25.0, 4),
            Err(GeometryError::InvalidRectangleDimensions)
        );
        assert_eq!(
            regenerate_extrusion(
                &rectangular_profile(50.0, 25.0, 4).expect("valid profile"),
                &ExtrusionDefinition { depth_mm: 0.0 },
                &MaterialProfile::aluminum_6061(),
            ),
            Err(GeometryError::InvalidDepth)
        );
        assert_eq!(
            regenerate_extrusion(
                &rectangular_profile(50.0, 25.0, 4).expect("valid profile"),
                &ExtrusionDefinition { depth_mm: 10.0 },
                &MaterialProfile {
                    name: "Void".to_string(),
                    density_kg_m3: 0.0,
                },
            ),
            Err(GeometryError::InvalidDensity)
        );
    }
}
