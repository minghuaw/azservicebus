use crate::sealed::Sealed;

pub(crate) trait TransportConnectionScope: Sealed {
    type Error: std::error::Error + Send;

    /// Disposes of the connection scope.
    async fn dispose(&mut self) -> Result<(), Self::Error>;
}
