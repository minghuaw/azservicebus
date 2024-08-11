//! Defines the [`ServiceBusMessageState`] enum.

use serde_amqp::Value;

// Conditional import for docs.rs
#[cfg(docsrs)]
use crate::ServiceBusReceivedMessage;

/// Represents the message state of the [`ServiceBusReceivedMessage`]
#[derive(Debug)]
pub enum ServiceBusMessageState {
    /// Specifies an active message state.
    Active = 0,

    /// Specifies a deferred message state.
    Deferred = 1,

    /// Specifies the scheduled message state.
    Scheduled = 2,
}

// azservicebus.message.go #L399
impl Default for ServiceBusMessageState {
    fn default() -> Self {
        Self::Active
    }
}

macro_rules! impl_try_from_num_for_sb_messagestate {
    ($num_ty:ty) => {
        impl TryFrom<$num_ty> for ServiceBusMessageState {
            type Error = $num_ty;

            fn try_from(value: $num_ty) -> Result<Self, Self::Error> {
                match value {
                    0 => Ok(Self::Active),
                    1 => Ok(Self::Deferred),
                    2 => Ok(Self::Scheduled),
                    _ => Err(value),
                }
            }
        }

        impl From<ServiceBusMessageState> for $num_ty {
            fn from(value: ServiceBusMessageState) -> Self {
                value as $num_ty
            }
        }
    };
}

impl_try_from_num_for_sb_messagestate!(u8);
impl_try_from_num_for_sb_messagestate!(i8);
impl_try_from_num_for_sb_messagestate!(u16);
impl_try_from_num_for_sb_messagestate!(i16);
impl_try_from_num_for_sb_messagestate!(u32);
impl_try_from_num_for_sb_messagestate!(i32);
impl_try_from_num_for_sb_messagestate!(u64);
impl_try_from_num_for_sb_messagestate!(i64);

impl TryFrom<Value> for ServiceBusMessageState {
    type Error = Value;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Ubyte(val) => Self::try_from(val).map_err(Value::from),
            Value::Ushort(val) => Self::try_from(val).map_err(Value::from),
            Value::Uint(val) => Self::try_from(val).map_err(Value::from),
            Value::Ulong(val) => Self::try_from(val).map_err(Value::from),
            Value::Byte(val) => Self::try_from(val).map_err(Value::from),
            Value::Short(val) => Self::try_from(val).map_err(Value::from),
            Value::Int(val) => Self::try_from(val).map_err(Value::from),
            Value::Long(val) => Self::try_from(val).map_err(Value::from),
            _ => Err(value),
        }
    }
}

impl<'a> TryFrom<&'a Value> for ServiceBusMessageState {
    type Error = &'a Value;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Ubyte(val) => Self::try_from(*val).map_err(|_| value),
            Value::Ushort(val) => Self::try_from(*val).map_err(|_| value),
            Value::Uint(val) => Self::try_from(*val).map_err(|_| value),
            Value::Ulong(val) => Self::try_from(*val).map_err(|_| value),
            Value::Byte(val) => Self::try_from(*val).map_err(|_| value),
            Value::Short(val) => Self::try_from(*val).map_err(|_| value),
            Value::Int(val) => Self::try_from(*val).map_err(|_| value),
            Value::Long(val) => Self::try_from(*val).map_err(|_| value),
            _ => Err(value),
        }
    }
}
