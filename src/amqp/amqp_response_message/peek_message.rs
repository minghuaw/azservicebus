use fe2o3_amqp_management::error::{Error as ManagementError, InvalidType};
use fe2o3_amqp_management::{response::Response, status::StatusCode};
use fe2o3_amqp_types::messaging::message::__private::Deserializable;
use fe2o3_amqp_types::messaging::{Body, Message};
use fe2o3_amqp_types::primitives::{Binary, OrderedMap};
use serde_amqp::Value;

use crate::amqp::management_constants::properties::{MESSAGE, MESSAGES};
use crate::primitives::service_bus_peeked_message::ServiceBusPeekedMessage;

use super::{HTTP_STATUS_CODE_NO_CONTENT, HTTP_STATUS_CODE_OK};

pub(super) type EncodedMessage = Binary;
pub(super) type EncodedMessages = Vec<OrderedMap<String, EncodedMessage>>;
pub(super) type PeekMessageResponseBody = OrderedMap<String, EncodedMessages>;

#[derive(Debug)]
pub(crate) struct PeekMessageResponse {
    pub messages: Vec<Vec<u8>>,
}

pub(crate) fn get_messages_from_body(
    mut body: PeekMessageResponseBody,
) -> Option<impl Iterator<Item = Vec<u8>>> {
    let messages = body.swap_remove(MESSAGES)?;

    let messages = messages
        .into_iter()
        .filter_map(|mut map| map.swap_remove(MESSAGE).map(|arr| arr.into_vec()));

    Some(messages)
}

impl PeekMessageResponse {
    pub fn into_peeked_messages(self) -> Result<Vec<ServiceBusPeekedMessage>, serde_amqp::Error> {
        self.messages
            .into_iter()
            .map(|buf| {
                let raw_amqp_message: Deserializable<Message<Body<Value>>> =
                    serde_amqp::from_slice(&buf)?;
                let message = ServiceBusPeekedMessage {
                    raw_amqp_message: raw_amqp_message.0,
                };
                Ok(message)
            })
            .collect()
    }
}

impl Response for PeekMessageResponse {
    // There are more than one Ok status code. So we are ignoring the status code here.
    const STATUS_CODE: u16 = super::HTTP_STATUS_CODE_OK;

    type Body = Option<PeekMessageResponseBody>;

    type Error = ManagementError;

    fn verify_status_code(
        message: &mut fe2o3_amqp_types::messaging::Message<Self::Body>,
    ) -> Result<StatusCode, Self::Error> {
        super::verify_ok_or_no_content_status_code(message)
    }

    fn decode_message(
        message: fe2o3_amqp_types::messaging::Message<Self::Body>,
    ) -> Result<Self, Self::Error> {
        let body = message.body.ok_or(ManagementError::DecodeError(None))?;
        let messages = get_messages_from_body(body)
            .ok_or_else(|| InvalidType {
                expected: MESSAGES.to_string(),
                actual: "None".to_string(),
            })?
            .collect();

        Ok(Self { messages })
    }

    fn from_message(
        mut message: fe2o3_amqp_types::messaging::Message<Self::Body>,
    ) -> Result<Self, Self::Error> {
        let status_code = Self::verify_status_code(&mut message)?;

        match status_code.0.get() {
            HTTP_STATUS_CODE_OK => Self::decode_message(message),
            HTTP_STATUS_CODE_NO_CONTENT => Ok(Self { messages: vec![] }),
            _ => unreachable!(),
        }
    }
}
