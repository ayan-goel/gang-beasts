use brawler_engine::{BodyPart, Consciousness, FighterId, Hand, HandPhase, MeleePhase};
use brawler_protocol::{Button, InputFrame};
use brawler_server::Room;

const HOLDER: FighterId = FighterId::First;
const TARGET: FighterId = FighterId::Second;
const HAND: Hand = Hand::Left;

fn steer_and_hold(room: &mut Room, first: bool) {
    let pelvis = |id| {
        room.game().body_states(id)[BodyPart::Pelvis.index()]
            .state
            .position
    };
    let a = pelvis(HOLDER);
    let b = pelvis(TARGET);
    let delta = [b[0] - a[0], b[2] - a[2]];
    let length = delta[0].hypot(delta[1]).max(0.001);
    let left = Button::LeftHand as u16;
    let frame = InputFrame {
        sequence: room.tick() as u32,
        tick: room.tick() as u32,
        axes: delta.map(|v| (v / length * 127.).round() as i8),
        held: left,
        pressed: if first { left } else { 0 },
        ..Default::default()
    };
    room.submit(HOLDER, 1, &frame.encode().unwrap()).unwrap();
}

fn gripping() -> Room {
    let mut room = Room::skyline_with_players(3).unwrap();
    for _ in 0..180 {
        room.step();
    }
    for tick in 0..600 {
        steer_and_hold(&mut room, tick == 0);
        room.step();
        if room
            .game()
            .grip(HOLDER, HAND)
            .is_some_and(|g| g.target == TARGET)
        {
            return room;
        }
    }
    panic!("The hand never caught the opponent");
}

#[test]
fn detaching_a_holder_clears_the_joint_and_queued_hold_before_rebinding() {
    let mut room = gripping();
    let held_part = room.game().grip(HOLDER, HAND).unwrap().region;
    steer_and_hold(&mut room, false);
    room.detach(HOLDER);
    assert!(room.game().grip(HOLDER, HAND).is_none());
    room.attach(HOLDER);
    for _ in 0..60 {
        room.step();
        assert_eq!(room.game().hand_state(HOLDER, HAND).phase, HandPhase::Idle);
        assert!(room.game().grip(HOLDER, HAND).is_none());
    }
    // The formerly held limb must travel independently of the palm after the logical grip disappears.
    for _ in 0..120 {
        let walk = InputFrame {
            sequence: room.tick() as u32,
            axes: [127, 0],
            ..Default::default()
        };
        room.submit(TARGET, 1, &walk.encode().unwrap()).unwrap();
        room.step();
    }
    let palm = room.game().body_states(HOLDER)[BodyPart::LeftHand.index()]
        .state
        .position;
    let limb = room.game().body_states(TARGET)[held_part.index()]
        .state
        .position;
    let separation = (palm[0] - limb[0]).hypot(palm[2] - limb[2]);
    assert!(separation > 2., "released palm to limb: {separation} m");
    assert_eq!(room.game().match_state().unwrap().scores(), &[0, 0, 0]);
}

#[test]
fn a_grip_survives_target_detachment_and_is_removed_on_forfeit() {
    let mut room = gripping();
    room.detach(TARGET);
    room.step();
    assert_eq!(room.game().grip(HOLDER, HAND).unwrap().target, TARGET);
    room.step_with_forfeits(1 << TARGET as u8);
    for _ in 0..60 {
        assert!(room.game().grip(HOLDER, HAND).is_none());
        assert_eq!(room.game().vitals(TARGET).phase, Consciousness::Eliminated);
        steer_and_hold(&mut room, false);
        room.step();
    }
    let state = room.game().match_state().unwrap();
    assert_eq!(state.phase, MeleePhase::Playing);
    assert_eq!(state.scores(), &[0, 0, 0]);
}

#[test]
fn a_silent_holder_loses_a_live_grip_on_the_first_stale_solve() {
    let mut room = gripping();
    steer_and_hold(&mut room, false);
    for _ in 0..15 {
        room.step();
        assert_eq!(room.game().grip(HOLDER, HAND).unwrap().target, TARGET);
    }
    room.step();
    assert!(room.game().grip(HOLDER, HAND).is_none());
    assert_eq!(room.game().hand_state(HOLDER, HAND).phase, HandPhase::Idle);
    assert_ne!(room.game().vitals(TARGET).phase, Consciousness::Eliminated);
    assert_eq!(room.game().match_state().unwrap().scores(), &[0, 0, 0]);
}
