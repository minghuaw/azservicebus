use std::sync::Arc;

use azure_core::credentials::{AccessToken, TokenCredential};

use crate::constants::DEFAULT_SCOPE;

use super::shared_access_credential::SharedAccessCredential;

/// Token-based credential for Service Bus.
///
/// This supports `SharedAccessCredential` and other credential types that
/// implement `TokenCredential` (eg. `azure_identity::DefaultAzureCredential`).
///
/// # Example
///
/// ```rust,no_run
/// use azure_identity::DefaultAzureCredential;
/// use azservicebus::authorization::ServiceBusTokenCredential;
///
/// let default_credential = DefaultAzureCredential::new().unwrap();
/// let credential = ServiceBusTokenCredential::from(default_credential);
/// ```
pub enum ServiceBusTokenCredential {
    /// Shared Access Signature credential.
    ///
    /// FIXME: This is a temporary workaround until specialization is stablized.
    SharedAccessCredential(SharedAccessCredential),

    /// Other credential types.
    ///
    /// TODO: Is the use of trait object here justified?
    Other(Arc<dyn TokenCredential>),
}

impl std::fmt::Debug for ServiceBusTokenCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SharedAccessCredential(_arg0) => f.debug_tuple("SharedAccessCredential").finish(),
            Self::Other(_arg0) => f.debug_tuple("Other").finish(),
        }
    }
}

impl From<SharedAccessCredential> for ServiceBusTokenCredential {
    fn from(source: SharedAccessCredential) -> Self {
        Self::SharedAccessCredential(source)
    }
}

impl<TC> From<Arc<TC>> for ServiceBusTokenCredential
where
    TC: TokenCredential + 'static,
{
    fn from(source: Arc<TC>) -> Self {
        Self::Other(source)
    }
}

impl ServiceBusTokenCredential {
    /// Creates a new `ServiceBusTokenCredential` from the given credential. This is an alias for
    /// `From::from`.
    pub fn new(source: Arc<dyn TokenCredential>) -> Self {
        Self::Other(source)
    }

    /// Indicates whether the credential is based on an Service Bus
    /// shared access policy.
    pub fn is_shared_access_credential(&self) -> bool {
        matches!(self, ServiceBusTokenCredential::SharedAccessCredential(_))
    }
}

impl ServiceBusTokenCredential {
    /// Gets a `AccessToken` for the specified resource
    pub(crate) async fn get_token(&self, scopes: &[&str]) -> azure_core::Result<AccessToken> {
        match self {
            ServiceBusTokenCredential::SharedAccessCredential(credential) => {
                credential.get_token(scopes).await
            }
            ServiceBusTokenCredential::Other(credential) => credential.get_token(scopes, None).await,
        }
    }

    pub(crate) async fn get_token_using_default_resource(&self) -> azure_core::Result<AccessToken> {
        self.get_token(&[DEFAULT_SCOPE]).await
    }
}

cfg_not_wasm32! {
    #[cfg(test)]
    mod tests {
        use azure_core::credentials::Secret;
        use time::macros::datetime;
        use std::sync::Arc;

        use crate::authorization::{
            shared_access_credential::SharedAccessCredential,
            shared_access_signature::SharedAccessSignature, tests::MockTokenCredential,
        };

        use super::ServiceBusTokenCredential;

        #[tokio::test]
        async fn get_token_delegates_to_the_source_credential() {
            let token_value = "token";
            let mut mock_credentials = MockTokenCredential::new();
            let scope = "the scope value";
            let token_response = azure_core::credentials::AccessToken {
                token: Secret::new(token_value),
                expires_on: datetime!(2015-10-27 00:00:00).assume_utc(),
            };
            mock_credentials
                .expect_get_token()
                .times(1)
                .returning(move |_resource, _options| {
                    let token_response_clone = token_response.clone();
                    Box::pin( async { Ok(token_response_clone) } )
                });

            let credential = ServiceBusTokenCredential::from(Arc::new(mock_credentials));
            let token_result = credential.get_token(&[scope]).await;
            assert_eq!(token_result.unwrap().token.secret(), token_value);
        }

        #[tokio::test]
        async fn is_shared_access_credential_recognized_as_sas_credentials() {
            let signature = SharedAccessSignature::try_from_parts(
                "sb-name",
                "keyName",
                "key",
                Some(std::time::Duration::from_secs(4 * 60 * 60)),
            )
            .unwrap();
            let sas_credential = SharedAccessCredential::from(signature);
            let credential = ServiceBusTokenCredential::from(sas_credential);
            assert!(credential.is_shared_access_credential());
        }
    }
}
