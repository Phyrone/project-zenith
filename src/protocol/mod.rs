use std::any::Any;

use enumset::EnumSetType;

pub mod common;
pub mod packets;
pub mod channel;
pub mod datagram;
pub mod error;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpenIntent {
    /// A single message is send over the channel. EOF is used to determine the end of the message.
    /// Comparable to a HTTP Get/Post/Put.
    Single = 0,

    /// A sequence of messages is send over the channel. Using LEN wireformat to determine the length of each message.
    /// EOF is used to determine the end of the sequence.
    /// Comparable to websockets.
    Sequence = 1,

    /// Send a header message followed by a stream of data.
    /// LEN wireformat is used to determine the length of the header message.
    /// EOF is used to determine the end of the stream of data.
    /// Comparable TCP (with a header).
    Stream = 2,

    /// Send an error message. EOF is used to determine the end of the message.
    Error = 0x0F,
}

impl TryFrom<u8> for OpenIntent {
    type Error = InvalidChannelTypeError;

    fn try_from(value: u8) -> Result<Self, InvalidChannelTypeError> {
        match value {
            0 => Ok(OpenIntent::Single),
            1 => Ok(OpenIntent::Sequence),
            2 => Ok(OpenIntent::Stream),
            0x0F /* 16 */ => Ok(OpenIntent::Error),
            _ => Err(InvalidChannelTypeError::new(value)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct InvalidChannelTypeError(u8);

impl InvalidChannelTypeError {
    pub fn new(value: u8) -> Self {
        InvalidChannelTypeError(value)
    }
}

impl std::fmt::Display for InvalidChannelTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid channel type: {}", self.0)
    }
}

impl std::error::Error for InvalidChannelTypeError {}


