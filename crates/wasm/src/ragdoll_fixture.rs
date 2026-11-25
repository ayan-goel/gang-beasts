use brawler_engine::{
    BalanceConfig, BodyPart, CharacterDefinition, CharacterError, CommandError, LocomotionConfig,
    LocomotionRagdoll, MotorConfig, PassiveRagdoll, PhysicsConfig, PoseRagdoll,
};

#[derive(Clone, Copy)]
pub(crate) enum RigKind {
    Passive,
    Standing,
    Locomotion,
    Stairs,
}

pub(crate) enum RagdollFixture {
    Passive(Box<PassiveRagdoll>),
    Standing(Box<PoseRagdoll>),
    Locomotion(Box<LocomotionRagdoll>),
}

impl RagdollFixture {
    pub fn new(physics: PhysicsConfig, kind: RigKind) -> Result<Self, CharacterError> {
        if matches!(kind, RigKind::Stairs) {
            return Ok(Self::Locomotion(Box::new(LocomotionRagdoll::stairs(
                physics,
                LocomotionConfig::bundled()?,
            )?)));
        }
        if matches!(kind, RigKind::Locomotion) {
            return Ok(Self::Locomotion(Box::new(LocomotionRagdoll::new(
                physics,
                LocomotionConfig::bundled()?,
                [0.0; 3],
            )?)));
        }
        let definition = CharacterDefinition::bundled()?;
        if matches!(kind, RigKind::Standing) {
            let mut rig =
                PoseRagdoll::new(physics, definition, MotorConfig::standing()?, [0.0; 3])?;
            rig.configure_balance(Some(BalanceConfig::bundled()?))?;
            Ok(Self::Standing(Box::new(rig)))
        } else {
            Ok(Self::Passive(Box::new(PassiveRagdoll::new(
                physics,
                definition,
                [0.0, 2.0, 0.0],
            )?)))
        }
    }

    pub fn kind(&self) -> RigKind {
        match self {
            Self::Passive(_) => RigKind::Passive,
            Self::Standing(_) => RigKind::Standing,
            Self::Locomotion(rig) if rig.is_stair_fixture() => RigKind::Stairs,
            Self::Locomotion(_) => RigKind::Locomotion,
        }
    }
    pub fn walking(&self) -> Option<&LocomotionRagdoll> {
        match self {
            Self::Locomotion(rig) => Some(rig),
            _ => None,
        }
    }
    pub fn set_movement(
        &mut self,
        direction: [f32; 2],
        running: bool,
    ) -> Result<(), CharacterError> {
        match self {
            Self::Locomotion(rig) => rig.set_input(direction, running),
            _ => Err(CharacterError::Invalid(
                "this rig has no locomotion controller",
            )),
        }
    }

    pub fn view(&self) -> &PassiveRagdoll {
        match self {
            Self::Passive(rig) => rig,
            Self::Standing(rig) => rig.rig(),
            Self::Locomotion(rig) => rig.pose().rig(),
        }
    }

    pub fn driven(&self) -> Option<&PoseRagdoll> {
        match self {
            Self::Passive(_) => None,
            Self::Standing(rig) => Some(rig),
            Self::Locomotion(rig) => Some(rig.pose()),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), CharacterError> {
        match self {
            Self::Passive(_) => Err(CharacterError::Invalid("passive rig has no controller")),
            Self::Locomotion(rig) => {
                rig.set_enabled(enabled);
                Ok(())
            }
            Self::Standing(rig) => {
                rig.set_enabled(enabled);
                Ok(())
            }
        }
    }

    pub fn step(&mut self) {
        match self {
            Self::Passive(rig) => rig.step(),
            Self::Standing(rig) => rig.step(),
            Self::Locomotion(rig) => rig.step(),
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Passive(rig) => rig.reset(),
            Self::Standing(rig) => rig.reset(),
            Self::Locomotion(rig) => rig.reset(),
        }
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        match self {
            Self::Passive(rig) => rig.collider_lines(),
            Self::Standing(rig) => rig.collider_lines(),
            Self::Locomotion(rig) => rig.collider_lines(),
        }
    }

    pub fn queue_impulse(&mut self, part: BodyPart, impulse: [f32; 3]) -> Result<(), CommandError> {
        match self {
            Self::Passive(rig) => rig.queue_impulse(part, impulse),
            Self::Standing(rig) => rig.queue_impulse(part, impulse),
            Self::Locomotion(rig) => rig.queue_impulse(part, impulse),
        }
    }
}
