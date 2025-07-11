use std::{marker::PhantomData, sync::Arc, time::Duration};


use azure_core::http::Url;
use tokio::sync::Mutex;

use crate::{
    authorization::service_bus_token_credential::ServiceBusTokenCredential,
    constants::DEFAULT_LAST_PEEKED_SEQUENCE_NUMBER,
    core::{TransportClient, TransportConnectionScope},
    primitives::{
        service_bus_retry_options::ServiceBusRetryOptions,
        service_bus_retry_policy::ServiceBusRetryPolicyExt,
        service_bus_transport_type::ServiceBusTransportType,
    },
    receiver::service_bus_receive_mode::ServiceBusReceiveMode,
    sealed::Sealed,
    ServiceBusRetryPolicy,
};

use super::{
    amqp_connection_scope::{AmqpConnectionScope, ReceiverType},
    amqp_receiver::AmqpReceiver,
    amqp_rule_manager::AmqpRuleManager,
    amqp_sender::AmqpSender,
    amqp_session_receiver::AmqpSessionReceiver,
    error::{AmqpClientError, OpenReceiverError, OpenRuleManagerError, OpenSenderError},
};

/// A transport client abstraction responsible for brokering operations for AMQP-based connections.
///
/// See also [`TransportClient`]
#[derive(Debug)]
pub struct AmqpClient<RP> {
    /// The endpoint for the Service Bus service to which the scope is associated.
    service_endpoint: Arc<Url>,

    /// The AMQP connection scope responsible for managing transport constructs for this instance.
    ///
    /// There isn't much read operations on this so it's fine to use a Mutex.
    connection_scope: Arc<Mutex<AmqpConnectionScope>>,

    /// Keep a copy of the transport type to avoid having to lock the connection_scope
    transport_type: ServiceBusTransportType,

    /// Retry policy phantom
    retry_policy: PhantomData<RP>,

    /// Keep track of whether the client has been disposed.
    ///
    /// this is duplicated as the connection scope also keeps track of that. The reason is that
    /// the connection scope is shared between the client and the receivers/senders. The client
    /// shouldn't have to .await for a lock on the connection scope to check if it's been disposed.
    is_connection_scope_disposed: bool,
}

impl<RP> AmqpClient<RP> {
    async fn create_transport_client_inner(
        service_endpoint: Url,
        connection_endpoint: Url,
        credential: ServiceBusTokenCredential,
        transport_type: ServiceBusTransportType,
        retry_timeout: Duration,
    ) -> Result<Self, AmqpClientError> {
        // Create AmqpConnectionScope
        let fut = AmqpConnectionScope::new(
            &service_endpoint,
            connection_endpoint,
            credential,
            transport_type,
        );
        let connection_scope = crate::util::time::timeout(retry_timeout, fut).await??;
    
        Ok(Self {
            service_endpoint: Arc::new(service_endpoint),
            connection_scope: Arc::new(Mutex::new(connection_scope)),
            transport_type,
            retry_policy: PhantomData,
            is_connection_scope_disposed: false,
        })
    }
}

impl<RP> Sealed for AmqpClient<RP> {}

impl<RP> TransportClient for AmqpClient<RP>
where
    RP: ServiceBusRetryPolicyExt + Send + Sync + 'static,
{
    type CreateClientError = AmqpClientError;
    type CreateSenderError = OpenSenderError;
    type CreateReceiverError = OpenReceiverError;
    type CreateRuleManagerError = OpenRuleManagerError;
    type DisposeError = AmqpClientError;

    type Sender = AmqpSender;
    type Receiver = AmqpReceiver;
    type SessionReceiver = AmqpSessionReceiver;
    type RuleManager = AmqpRuleManager;

    async fn create_transport_client(
        host: &str,
        credential: ServiceBusTokenCredential,
        transport_type: ServiceBusTransportType,
        custom_endpoint: Option<Url>,
        retry_timeout: Duration,
    ) -> Result<Self, Self::CreateClientError> {
        // Scheme of service endpoint must always be either "amqp" or "amqps"
        let service_endpoint = format_service_endpoint(host)?;

        let connection_endpoint = format_connection_endpoint(host, transport_type, custom_endpoint, &service_endpoint)?;

        Self::create_transport_client_inner(
            service_endpoint,
            connection_endpoint,
            credential,
            transport_type,
            retry_timeout,
        ).await
    }

    cfg_unsecured! {
        async fn create_unsecured_transport_client(
            host: &str,
            credential: ServiceBusTokenCredential,
            transport_type: ServiceBusTransportType,
            custom_endpoint: Option<Url>,
            retry_timeout: Duration,
        ) -> Result<Self, Self::CreateClientError> {
            // Scheme of service endpoint must always be either "amqp" or "amqps"
            let service_endpoint = format_unsecured_service_endpoint(host)?;

            let connection_endpoint = format_unsecured_connection_endpoint(host, transport_type, custom_endpoint, &service_endpoint)?;

            Self::create_transport_client_inner(
                service_endpoint,
                connection_endpoint,
                credential,
                transport_type,
                retry_timeout,
            ).await
        }
    }

    fn transport_type(&self) -> ServiceBusTransportType {
        // `transport_type` is a simple enum, cloning should be cheap
        self.transport_type
    }

    fn is_closed(&self) -> bool {
        self.is_connection_scope_disposed
    }

    async fn create_sender(
        &mut self,
        entity_path: String,
        identifier: String,
        retry_options: ServiceBusRetryOptions,
    ) -> Result<Self::Sender, Self::CreateSenderError> {
        let mut connection_scope = self.connection_scope.lock().await;

        let (link_identifier, sender, cbs_command_sender) = connection_scope
            .open_sender_link(&self.service_endpoint, &entity_path, &identifier)
            .await?;
        let management_link = connection_scope
            .open_management_link(&self.service_endpoint, &entity_path, &identifier)
            .await?;
        let retry_policy = RP::from(retry_options);
        Ok(AmqpSender {
            id: link_identifier,
            service_endpoint: self.service_endpoint.clone(),
            entity_path,
            identifier_str: identifier,
            retry_policy: Box::new(retry_policy) as Box<dyn ServiceBusRetryPolicy>,
            sender,
            management_link,
            cbs_command_sender,
            connection_scope: self.connection_scope.clone(),
        })
    }

    async fn create_receiver(
        &mut self,
        entity_path: String,
        identifier: String,
        retry_options: ServiceBusRetryOptions,
        receive_mode: ServiceBusReceiveMode,
        prefetch_count: u32,
    ) -> Result<Self::Receiver, Self::CreateReceiverError> {
        let mut connection_scope = self.connection_scope.lock().await;

        let (link_identifier, receiver, cbs_command_sender) = connection_scope
            .open_receiver_link(
                &self.service_endpoint,
                &entity_path,
                &identifier,
                &receive_mode,
                ReceiverType::NonSession,
                prefetch_count,
            )
            .await?;
        let management_link = connection_scope
            .open_management_link(&self.service_endpoint, &entity_path, &identifier)
            .await?;
        let retry_policy = RP::from(retry_options);
        Ok(AmqpReceiver {
            id: link_identifier,
            service_endpoint: self.service_endpoint.clone(),
            entity_path,
            identifier_str: identifier,
            retry_policy: Box::new(retry_policy) as Box<dyn ServiceBusRetryPolicy>,
            receiver,
            receive_mode,
            _is_processor: false,
            prefetch_count,
            management_link,
            request_response_locked_messages: Default::default(),
            last_peeked_sequence_number: DEFAULT_LAST_PEEKED_SEQUENCE_NUMBER,
            cbs_command_sender,
            connection_scope: self.connection_scope.clone(),
        })
    }

    async fn create_session_receiver(
        &mut self,
        entity_path: String,
        identifier: String,
        retry_options: ServiceBusRetryOptions,
        receive_mode: ServiceBusReceiveMode,
        session_id: Option<String>,
        prefetch_count: u32,
    ) -> Result<Self::SessionReceiver, Self::CreateReceiverError> {
        let mut connection_scope = self.connection_scope.lock().await;
        let (link_identifier, receiver, cbs_command_sender) = connection_scope
            .open_receiver_link(
                &self.service_endpoint,
                &entity_path,
                &identifier,
                &receive_mode,
                ReceiverType::Session { session_id },
                prefetch_count,
            )
            .await?;
        let management_link = connection_scope
            .open_management_link(&self.service_endpoint, &entity_path, &identifier)
            .await?;
        let retry_policy = RP::from(retry_options);
        let inner = AmqpReceiver {
            id: link_identifier,
            service_endpoint: self.service_endpoint.clone(),
            entity_path,
            identifier_str: identifier,
            retry_policy: Box::new(retry_policy) as Box<dyn ServiceBusRetryPolicy>,
            receiver,
            receive_mode,
            _is_processor: false,
            prefetch_count,
            management_link,
            request_response_locked_messages: Default::default(),
            last_peeked_sequence_number: DEFAULT_LAST_PEEKED_SEQUENCE_NUMBER,
            cbs_command_sender,
            connection_scope: self.connection_scope.clone(),
        };

        Ok(AmqpSessionReceiver { inner })
    }

    async fn create_rule_manager(
        &mut self,
        subscription_path: String,
        identifier: String,
        retry_options: ServiceBusRetryOptions,
    ) -> Result<Self::RuleManager, Self::CreateRuleManagerError> {
        let mut connection_scope = self.connection_scope.lock().await;
        let retry_policy = RP::from(retry_options);
        let management_link = connection_scope
            .open_management_link(&self.service_endpoint, &subscription_path, &identifier)
            .await?;
        Ok(AmqpRuleManager {
            identifier_str: identifier,
            service_endpoint: self.service_endpoint.clone(),
            subscription_path,
            retry_policy: Box::new(retry_policy) as Box<dyn ServiceBusRetryPolicy>,
            management_link,
            connection_scope: self.connection_scope.clone(),
        })
    }

    async fn close(&mut self) -> Result<(), Self::DisposeError> {
        if self.is_connection_scope_disposed {
            Ok(())
        } else {
            self.is_connection_scope_disposed = true;
            self.connection_scope
                .lock()
                .await
                .dispose()
                .await
                .map_err(Into::into)
        }
    }
}

fn format_service_endpoint(host: &str) -> Result<Url, url::ParseError> {
    let addr = format!("{}://{}", ServiceBusTransportType::AMQP_SCHEME, host);
    Url::parse(&addr)
}

cfg_unsecured! {
    fn format_unsecured_service_endpoint(host: &str) -> Result<Url, url::ParseError> {
        let addr = format!("{}://{}", ServiceBusTransportType::UNSECURED_AMQP_SCHEME, host);
        Url::parse(&addr)
    }
}

macro_rules! format_connection_endpoint_impl {
    ($host:ident, $transport_type:ident, $custom_endpoint:ident, $service_endpoint:ident, $url_scheme_fn:ident) => {
        match $custom_endpoint.as_ref().and_then(|url| url.host_str()) {
            Some(custom_host) => match $transport_type {
                #[cfg(not(target_arch = "wasm32"))]
                ServiceBusTransportType::AmqpTcp => {
                    let addr = format!("{}://{}", $transport_type.$url_scheme_fn(), custom_host);
                    Url::parse(&addr)
                }
                ServiceBusTransportType::AmqpWebSocket => {
                    let addr = format!(
                        "{}://{}{}",
                        $transport_type.$url_scheme_fn(),
                        custom_host,
                        AmqpConnectionScope::WEB_SOCKETS_PATH_SUFFIX
                    );
                    Url::parse(&addr)
                }
            },
            None => match $transport_type {
                #[cfg(not(target_arch = "wasm32"))]
                ServiceBusTransportType::AmqpTcp => Ok($service_endpoint.clone()),
                ServiceBusTransportType::AmqpWebSocket => {
                    let addr = format!(
                        "{}://{}{}",
                        $transport_type.$url_scheme_fn(),
                        $host,
                        AmqpConnectionScope::WEB_SOCKETS_PATH_SUFFIX
                    );
                    Url::parse(&addr)
                }
            },
        }
    };
}

#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
fn format_connection_endpoint(
    host: &str,
    transport_type: ServiceBusTransportType,
    custom_endpoint: Option<Url>,
    service_endpoint: &Url,
) -> Result<Url, url::ParseError> {
    format_connection_endpoint_impl!(host, transport_type, custom_endpoint, service_endpoint, url_scheme)
}

cfg_unsecured! {
    fn format_unsecured_connection_endpoint(
        host: &str,
        transport_type: ServiceBusTransportType,
        custom_endpoint: Option<Url>,
        service_endpoint: &Url,
    ) -> Result<Url, url::ParseError> {
        format_connection_endpoint_impl!(host, transport_type, custom_endpoint, service_endpoint, unsecured_url_scheme)
    }
}

#[cfg(test)]
mod tests {}
