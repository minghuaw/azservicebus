use azure_core::auth::AccessToken;
use std::{fmt::Display, sync::Arc};

use crate::{
    authorization::service_bus_token_credential::ServiceBusTokenCredential,
    constants::{JSON_WEB_TOKEN_TYPE, SAS_TOKEN_TYPE},
};

#[derive(Debug)]
pub(crate) enum TokenType {
    /// The type to consider a token if it is based on an Service Bus shared access signature.
    SharedAccessToken {
        credential: Arc<ServiceBusTokenCredential>,
    },
    /// The type to consider a token if not based on a shared access signature.
    JsonWebToken {
        credential: Arc<ServiceBusTokenCredential>,

        /// The JWT-based token that is currently cached for authorization.
        cached_token: Option<AccessToken>,
    },
}

impl TokenType {
    pub(crate) fn entity_type(&self) -> &str {
        match self {
            TokenType::SharedAccessToken { .. } => SAS_TOKEN_TYPE,
            TokenType::JsonWebToken { .. } => JSON_WEB_TOKEN_TYPE,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::SharedAccessToken { .. } => write!(f, "{}", SAS_TOKEN_TYPE),
            TokenType::JsonWebToken { .. } => write!(f, "{}", JSON_WEB_TOKEN_TYPE),
        }
    }
}
