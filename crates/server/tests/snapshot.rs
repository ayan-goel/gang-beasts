use brawler_engine::{FighterId, Hand};
use brawler_protocol::{Button, InputFrame};
use brawler_server::Room;

#[test]
fn captures_a_complete_world_at_the_half_rate_and_freezes_recipient_acks_at_capture() {
    let mut room = Room::skyline().unwrap();
    let coral = FighterId::First;
    let blue = FighterId::Second;
    for tick in 1..=180 {
        room.step();
        assert_eq!(room.snapshot_due(), tick % 2 == 0);
    }
    let tap = Button::LeftHand as u16;
    room.submit(
        coral,
        1,
        &InputFrame {
            pressed: tap,
            released: tap,
            ..Default::default()
        }
        .encode()
        .unwrap(),
    )
    .unwrap();
    let before = room.capture().unwrap();
    room.step();
    assert!(!room.snapshot_due());
    let after = room.capture().unwrap();
    let old = before.for_recipient(coral);
    let c = after.for_recipient(coral);
    let b = after.for_recipient(blue);
    assert_eq!(old[2], 1);
    assert_eq!(c[2], 3);
    assert_eq!(b[2], 1);
    assert_eq!(c[8..12], [0; 4]);
    assert_eq!(c[4..8], 181u32.to_le_bytes());
    assert_eq!(b[16..], c[16..]);
    assert_eq!(c.len(), 20 + 39 * 32 + 72);
    for (slot, id) in [coral, blue].into_iter().enumerate() {
        for (part, body) in room.game().body_states(id).into_iter().enumerate() {
            let offset = 20 + (slot * 15 + part) * 32;
            assert_eq!(
                c[offset..offset + 4],
                (1 + 16 * slot as u32 + part as u32).to_le_bytes()
            );
            for (i, expected) in body
                .state
                .position
                .into_iter()
                .chain(body.state.rotation)
                .enumerate()
            {
                let at = offset + 4 + i * 4;
                assert_eq!(&c[at..at + 4], &expected.to_le_bytes());
            }
        }
    }
    assert_eq!(
        c[20 + 39 * 32 + 24 + 3],
        room.game().hand_state(coral, Hand::Left).phase as u8
    );
}
