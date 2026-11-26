//! Version-one controls for an authenticated lobby socket.
use crate::VERSION;

#[derive(Debug, PartialEq, Eq)]
pub enum Command<'a> {
    Connect(&'a [u8; 6]),
    Resume(&'a [u8; 6], u64),
    SetReady(bool),
    StartMatch,
    MapLoaded(&'a [u8; 128]),
}

impl<'a> Command<'a> {
    pub fn decode(bytes: &'a [u8]) -> Option<Self> {
        match bytes {
            [16, VERSION, code @ ..] => Some(Self::Connect(code.try_into().ok()?)),
            [32, VERSION, rest @ ..] if rest.len() == 14 => {
                let (code, ticket) = rest.split_at(6);
                Some(Self::Resume(
                    code.try_into().ok()?,
                    u64::from_le_bytes(ticket.try_into().ok()?),
                ))
            }
            [22, VERSION] => Some(Self::StartMatch),
            [20, VERSION, hashes @ ..] => Some(Self::MapLoaded(hashes.try_into().ok()?)),
            [18, VERSION, value @ 0..=1] => Some(Self::SetReady(*value == 1)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LobbyState {
    pub revision: u32,
    pub occupied: u8,
    pub ready: u8,
    pub host: u8,
}

impl LobbyState {
    pub fn encode_header(&self, recipient: Option<u8>) -> Vec<u8> {
        let mut record = Vec::with_capacity(10);
        match recipient {
            Some(slot) => record.extend([17, VERSION, slot]),
            None => record.extend([19, VERSION]),
        }
        record.extend([self.occupied, self.ready, self.host]);
        record.extend(self.revision.to_le_bytes());
        record
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_have_exact_lengths_and_unambiguous_seat_masks() {
        assert_eq!(
            Command::decode(b"\x10\x01AB2C3D"),
            Some(Command::Connect(b"AB2C3D"))
        );
        let mut resume = b"\x20\x01AB2C3D".to_vec();
        resume.extend(0x123456789abcdef0_u64.to_le_bytes());
        assert_eq!(
            Command::decode(&resume),
            Some(Command::Resume(b"AB2C3D", 0x123456789abcdef0))
        );
        assert_eq!(Command::decode(&resume[..15]), None);
        resume.push(0);
        assert_eq!(Command::decode(&resume), None);
        for (value, expected) in [(0, false), (1, true)] {
            assert_eq!(
                Command::decode(&[18, 1, value]),
                Some(Command::SetReady(expected))
            );
        }
        for bad in [
            &[18, 1, 2][..],
            &[18, 2, 1],
            &[18, 1, 1, 7],
            &[16, 1],
            &[17, 1, 0],
        ] {
            assert_eq!(Command::decode(bad), None);
        }
        let state = LobbyState {
            occupied: 129,
            ready: 128,
            host: 0,
            revision: 0x12345678,
        };
        assert_eq!(
            state.encode_header(Some(7)),
            [17, 1, 7, 129, 128, 0, 120, 86, 52, 18]
        );
        assert_eq!(
            state.encode_header(None),
            [19, 1, 129, 128, 0, 120, 86, 52, 18]
        );
    }
}
