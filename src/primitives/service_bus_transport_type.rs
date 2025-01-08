//! Defines the type of protocol and transport that will be used for communicating with Azure Service

/// Specifies the type of protocol and transport that will be used for communicating with Azure
/// Service Bus.
#[derive(Debug, Default, Clone, Copy)]
pub enum ServiceBusTransportType {
    /// The connection uses the AMQP protocol over TCP.
    #[cfg_attr(not(target_arch = "wasm32"), default)]
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(docsrs, doc(cfg(not(target_arch = "wasm32"))))]
    AmqpTcp,

    /// The connection uses the AMQP protocol over web sockets.
    #[cfg_attr(target_arch = "wasm32", default)]
    AmqpWebSocket,
}

impl ServiceBusTransportType {
    pub(crate) const AMQP_SCHEME: &'static str = "amqps";
    pub(crate) const WEBSOCKET_SCHEME: &'static str = "wss";

    /// Returns the URI scheme for the transport type.
    pub fn url_scheme(&self) -> &str {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            ServiceBusTransportType::AmqpTcp => Self::AMQP_SCHEME,
            ServiceBusTransportType::AmqpWebSocket => Self::WEBSOCKET_SCHEME,
        }
    }

    cfg_unsecured!{
        pub(crate) const UNSECURED_AMQP_SCHEME: &'static str = "amqp";
        pub(crate) const UNSECURED_WEBSOCKET_SCHEME: &'static str = "ws";

        /// Returns the URI scheme for the transport type without the secure layer.
        pub fn unsecured_url_scheme(&self) -> &str {
            match self {
                #[cfg(not(target_arch = "wasm32"))]
                ServiceBusTransportType::AmqpTcp => Self::UNSECURED_AMQP_SCHEME,
                ServiceBusTransportType::AmqpWebSocket => Self::UNSECURED_WEBSOCKET_SCHEME,
            }
        }
    }
}
