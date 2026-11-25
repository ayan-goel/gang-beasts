use super::{channel::Command, *};
use brawler_protocol::InputFrame;

#[test]
fn slot_saturation_and_expired_arrivals_leave_the_other_slot_operable() {
    let mut room = Room::skyline().unwrap();
    for _ in 0..180 {
        room.step();
    }
    let (client, [coral, blue]) = RoomClient::channel();
    let first = FighterId::First;
    let second = FighterId::Second;
    let bytes = InputFrame::default().encode().unwrap();
    for _ in 0..8 {
        client.input(first, 1, &bytes).unwrap();
    }
    assert_eq!(client.input(first, 1, &bytes), Err(DispatchError::Busy));
    client.input(second, 1, &bytes).unwrap();
    let mut last = None;
    drain(&mut room, first, &coral, &client.shared, &mut last);
    assert_eq!(client.rejected_inputs(), 7);
    drain(&mut room, second, &blue, &client.shared, &mut None);
    room.step();
    assert_eq!(room.applied(first), Some(0));
    assert_eq!(room.applied(second), Some(0));

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let stale = Instant::now() - Duration::from_millis(251);
    let next = InputFrame {
        sequence: 1,
        ..Default::default()
    }
    .encode()
    .unwrap();
    tx.send(Command::Input {
        round: 1,
        bytes: next,
        received: stale,
    })
    .unwrap();
    last = Some(stale);
    drain(&mut room, first, &rx, &client.shared, &mut last);
    room.step();
    assert_eq!(last, None);
    assert_eq!(client.rejected_inputs(), 8);
    assert_eq!(room.applied(first), Some(0));
}

#[test]
fn detach_fences_following_inputs_until_reattached_in_command_order() {
    let mut room = Room::skyline().unwrap();
    for _ in 0..180 {
        room.step();
    }
    let (client, [rx, _]) = RoomClient::channel();
    let id = FighterId::First;
    let bytes = InputFrame::default().encode().unwrap();
    client.input(id, 1, &bytes).unwrap();
    client.detach(id).unwrap();
    client.input(id, 1, &bytes).unwrap();
    drain(&mut room, id, &rx, &client.shared, &mut None);
    room.step();
    assert_eq!(room.applied(id), None);
    assert_eq!(client.rejected_inputs(), 1);
    client.attach(id).unwrap();
    client
        .input(
            id,
            1,
            &InputFrame {
                sequence: 1,
                ..Default::default()
            }
            .encode()
            .unwrap(),
        )
        .unwrap();
    drain(&mut room, id, &rx, &client.shared, &mut None);
    room.step();
    assert_eq!(room.applied(id), Some(1));
}
