use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum BodyPart {
    Head,
    Chest,
    Pelvis,
    LeftUpperArm,
    RightUpperArm,
    LeftForearm,
    RightForearm,
    LeftHand,
    RightHand,
    LeftThigh,
    RightThigh,
    LeftShin,
    RightShin,
    LeftFoot,
    RightFoot,
}

impl BodyPart {
    pub const ALL: [Self; 15] = [
        Self::Head,
        Self::Chest,
        Self::Pelvis,
        Self::LeftUpperArm,
        Self::RightUpperArm,
        Self::LeftForearm,
        Self::RightForearm,
        Self::LeftHand,
        Self::RightHand,
        Self::LeftThigh,
        Self::RightThigh,
        Self::LeftShin,
        Self::RightShin,
        Self::LeftFoot,
        Self::RightFoot,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn mass(self) -> f32 {
        match self {
            Self::Head => 4.0,
            Self::Chest => 10.0,
            Self::Pelvis => 6.0,
            Self::LeftUpperArm | Self::RightUpperArm => 1.5,
            Self::LeftForearm | Self::RightForearm | Self::LeftFoot | Self::RightFoot => 1.0,
            Self::LeftHand | Self::RightHand => 0.5,
            Self::LeftThigh | Self::RightThigh => 3.5,
            Self::LeftShin | Self::RightShin => 2.5,
        }
    }

    pub fn parent(self) -> Option<Self> {
        Some(match self {
            Self::Pelvis => return None,
            Self::Chest | Self::LeftThigh | Self::RightThigh => Self::Pelvis,
            Self::Head | Self::LeftUpperArm | Self::RightUpperArm => Self::Chest,
            Self::LeftForearm => Self::LeftUpperArm,
            Self::RightForearm => Self::RightUpperArm,
            Self::LeftHand => Self::LeftForearm,
            Self::RightHand => Self::RightForearm,
            Self::LeftShin => Self::LeftThigh,
            Self::RightShin => Self::RightThigh,
            Self::LeftFoot => Self::LeftShin,
            Self::RightFoot => Self::RightShin,
        })
    }

    pub(crate) fn valid_angular_axes(self, limits: [[f32; 2]; 3]) -> bool {
        match self {
            Self::LeftHand | Self::RightHand => limits == [[0.0; 2]; 3],
            Self::LeftForearm | Self::RightForearm => {
                limits[0][0] == 0.0 && limits[0][1] > 0.0 && limits[1..] == [[0.0; 2]; 2]
            }
            Self::LeftShin | Self::RightShin => {
                limits[0][0] < 0.0 && limits[0][1] == 0.0 && limits[1..] == [[0.0; 2]; 2]
            }
            _ => limits.iter().all(|[min, max]| *min < 0.0 && *max > 0.0),
        }
    }
}
