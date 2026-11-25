use crate::recording::{ImpulseRecord, MAX_RECORDED_COMMANDS, MAX_RECORDING_TICKS};
use crate::{FixtureRecording, PhysicsConfig, RecordingError, Simulation};

pub struct FixtureSession {
    simulation: Simulation,
    recording: FixtureRecording,
    replay_cursor: Option<usize>,
    pending_impulse: Option<[f32; 3]>,
    overflowed: bool,
}

impl FixtureSession {
    pub fn new(config: PhysicsConfig) -> Result<Self, RecordingError> {
        Ok(Self {
            simulation: Simulation::new(config.clone())?,
            recording: FixtureRecording::new(config),
            replay_cursor: None,
            pending_impulse: None,
            overflowed: false,
        })
    }

    pub fn simulation(&self) -> &Simulation {
        &self.simulation
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.simulation.collider_lines()
    }

    pub fn replay_end(&self) -> Option<u32> {
        self.replay_cursor.map(|_| self.recording.ticks)
    }

    pub fn recording_available(&self) -> bool {
        !self.overflowed
    }

    pub fn queue_impulse(&mut self, impulse: [f32; 3]) -> Result<(), RecordingError> {
        if self.replay_cursor.is_some() {
            return Err(RecordingError::Invalid("reset before changing a replay"));
        }
        self.simulation.queue_fixture_impulse(impulse)?;
        self.pending_impulse = Some(impulse);
        Ok(())
    }

    pub fn step(&mut self) -> bool {
        if let Some(cursor) = self.replay_cursor {
            if self.simulation.tick() >= u64::from(self.recording.ticks) {
                return false;
            }
            if let Some(command) = self.recording.commands.get(cursor) {
                if u64::from(command.tick) == self.simulation.tick() + 1 {
                    self.simulation
                        .queue_fixture_impulse(command.impulse)
                        .expect("validated replay has one bounded impulse per tick");
                    self.replay_cursor = Some(cursor + 1);
                }
            }
            self.simulation.step();
            return true;
        }
        self.simulation.step();
        let impulse = self.pending_impulse.take();
        if self.simulation.tick() > MAX_RECORDING_TICKS
            || (impulse.is_some() && self.recording.commands.len() == MAX_RECORDED_COMMANDS)
        {
            // Keep the playground usable after its recording budget is exhausted,
            // but never export a silently truncated reproduction as complete.
            self.overflowed = true;
        }
        if !self.overflowed {
            self.recording.ticks = self.simulation.tick() as u32;
            if let Some(impulse) = impulse {
                self.recording.commands.push(ImpulseRecord {
                    tick: self.recording.ticks,
                    impulse,
                });
            }
        }
        true
    }

    pub fn reset(&mut self) {
        *self =
            Self::new(self.recording.config.clone()).expect("stored configuration was validated");
    }

    pub fn configure(&mut self, json: &str) -> Result<(), RecordingError> {
        // Construct first so a rejected tuning change cannot destroy the live fixture.
        let replacement = Self::new(PhysicsConfig::from_json(json)?)?;
        *self = replacement;
        Ok(())
    }

    pub fn configuration_json(&self) -> String {
        serde_json::to_string_pretty(&self.recording.config).expect("validated configuration")
    }

    pub fn export_recording(&self) -> Result<String, RecordingError> {
        if self.overflowed {
            return Err(RecordingError::LimitReached);
        }
        Ok(serde_json::to_string(&self.recording)?)
    }

    pub fn from_recording(json: &str) -> Result<Self, RecordingError> {
        let recording = FixtureRecording::from_json(json)?;
        Ok(Self {
            simulation: Simulation::new(recording.config.clone())?,
            recording,
            replay_cursor: Some(0),
            pending_impulse: None,
            overflowed: false,
        })
    }

    pub fn import_recording(&mut self, json: &str) -> Result<(), RecordingError> {
        let replacement = Self::from_recording(json)?;
        *self = replacement;
        Ok(())
    }
}
