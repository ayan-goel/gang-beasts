use super::{BodyPart, BodyShape};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CharacterError {
    #[error("invalid character: {0}")]
    Invalid(&'static str),
    #[error("invalid character JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Physics(#[from] crate::ConfigError),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyDefinition {
    pub part: BodyPart,
    pub mass: f32,
    pub position: [f32; 3],
    pub shape: BodyShape,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointDefinition {
    pub parent: BodyPart,
    pub child: BodyPart,
    pub anchor: [f32; 3],
    pub angular_limits: [[f32; 2]; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterDefinition {
    pub schema_version: u32,
    pub bodies: [BodyDefinition; 15],
    pub joints: [JointDefinition; 14],
}

impl CharacterDefinition {
    pub fn bundled() -> Result<Self, CharacterError> {
        Self::from_json(include_str!("../../../../config/character.json"))
    }

    pub fn from_json(json: &str) -> Result<Self, CharacterError> {
        // Bound allocation before parsing assets; a rig cannot allocate arbitrary bodies.
        if json.len() > 32_768 {
            return Err(CharacterError::Invalid("asset exceeds 32 KiB"));
        }
        let definition: Self = serde_json::from_str(json)?;
        definition.validate()?;
        Ok(definition)
    }

    pub fn validate(&self) -> Result<(), CharacterError> {
        if self.schema_version != 1 {
            return Err(CharacterError::Invalid("unsupported schema"));
        }
        for (body, part) in self.bodies.iter().zip(BodyPart::ALL) {
            // INVARIANT: the release skeleton retains its specified topology and 40 kg mass.
            // Canonical ordering also gives render buffers stable body-part identities.
            if body.part != part || !body.mass.is_finite() || (body.mass - part.mass()).abs() > 1e-6
            {
                return Err(CharacterError::Invalid(
                    "body order or anatomical mass mismatch",
                ));
            }
            if !bounded_point(body.position)
                || !body.shape.valid()
                || !body.shape.matches_part(part)
            {
                return Err(CharacterError::Invalid("invalid body geometry"));
            }
        }
        let mut children = [false; 15];
        for joint in &self.joints {
            if joint.child.parent() != Some(joint.parent) || children[joint.child.index()] {
                return Err(CharacterError::Invalid(
                    "invalid or duplicate anatomical joint",
                ));
            }
            children[joint.child.index()] = true;
            if !bounded_point(joint.anchor)
                || !joint.child.valid_angular_axes(joint.angular_limits)
                || joint.angular_limits.iter().any(|[min, max]| {
                    !min.is_finite()
                        || !max.is_finite()
                        || *min < -3.0
                        || *max > 3.0
                        || *min > 0.0
                        || *max < 0.0
                })
            {
                return Err(CharacterError::Invalid("invalid anchor or angular limits"));
            }
            for part in [joint.parent, joint.child] {
                let body = &self.bodies[part.index()];
                let distance = joint
                    .anchor
                    .iter()
                    .zip(body.position)
                    .map(|(anchor, center)| (anchor - center).powi(2))
                    .sum::<f32>()
                    .sqrt();
                if distance > body.shape.bounding_radius() + 0.05 {
                    return Err(CharacterError::Invalid(
                        "anchor too far from connected body",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_spawn(&self, spawn: [f32; 3]) -> Result<(), CharacterError> {
        // Origin-only checks miss authored body offsets. Check actual collider
        // bounds so a valid origin cannot create a rig inside/below the floor.
        for body in &self.bodies {
            let bounds = body.shape.collider().build().compute_aabb();
            for (axis, offset) in spawn.iter().enumerate() {
                let min = bounds.mins[axis] + body.position[axis] + offset;
                let max = bounds.maxs[axis] + body.position[axis] + offset;
                if (axis == 1 && min < -0.005) || (axis != 1 && (min < -6.0 || max > 6.0)) {
                    return Err(CharacterError::Invalid(
                        "body outside diagnostic spawn region",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn bounded_point(point: [f32; 3]) -> bool {
    point
        .iter()
        .all(|value| value.is_finite() && value.abs() <= 2.0)
}
