pub(crate) trait RecoverableTransport {
    type RecoverError: Send;

    async fn recover(&mut self) -> Result<(), Self::RecoverError>;
}
