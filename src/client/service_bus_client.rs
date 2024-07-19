//! Client and client configuration options for Azure Service Bus.

use std::{borrow::Cow, marker::PhantomData};

use azure_core::{auth::TokenCredential, Url};

use crate::{
    amqp::{
        amqp_client::AmqpClient,
        error::OpenReceiverError,
    },
    authorization::{
        service_bus_token_credential::ServiceBusTokenCredential,
        shared_access_credential::SharedAccessCredential, AzureNamedKeyCredential,
        AzureSasCredential,
    },
    core::{BasicRetryPolicy, TransportSessionReceiver},
    diagnostics,
    entity_name_formatter::{self, format_entity_path},
    primitives::{
        service_bus_connection::{build_connection_resource, ServiceBusConnection},
        service_bus_retry_options::RetryOptions,
        service_bus_retry_policy::ServiceBusRetryPolicyExt,
        service_bus_transport_type::TransportType,
    },
    receiver::service_bus_session_receiver::{
        SessionReceiver, SessionReceiverOptions,
    },
    Receiver, ReceiverOptions, RuleManager, Sender,
    SenderOptions,
};

use super::error::AcceptNextSessionError;

/// The set of options that can be specified when creating an [`Client`]
/// to configure its behavior.
#[derive(Debug, Clone, Default)]
pub struct ClientOptions {
    /// The type of protocol and transport that will be used for communicating with the Service
    /// Bus service.
    pub transport_type: TransportType,

    /// A property used to set the [`Client`] ID to identify the client. This can be used
    /// to correlate logs and exceptions. If `None` or empty, a random unique value will be
    /// used.
    pub identifier: Option<String>,

    /// A custom endpoint address that can be used when establishing the connection to the Service
    /// Bus service.
    ///
    /// The custom endpoint address will be used in place of the default endpoint provided by the
    /// Service Bus namespace when establishing the connection. The connection string or fully
    /// qualified namespace will still be needed in order to validate the connection with the
    /// service.
    pub custom_endpoint_address: Option<Url>,

    /// The set of options to use for determining whether a failed operation should be retried and,
    /// if so, the amount of time to wait between retry attempts.  These options also control the
    /// amount of time allowed for receiving messages and other interactions with the Service Bus
    /// service.
    pub retry_options: RetryOptions,

    /// Gets or sets a flag that indicates whether or not transactions may span multiple
    /// Service Bus entities.
    pub enable_cross_entity_transactions: bool,
}

/// Type state for [`Client`] indicating that the client is using a custom retry policy.
#[derive(Debug)]
pub struct WithCustomRetryPolicy<RP> {
    retry_policy: PhantomData<RP>,
}

impl<RP> WithCustomRetryPolicy<RP>
where
    RP: ServiceBusRetryPolicyExt + Send + Sync + 'static,
{
    /// Creates a new instance of the [`Client`] class using the specified
    /// connection string and [`ClientOptions`].
    pub async fn new_from_connection_string<'a>(
        self,
        connection_string: impl Into<Cow<'a, str>>,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        let connection_string = connection_string.into();
        let identifier = options.identifier.clone();
        let connection = ServiceBusConnection::new(connection_string, options).await?;
        let identifier = identifier.unwrap_or_else(|| {
            diagnostics::utilities::generate_identifier(connection.fully_qualified_namespace())
        });
        Ok(Client {
            identifier,
            connection,
        })
    }

    /// Creates a new instance of the [`Client`] class using the specified
    /// connection string and [`ClientOptions`].
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_connection_string` instead"
    )]
    pub async fn create_client<'a>(
        self,
        connection_string: impl Into<Cow<'a, str>>,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        self.new_from_connection_string(connection_string, options).await
    }

    /// Creates a new instance of the [`Client`] class using a named key credential.
    pub async fn new_from_named_key_credential(
        self,
        fully_qualified_namespace: impl Into<String>,
        credential: AzureNamedKeyCredential,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        let fully_qualified_namespace = fully_qualified_namespace.into();
        let signuture_resource = build_connection_resource(
            &options.transport_type,
            Some(&fully_qualified_namespace),
            None,
        )?;
        let shared_access_credential =
            SharedAccessCredential::try_from_named_key_credential(credential, signuture_resource)?;
        self.new_from_credential(fully_qualified_namespace, shared_access_credential, options).await
    }

    /// Creates a new instance of the [`Client`] class using a named key credential.
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_named_key_credential` instead"
    )]
    pub async fn create_client_with_named_key_credential(
        self,
        fully_qualified_namespace: impl Into<String>,
        credential: AzureNamedKeyCredential,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        self.new_from_named_key_credential(fully_qualified_namespace, credential, options).await
    }

    /// Creates a new instance of the [`Client`] class using a SAS token credential.
    pub async fn new_from_sas_credential(
        self,
        fully_qualified_namespace: impl Into<String>,
        credential: AzureSasCredential,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        let shared_access_credential = SharedAccessCredential::try_from_sas_credential(credential)?;
        self.new_from_credential(fully_qualified_namespace, shared_access_credential, options).await
    }

    /// Creates a new instance of the [`Client`] class using a SAS token credential.
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_sas_credential` instead"
    )]
    pub async fn create_client_with_sas_credential(
        self,
        fully_qualified_namespace: impl Into<String>,
        credential: AzureSasCredential,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        self.new_from_sas_credential(fully_qualified_namespace, credential, options).await
    }

    /// Creates a new instance of the [`Client`] class using a token credential.
    pub async fn new_from_token_credential(
        self,
        fully_qualified_namespace: impl Into<String>,
        credential: impl TokenCredential + 'static,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        let credential = ServiceBusTokenCredential::new(credential);
        self.new_from_credential(fully_qualified_namespace, credential, options).await
    }

    /// Creates a new instance of the [`Client`] class using a token credential.
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_token_credential` instead"
    )]
    pub async fn create_client_with_token_credential(
        self,
        fully_qualified_namespace: impl Into<String>,
        credential: impl TokenCredential + 'static,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        self.new_from_token_credential(fully_qualified_namespace, credential, options).await
    }

    /// Creates a new instance of the [`Client`] using the specified
    /// namespace and credential
    pub async fn new_from_credential(
        self,
        fully_qualified_namespace: impl Into<String>,
        credential: impl Into<ServiceBusTokenCredential>,
        options: ClientOptions,
    ) -> Result<Client<RP>, azure_core::Error> {
        let fully_qualified_namespace = fully_qualified_namespace.into();
        let identifier = options.identifier.clone().unwrap_or_else(|| {
            diagnostics::utilities::generate_identifier(&fully_qualified_namespace)
        });
        let credential = credential.into();
        let connection = ServiceBusConnection::new_from_credential(
            fully_qualified_namespace,
            credential,
            options,
        )
        .await?;
        Ok(Client {
            identifier,
            connection,
        })
    }
}

/// The [`Client`] is the top-level client through which all Service Bus entities can be
/// interacted with. Any lower level types retrieved from here, such as [`Sender`] and
/// [`Receiver`] will share the same AMQP connection. Disposing the [`Client`]
/// will cause the AMQP connection to close.
///
/// # WebAssembly support
///
/// Creating a [`Client`] is supported for `wasm32-unknown-unknown` targets; however, the
/// user must ensure that the client is created within the scope of a `tokio::task::LocalSet`.
#[derive(Debug)]
pub struct Client<RP> {
    /// The name used to identify this [`Client`]
    identifier: String,

    /// The connection that is used for the client.
    connection: ServiceBusConnection<AmqpClient<RP>>,
}

impl Client<BasicRetryPolicy> {
    /// Use a custom retry policy for the client.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use azservicebus::{
    ///     Client, ClientOptions, RetryPolicy,
    /// };
    ///
    /// struct MyRetryPolicy;
    ///
    /// impl RetryPolicy for MyRetryPolicy {
    ///     // ...
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::with_custom_retry_policy::<MyRetryPolicy>()
    ///         .new_from_connection_string("<NAMESPACE-CONNECTION-STRING>", ClientOptions::default())
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    pub fn with_custom_retry_policy<RP>() -> WithCustomRetryPolicy<RP> {
        WithCustomRetryPolicy {
            retry_policy: PhantomData,
        }
    }

    /// Creates a new instance of the [`Client`] class using the specified connection
    /// string and [`ClientOptions`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use azservicebus::{
    ///     Client, ClientOptions,
    /// };
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut client = Client::new_from_connection_string(
    ///             "<NAMESPACE-CONNECTION-STRING>",
    ///             ClientOptions::default()
    ///         )
    ///         .await
    ///         .unwrap();
    ///     client.dispose().await.unwrap();
    /// }
    /// ```
    pub async fn new_from_connection_string<'a>(
        connection_string: impl Into<Cow<'a, str>>,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::with_custom_retry_policy()
            .new_from_connection_string(connection_string, options)
            .await
    }

    /// Creates a new instance of the [`Client`] class using the specified
    /// connection string and [`ClientOptions`].
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_connection_string` instead"
    )]
    pub async fn new<'a>(
        connection_string: impl Into<Cow<'a, str>>,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::with_custom_retry_policy()
            .new_from_connection_string(connection_string, options)
            .await
    }

    /// Creates a new instance of the [`Client`] class using a named key credential.
    pub async fn new_from_named_key_credential(
        fully_qualified_namespace: impl Into<String>,
        credential: AzureNamedKeyCredential,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::with_custom_retry_policy()
            .new_from_named_key_credential(fully_qualified_namespace, credential, options)
            .await
    }

    /// Creates a new instance of the [`Client`] class using a named key credential.
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_named_key_credential` instead"
    )]
    pub async fn new_with_named_key_credential(
        fully_qualified_namespace: impl Into<String>,
        credential: AzureNamedKeyCredential,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::new_from_named_key_credential(fully_qualified_namespace, credential, options).await
    }

    /// Creates a new instance of the [`Client`] class using a SAS token credential.
    pub async fn new_from_sas_credential(
        fully_qualified_namespace: impl Into<String>,
        credential: AzureSasCredential,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::with_custom_retry_policy()
            .new_from_sas_credential(fully_qualified_namespace, credential, options)
            .await
    }

    /// Creates a new instance of the [`Client`] class using a SAS token credential.
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_sas_credential` instead"
    )]
    pub async fn new_with_sas_credential(
        fully_qualified_namespace: impl Into<String>,
        credential: AzureSasCredential,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::new_from_sas_credential(fully_qualified_namespace, credential, options).await
    }

    /// Creates a new instance of the [`Client`] class using a token credential.
    pub async fn new_from_token_credential(
        fully_qualified_namespace: impl Into<String>,
        credential: impl TokenCredential + 'static,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::with_custom_retry_policy()
            .new_from_token_credential(fully_qualified_namespace, credential, options)
            .await
    }

    /// Creates a new instance of the [`Client`] class using a token credential.
    #[deprecated(
        since = "0.14.0",
        note = "Please use `new_from_token_credential` instead"
    )]
    pub async fn new_with_token_credential(
        fully_qualified_namespace: impl Into<String>,
        credential: impl TokenCredential + 'static,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::new_from_token_credential(fully_qualified_namespace, credential, options).await
    }

    /// Creates a new instance of the [`Client`] using the specified
    /// namespace and credential.
    pub async fn new_from_credential(
        fully_qualified_namespace: impl Into<String>,
        credential: impl Into<ServiceBusTokenCredential>,
        options: ClientOptions,
    ) -> Result<Self, azure_core::Error> {
        Self::with_custom_retry_policy()
            .new_from_credential(fully_qualified_namespace, credential, options)
            .await
    }
}

impl<RP> Client<RP>
where
    RP: ServiceBusRetryPolicyExt + 'static,
{
    /// The fully qualified Service Bus namespace that the connection is associated with. This is
    /// likely to be similar to `{yournamespace}.servicebus.windows.net`.
    pub fn fully_qualified_namespace(&self) -> &str {
        self.connection.fully_qualified_namespace()
    }

    /// The name used to identify this [`Client`].
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Indicates whether or not this [`Client`] has been closed.
    pub fn is_closed(&self) -> bool {
        self.connection.is_closed()
    }
}

/* -------------------------------------------------------------------------- */
/*                                   Dispose                                  */
/* -------------------------------------------------------------------------- */

impl<RP> Client<RP>
where
    RP: ServiceBusRetryPolicyExt + 'static,
{
    /// Performs the task needed to clean up resources used by the [`Client`],
    /// including ensuring that the client itself has been closed.
    pub async fn dispose(self) -> Result<(), azure_core::Error> {
        self.connection.dispose().await?;
        Ok(())
    }
}

/* -------------------------------------------------------------------------- */
/*                                Create Sender                               */
/* -------------------------------------------------------------------------- */

impl<RP> Client<RP>
where
    RP: ServiceBusRetryPolicyExt + 'static,
{
    /// Creates a new [`Sender`] which can be used to send messages to a specific queue or
    /// topic.
    ///
    /// # WebAssembly support
    ///
    /// When creating a sender for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn create_sender(
        &mut self,
        queue_or_topic_name: impl Into<String>,
        options: SenderOptions,
    ) -> Result<Sender, azure_core::Error> {
        let entity_path = queue_or_topic_name.into();
        let identifier = options
            .identifier
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| diagnostics::utilities::generate_identifier(&entity_path));
        let retry_options = self.connection.retry_options().clone();
        let inner = self
            .connection
            .create_transport_sender(entity_path, identifier, retry_options)
            .await?;

        Ok(Sender { inner })
    }
}

/* -------------------------------------------------------------------------- */
/*                               Create Receiver                              */
/* -------------------------------------------------------------------------- */

impl<RP> Client<RP>
where
    RP: ServiceBusRetryPolicyExt + 'static,
{
    /// The transport type used by the client.
    pub fn transport_type(&self) -> TransportType {
        self.connection.transport_type()
    }

    /// Creates a new [`Receiver`] which can be used to receive messages from a specific
    /// queue.
    ///
    /// # WebAssembly support
    ///
    /// When creating a receiver for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn create_receiver_for_queue(
        &mut self,
        queue_name: impl Into<String>,
        options: ReceiverOptions,
    ) -> Result<Receiver, azure_core::Error> {
        let entity_path = queue_name.into();
        self.create_receiver(entity_path, options).await
            .map_err(Into::into)
    }

    /// Creates a new [`Receiver`] which can be used to receive messages from a specific
    /// subscription.
    ///
    /// # WebAssembly support
    ///
    /// When creating a receiver for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn create_receiver_for_subscription(
        &mut self,
        topic_name: impl AsRef<str>,
        subscription_name: impl AsRef<str>,
        options: ReceiverOptions,
    ) -> Result<Receiver, azure_core::Error> {
        let entity_path = entity_name_formatter::format_subscription_path(
            topic_name.as_ref(),
            subscription_name.as_ref(),
        );
        self.create_receiver(entity_path, options).await
            .map_err(Into::into)
    }

    // This cannot be used to create a session receiver or proces
    async fn create_receiver(
        &mut self,
        entity_path: String,
        options: ReceiverOptions,
    ) -> Result<Receiver, OpenReceiverError> {
        let identifier = options
            .identifier
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| diagnostics::utilities::generate_identifier(&entity_path));
        let retry_options = self.connection.retry_options().clone();
        let receive_mode = options.receive_mode;
        let prefetch_count = options.prefetch_count;
        let entity_path = format_entity_path(entity_path, options.sub_queue);

        let inner = self
            .connection
            .create_transport_receiver(
                entity_path,
                identifier,
                retry_options,
                receive_mode,
                prefetch_count,
            )
            .await?;
        Ok(Receiver { inner })
    }

    /// Creates a [`SessionReceiver`] instance that can be used for receiving
    /// and settling messages from a session-enabled queue by accepting the next unlocked session that contains Active messages. If there
    /// are no unlocked sessions with Active messages, then the call will timeout after the configured [`RetryOptions::try_timeout`] value and returns
    /// an error.
    ///
    /// [`ReceiverOptions::receive_mode`] can be specified to configure how messages are received.
    ///
    /// # WebAssembly support
    ///
    /// When creating a receiver for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn accept_session_for_queue(
        &mut self,
        queue_name: impl Into<String>,
        session_id: impl Into<String>,
        options: SessionReceiverOptions,
    ) -> Result<SessionReceiver, azure_core::Error> {
        let entity_path = queue_name.into();
        let session_id = session_id.into();
        self.accept_session(entity_path, session_id, options).await.map_err(Into::into)
    }

    /// Creates a [`SessionReceiver`] instance that can be used for receiving
    /// and settling messages from a session-enabled subscription by accepting the next unlocked session that contains Active messages. If there
    /// are no unlocked sessions with Active messages, then the call will timeout after the configured [`RetryOptions::try_timeout`] value and returns
    /// an error.
    ///
    /// [`ReceiverOptions::receive_mode`] can be specified to configure how messages are received.
    ///
    /// # WebAssembly support
    ///
    /// When creating a receiver for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn accept_session_for_subscription(
        &mut self,
        topic_name: impl AsRef<str>,
        subscription_name: impl AsRef<str>,
        session_id: impl Into<String>,
        options: SessionReceiverOptions,
    ) -> Result<SessionReceiver, azure_core::Error> {
        let entity_path = entity_name_formatter::format_subscription_path(
            topic_name.as_ref(),
            subscription_name.as_ref(),
        );
        let session_id = session_id.into();
        self.accept_session(entity_path, session_id, options).await.map_err(Into::into)
    }

    async fn accept_session(
        &mut self,
        entity_path: String,
        session_id: String,
        options: SessionReceiverOptions,
    ) -> Result<SessionReceiver, OpenReceiverError> {
        let identifier = options
            .identifier
            .unwrap_or_else(|| diagnostics::utilities::generate_identifier(&entity_path));
        let retry_options = self.connection.retry_options().clone();
        let receive_mode = options.receive_mode;
        let prefetch_count = options.prefetch_count;

        let inner = self
            .connection
            .create_transport_session_receiver(
                entity_path,
                identifier,
                retry_options,
                receive_mode,
                prefetch_count,
                Some(session_id.clone()),
            )
            .await?;

        Ok(SessionReceiver { inner, session_id })
    }
}

impl<RP> Client<RP>
where
    RP: ServiceBusRetryPolicyExt + 'static,
{
    /// Creates a [`SessionReceiver`] instance that can be used for receiving and settling
    /// messages from a session-enabled queue by accepting the next unlocked session that contains
    /// Active messages.
    ///
    /// # WebAssembly support
    ///
    /// When creating a receiver for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn accept_next_session_for_queue(
        &mut self,
        queue_name: impl Into<String>,
        options: SessionReceiverOptions,
    ) -> Result<SessionReceiver, azure_core::Error> {
        let entity_path = queue_name.into();
        self.accept_next_session(entity_path, options).await.map_err(Into::into)
    }

    /// Creates a [`SessionReceiver`] instance that can be used for receiving and settling
    /// messages from a session-enabled subscription by accepting the next unlocked session that
    /// contains Active messages.
    ///
    /// # WebAssembly support
    ///
    /// When creating a receiver for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn accept_next_session_for_subscription(
        &mut self,
        topic_name: impl AsRef<str>,
        subscription_name: impl AsRef<str>,
        options: SessionReceiverOptions,
    ) -> Result<SessionReceiver, azure_core::Error> {
        let entity_path = entity_name_formatter::format_subscription_path(
            topic_name.as_ref(),
            subscription_name.as_ref(),
        );
        self.accept_next_session(entity_path, options).await.map_err(Into::into)
    }

    async fn accept_next_session(
        &mut self,
        entity_path: String,
        options: SessionReceiverOptions,
    ) -> Result<SessionReceiver, AcceptNextSessionError> {
        let identifier = options
            .identifier
            .unwrap_or_else(|| diagnostics::utilities::generate_identifier(&entity_path));
        let retry_options = self.connection.retry_options().clone();
        let receive_mode = options.receive_mode;
        let prefetch_count = options.prefetch_count;

        let inner = self
            .connection
            .create_transport_session_receiver(
                entity_path,
                identifier,
                retry_options,
                receive_mode,
                prefetch_count,
                None,
            )
            .await?;

        let session_id = inner
            .session_id()
            .ok_or(AcceptNextSessionError::SessionIdNotSet)?
            .to_string();

        Ok(SessionReceiver { inner, session_id })
    }
}

/* -------------------------------------------------------------------------- */
/*                             Create RuleManager                             */
/* -------------------------------------------------------------------------- */

impl<RP> Client<RP>
where
    RP: ServiceBusRetryPolicyExt + 'static,
{
    /// Creates a [`RuleManager`] instance that can be used for managing rules on a
    /// subscription.
    ///
    /// # WebAssembly support
    ///
    /// When creating a rule manager for `wasm32-unknown-unknown` targets, the method must be called
    /// within the scope of a `tokio::task::LocalSet`.
    pub async fn create_rule_manager(
        &mut self,
        topic_name: impl AsRef<str>,
        subscription_name: impl AsRef<str>,
    ) -> Result<RuleManager, azure_core::Error> {
        let subscription_path = entity_name_formatter::format_subscription_path(
            topic_name.as_ref(),
            subscription_name.as_ref(),
        );
        let identifier = diagnostics::utilities::generate_identifier(&subscription_path);
        let retry_options = self.connection.retry_options().clone();

        let inner = self
            .connection
            .create_transport_rule_manager(subscription_path, identifier, retry_options)
            .await?;

        Ok(RuleManager { inner })
    }
}
