use async_trait::async_trait;
use thiserror::Error;

use super::Adapter;
use crate::domain::subscription::Subscription;

#[derive(Debug, Error)]
pub enum SubscriptionRepositoryError {
    #[error("could not create subscription")]
    Creation,
}

#[async_trait]
pub trait SubscriptionRepository: Adapter {
    async fn create(&self, subscription: &Subscription) -> Result<(), SubscriptionRepositoryError>;
}
