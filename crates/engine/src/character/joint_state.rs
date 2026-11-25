use super::BodyPart;

pub struct RagdollJointState {
    pub parent: BodyPart,
    pub child: BodyPart,
    pub parent_anchor: [f32; 3],
    pub child_anchor: [f32; 3],
    pub parent_rotation: [f32; 4],
    pub child_rotation: [f32; 4],
    pub anchor_error: f32,
    pub contacts_enabled: bool,
}

impl RagdollJointState {
    pub fn to_buffer(&self) -> [f32; 17] {
        let mut buffer = [0.0; 17];
        buffer[0] = self.parent.index() as f32;
        buffer[1] = self.child.index() as f32;
        buffer[2..5].copy_from_slice(&self.parent_anchor);
        buffer[5..8].copy_from_slice(&self.child_anchor);
        buffer[8..12].copy_from_slice(&self.parent_rotation);
        buffer[12..16].copy_from_slice(&self.child_rotation);
        buffer[16] = self.anchor_error;
        buffer
    }
}
