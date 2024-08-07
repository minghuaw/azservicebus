//! Error types for the service bus primitives.

use fe2o3_amqp::{connection::OpenError, session::BeginError};
use fe2o3_amqp_management::error::AttachError;
use timer_kit::error::Elapsed;

use crate::{
    amqp::error::{AmqpClientError, DisposeError},
    authorization::shared_access_signature::SasSignatureError,
    util::IntoAzureCoreError,
};

use super::service_bus_connection_string_properties::FormatError;

/// Argument error
#[derive(Debug)]
pub struct ArgumentError(pub String);

impl std::fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Argument error: {}", self.0.as_str())
    }
}

impl std::error::Error for ArgumentError {}

impl From<ArgumentError> for azure_core::Error {
    fn from(value: ArgumentError) -> Self {
        azure_core::Error::new(azure_core::error::ErrorKind::Other, value)
    }
}

/// The client is already disposed
#[derive(Debug)]
pub struct ClientDisposedError;

impl std::fmt::Display for ClientDisposedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client is disposed")
    }
}

impl std::error::Error for ClientDisposedError {}

impl From<ClientDisposedError> for azure_core::Error {
    fn from(value: ClientDisposedError) -> Self {
        azure_core::Error::new(azure_core::error::ErrorKind::Other, value)
    }
}

// TODO: split this into a few different error types
//
/// Error with service bus connection
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// Argument error
    #[error("Argument error: {}", .0)]
    Argument(#[from] ArgumentError),

    /// Error with the connection string
    #[error(transparent)]
    Format(#[from] FormatError),

    /// Error with the SAS signature
    #[error(transparent)]
    SasSignature(#[from] SasSignatureError),

    /// Error parsing url from connection string
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    /// Error opening the connection
    #[error(transparent)]
    Open(#[from] OpenError),

    /// Error opening the connection over websocket
    #[error(transparent)]
    WebSocket(#[from] fe2o3_amqp_ws::Error),

    /// Opening the connection timed out
    #[error(transparent)]
    Elapsed(#[from] Elapsed),

    /// Error beginning the AMQP session
    #[error(transparent)]
    Begin(#[from] BeginError),

    #[error(transparent)]
    ManagementLinkAttach(#[from] AttachError),

    /// Error disposing the connection
    #[error(transparent)]
    Dispose(#[from] DisposeError),

    /// Client is disposed
    #[error("Client is disposed")]
    ClientDisposed(#[from] ClientDisposedError),

    #[cfg(feature = "transaction")]
    #[error(transparent)]
    ControllerAttach(fe2o3_amqp::link::SenderAttachError),
}

impl From<Error> for azure_core::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Argument(error) => error.into(),
            Error::Format(error) => error.into(),
            Error::SasSignature(error) => error.into(),
            Error::UrlParse(error) => error.into(),
            Error::Open(error) => error.into_azure_core_error(),
            Error::WebSocket(error) => error.into_azure_core_error(),
            Error::Elapsed(error) => error.into_azure_core_error(),
            Error::Begin(error) => error.into_azure_core_error(),
            Error::Dispose(error) => error.into(),
            Error::ClientDisposed(error) => error.into(),
            Error::ManagementLinkAttach(error) => error.into_azure_core_error(),
            #[cfg(feature = "transaction")]
            Error::ControllerAttach(error) => error.into_azure_core_error(),
        }
    }
}

impl From<AmqpClientError> for Error {
    fn from(err: AmqpClientError) -> Self {
        match err {
            AmqpClientError::UrlParseError(err) => Self::UrlParse(err),
            AmqpClientError::Open(err) => Self::Open(err),
            AmqpClientError::WebSocket(err) => Self::WebSocket(err),
            AmqpClientError::Elapsed(err) => Self::Elapsed(err),
            AmqpClientError::Begin(err) => Self::Begin(err),
            AmqpClientError::Dispose(err) => Self::Dispose(err),
            AmqpClientError::ClientDisposed(err) => Self::ClientDisposed(err),
            AmqpClientError::ManagementLinkAttach(err) => Self::ManagementLinkAttach(err),
            #[cfg(feature = "transaction")]
            AmqpClientError::ControllerAttach(err) => Self::ControllerAttach(err),
        }
    }
}

/// Service bus retry policy error
#[derive(Debug, thiserror::Error)]
pub enum RetryError<E> {
    /// Retry policy exhausted
    #[error("Retry policy exhausted")]
    ServiceBusy,

    /// Error with the operation
    #[error(transparent)]
    Operation(E),
}

impl<E> From<RetryError<E>> for azure_core::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: RetryError<E>) -> Self {
        azure_core::Error::new(azure_core::error::ErrorKind::Other, value)
    }
}
