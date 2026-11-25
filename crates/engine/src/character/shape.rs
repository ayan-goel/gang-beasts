use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BodyShape {
    Sphere {
        radius: f32,
    },
    Capsule {
        half_height: f32,
        radius: f32,
    },
    RoundedBox {
        half_extents: [f32; 3],
        border_radius: f32,
    },
}

impl BodyShape {
    pub(crate) fn matches_part(&self, part: super::BodyPart) -> bool {
        use super::BodyPart::*;
        match part {
            Head | LeftHand | RightHand => matches!(self, Self::Sphere { .. }),
            LeftFoot | RightFoot => matches!(self, Self::RoundedBox { .. }),
            _ => matches!(self, Self::Capsule { .. }),
        }
    }

    pub(crate) fn valid(&self) -> bool {
        let bounded = |value: f32| value.is_finite() && (0.01..=0.4).contains(&value);
        match self {
            Self::Sphere { radius } => bounded(*radius),
            Self::Capsule {
                half_height,
                radius,
            } => bounded(*half_height) && bounded(*radius),
            Self::RoundedBox {
                half_extents,
                border_radius,
            } => half_extents.iter().all(|value| bounded(*value)) && bounded(*border_radius),
        }
    }

    pub(crate) fn bounding_radius(&self) -> f32 {
        match self {
            Self::Sphere { radius } => *radius,
            Self::Capsule {
                half_height,
                radius,
            } => half_height + radius,
            Self::RoundedBox {
                half_extents,
                border_radius,
            } => {
                half_extents
                    .iter()
                    .map(|value| value * value)
                    .sum::<f32>()
                    .sqrt()
                    + border_radius
            }
        }
    }

    pub(crate) fn collider(&self) -> ColliderBuilder {
        match self {
            Self::Sphere { radius } => ColliderBuilder::ball(*radius),
            Self::Capsule {
                half_height,
                radius,
            } => ColliderBuilder::capsule_y(*half_height, *radius),
            Self::RoundedBox {
                half_extents: [x, y, z],
                border_radius,
            } => ColliderBuilder::round_cuboid(*x, *y, *z, *border_radius),
        }
    }
}
