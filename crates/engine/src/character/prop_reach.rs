use super::{BodyPart, rig_frame::ReadRig};
use crate::{ArenaKind, Hand, HandPhase, arena::Skyline, combat::Hands};
use rapier3d::prelude::*;

pub(super) fn aim(
    rig: ReadRig<'_>,
    map: &Skyline,
    hands: &Hands,
    targets: &mut [Rotation<Real>; 14],
) {
    let chest = &rig.world.bodies[rig.parts.bodies[BodyPart::Chest.index()]];
    let rest = |part: BodyPart| Vector::from(rig.parts.definition.bodies[part.index()].position);
    for hand in [Hand::Left, Hand::Right] {
        if hands.state(hand).phase != HandPhase::Reach {
            continue;
        }
        let shoulder_part = hand.parts()[0];
        let (index, shoulder) = rig
            .parts
            .definition
            .joints
            .iter()
            .enumerate()
            .find(|(_, j)| j.child == shoulder_part)
            .unwrap();
        let anchor = Vector::from(shoulder.anchor);
        let shoulder_world = chest.position() * Point::from(anchor - rest(BodyPart::Chest));
        let length = (anchor - rest(hand.body())).norm();
        let palm = rig.world.bodies[rig.parts.bodies[hand.body().index()]].center_of_mass();
        let closest = map
            .grip_surfaces(rig.world)
            .filter_map(|(id, _, handle)| {
                if !matches!(
                    map.definition.objects[usize::from(id - 1)].kind,
                    ArenaKind::Crate | ArenaKind::Vent
                ) {
                    return None;
                }
                let c = &rig.world.colliders[handle];
                let point = c
                    .shape()
                    .project_point(c.position(), &shoulder_world, true)
                    .point;
                let forward = chest.rotation() * -Vector::z();
                let delta = chest.rotation().inverse() * (point - shoulder_world);
                (delta.norm() <= length * 1.5
                    && (c.translation() - shoulder_world.coords).dot(&forward) > 0.
                    && delta.y < 0.)
                    .then_some(((point - palm).norm_squared(), delta))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, direction)) = closest {
            // Begin aiming half an arm length before contact so the motors can turn during approach.
            // The ordinary motor cap and short contact query still govern actual acquisition.
            let pitch = (-direction.z).max(0.).atan2(-direction.y);
            let [min, max] = shoulder.angular_limits[0];
            targets[index] = Rotation::from_euler_angles(pitch.clamp(min, max), 0., 0.);
        }
    }
}
