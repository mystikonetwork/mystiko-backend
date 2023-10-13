use crate::Notification;
use async_trait::async_trait;
use rusoto_core::{Region, RusotoError};
use rusoto_sns::{PublishError, PublishInput, Sns, SnsClient};
use thiserror::Error;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct SnsNotification<S: Sns = SnsClient> {
    client: S,
}

#[derive(Debug, Error)]
pub enum SnsNotificationError {
    #[error(transparent)]
    PushError(#[from] RusotoError<PublishError>),
}

#[async_trait]
impl<S> Notification<PublishInput> for SnsNotification<S>
where
    S: Sns + Send + Sync,
{
    type Error = SnsNotificationError;

    async fn push(&self, message: PublishInput) -> Result<(), Self::Error> {
        self.client.publish(message).await?;
        Ok(())
    }
}

impl SnsNotification<SnsClient> {
    pub fn new(client: SnsClient) -> Self {
        Self { client }
    }

    pub fn from_region<R>(region: R) -> Self
    where
        R: Into<Region>,
    {
        Self::new(SnsClient::new(region.into()))
    }
}
