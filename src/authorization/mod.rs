//! Authorization primitives.

mod azure_named_key_credential;
mod azure_sas_credential;
pub(crate) mod service_bus_claim;
pub(crate) mod service_bus_token_credential;
pub(crate) mod shared_access_credential;
pub(crate) mod shared_access_signature;

pub use azure_named_key_credential::AzureNamedKeyCredential;
pub use azure_sas_credential::AzureSasCredential;
pub use service_bus_token_credential::ServiceBusTokenCredential;
pub use shared_access_credential::SharedAccessCredential;

cfg_not_wasm32! {
    #[cfg(test)]
    pub(crate) mod tests {
        use azure_core::credentials::AccessToken;
        use azure_core::error::Result;

        use std::pin::Pin;
        use std::future::Future;

        use mockall::mock;

        mock! {
            #[derive(Debug)]
            pub TokenCredential {}

            impl azure_core::credentials::TokenCredential for TokenCredential {
                // Required methods
                fn get_token<'life0, 'life1, 'life2, 'async_trait>(
                    &'life0 self,
                    scopes: &'life1 [&'life2 str],
                    options: Option<azure_core::credentials::TokenRequestOptions>
                ) -> Pin<Box<dyn Future<Output = Result<AccessToken>> + Send + 'async_trait>>
                where Self: 'async_trait,
                        'life0: 'async_trait,
                        'life1: 'async_trait,
                        'life2: 'async_trait;
            }
        }
    }
}
