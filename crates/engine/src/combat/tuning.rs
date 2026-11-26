use crate::CharacterError;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HandTuning {
    schema_version: u8,
    pub aim_cone_cos: f32,
    pub aim_max_rad: f32,
    pub windup_ticks: u8,
    pub strike_ticks: u8,
    pub recovery_ticks: u8,
    pub windup_pose_rad: [f32; 2],
    pub strike_pose_rad: [f32; 2],
    pub lift_pose_rad: [f32; 2],
    pub lift_force_per_hand_n: f32,
    pub lift_chest_clearance_m: f32,
    pub reach_pose_rad: [f32; 2],
    pub stiffness_nm_per_rad: f32,
    pub damping_nm_s_per_rad: f32,
    pub max_torque_nm: f32,
}

impl HandTuning {
    pub fn bundled() -> Result<Self, CharacterError> {
        let tuning: Self = serde_json::from_str(include_str!("../../../../config/hands.json"))?;
        let valid_pose = |[shoulder, elbow]: [f32; 2]| {
            (-2.4..=2.4).contains(&shoulder) && (0.0..=2.4).contains(&elbow)
        };
        if tuning.schema_version != 1
            || !(0.5..=1.).contains(&tuning.aim_cone_cos)
            || !(0.0..=0.7).contains(&tuning.aim_max_rad)
            || ![
                tuning.windup_ticks,
                tuning.strike_ticks,
                tuning.recovery_ticks,
            ]
            .into_iter()
            .all(|ticks| (1..=60).contains(&ticks))
            || ![
                tuning.windup_pose_rad,
                tuning.strike_pose_rad,
                tuning.reach_pose_rad,
                tuning.lift_pose_rad,
            ]
            .into_iter()
            .all(valid_pose)
            || !(0.0..=450.).contains(&tuning.lift_force_per_hand_n)
            || !(0.05..=0.5).contains(&tuning.lift_chest_clearance_m)
            || !(0.0..=600.0).contains(&tuning.stiffness_nm_per_rad)
            || !(0.0..=20.0).contains(&tuning.damping_nm_s_per_rad)
            || !(0.0..=150.0).contains(&tuning.max_torque_nm)
        {
            return Err(CharacterError::Invalid(
                "hand timing, pose or motor effort out of bounds",
            ));
        }
        Ok(tuning)
    }
    pub fn length(&self) -> u8 {
        self.windup_ticks + self.strike_ticks + self.recovery_ticks
    }
}
