use brawler_engine::{
    BalanceConfig, BodyPart, CharacterDefinition, CharacterError, CommandError, MotorConfig,
    PassiveRagdoll, PhysicsConfig, PoseRagdoll,
};

pub(crate) enum RagdollFixture {
    Passive(Box<PassiveRagdoll>),
    Standing(Box<PoseRagdoll>),
}

impl RagdollFixture {
    pub fn new(physics: PhysicsConfig, standing: bool) -> Result<Self, CharacterError> {
        let definition = CharacterDefinition::bundled()?;
        if standing {
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

    pub fn view(&self) -> &PassiveRagdoll {
        match self {
            Self::Passive(rig) => rig,
            Self::Standing(rig) => rig.rig(),
        }
    }

    pub fn driven(&self) -> Option<&PoseRagdoll> {
        match self {
            Self::Passive(_) => None,
            Self::Standing(rig) => Some(rig),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), CharacterError> {
        match self {
            Self::Passive(_) => Err(CharacterError::Invalid("passive rig has no controller")),
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
        }
    }

    pub fn reset(&mut self) {
        match self {
            Self::Passive(rig) => rig.reset(),
            Self::Standing(rig) => rig.reset(),
        }
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        match self {
            Self::Passive(rig) => rig.collider_lines(),
            Self::Standing(rig) => rig.collider_lines(),
        }
    }

    pub fn queue_impulse(&mut self, part: BodyPart, impulse: [f32; 3]) -> Result<(), CommandError> {
        match self {
            Self::Passive(rig) => rig.queue_impulse(part, impulse),
            Self::Standing(rig) => rig.queue_impulse(part, impulse),
        }
    }
}
