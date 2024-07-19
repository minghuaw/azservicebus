//! Defines the [`MessageState`] enum.

// Conditional import for docs.rs
#[cfg(docsrs)]
use crate::ReceivedMessage;

/// Represents the message state of the [`ReceivedMessage`]
#[derive(Debug)]
pub enum MessageState {
    /// Specifies an active message state.
    Active = 0,

    /// Specifies a deferred message state.
    Deferred = 1,

    /// Specifies the scheduled message state.
    Scheduled = 2,
}

// azservicebus.message.go #L399
impl Default for MessageState {
    fn default() -> Self {
        Self::Active
    }
}

impl From<i64> for MessageState {
    fn from(value: i64) -> Self {
        match value {
            1 => MessageState::Deferred,
            2 => MessageState::Scheduled,
            _ => MessageState::Active,
        }
    }
}

impl From<MessageState> for i64 {
    fn from(value: MessageState) -> Self {
        value as i64
    }
}
