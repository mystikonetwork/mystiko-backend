use async_trait::async_trait;

#[async_trait]
pub trait Notification<M>: Send + Sync {
    type Error;

    async fn push(&self, message: M) -> Result<(), Self::Error>;
}
