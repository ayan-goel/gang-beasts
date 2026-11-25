use brawler_protocol::{Button, InputError, InputFrame, is_newer};
use serde_json::Value;

#[test]
fn browser_golden_record_decodes_exactly_and_reencodes_byte_for_byte() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/protocol/input-v1.json"
    ))
    .unwrap();
    let bytes: Vec<u8> = serde_json::from_value(fixture["bytes"].clone()).unwrap();
    let frame = InputFrame::decode(&bytes).unwrap();
    assert_eq!(frame.sequence, fixture["sequence"].as_u64().unwrap() as u32);
    assert_eq!(frame.tick, fixture["tick"].as_u64().unwrap() as u32);
    assert_eq!(frame.axes, [-127, 127]);
    assert_eq!([frame.held, frame.pressed, frame.released], [545, 37, 38]);
    assert_eq!(frame.encode().unwrap().as_slice(), bytes);
    let [x, z] = frame.direction();
    assert!((x.hypot(z) - 1.).abs() < 1e-6 && x < 0. && z > 0.);
    assert_eq!(
        frame.edges(Button::LeftHand).collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(
        frame.edges(Button::Headbutt).collect::<Vec<_>>(),
        [false, true]
    );
    assert_eq!(frame.edges(Button::Run).collect::<Vec<_>>(), [false]);
    assert_eq!(frame.edges(Button::Jump).collect::<Vec<_>>(), [true]);
    assert_eq!(frame.edges(Button::Interact).count(), 0);
}

#[test]
fn malformed_lengths_kinds_versions_axes_and_masks_are_rejected() {
    let valid = InputFrame::default().encode().unwrap();
    for len in 0..18 {
        assert_eq!(InputFrame::decode(&valid[..len]), Err(InputError::Length));
    }
    assert_eq!(InputFrame::decode(&[0; 19]), Err(InputError::Length));
    for (offset, value, error) in [
        (0, 2, InputError::Kind),
        (1, 0, InputError::Version),
        (10, 128, InputError::Axis),
        (11, 128, InputError::Axis),
        (13, 4, InputError::Buttons),
        (15, 128, InputError::Buttons),
        (17, 4, InputError::Buttons),
        (14, 1, InputError::Edges),
    ] {
        let mut record = valid;
        record[offset] = value;
        assert_eq!(InputFrame::decode(&record), Err(error));
    }
    assert!(
        InputFrame {
            axes: [-128, 0],
            ..Default::default()
        }
        .encode()
        .is_err()
    );
}

#[test]
fn sequence_wrap_uses_the_half_range_and_axes_round_trip() {
    assert!(is_newer(0, u32::MAX));
    assert!(!is_newer(u32::MAX, 0) && !is_newer(3, 3) && !is_newer(1 << 31, 0));
    for x in -127..=127 {
        for z in -127..=127 {
            let f = InputFrame {
                axes: [x, z],
                sequence: u32::MAX,
                ..Default::default()
            };
            assert_eq!(InputFrame::decode(&f.encode().unwrap()).unwrap(), f);
            let [a, b] = f.direction();
            assert!(a.hypot(b) <= 1.000001);
        }
    }
}
