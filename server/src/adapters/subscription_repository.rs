use async_trait::async_trait;
use thiserror::Error;

use super::Adapter;
use crate::domain::subscription::Subscription;

#[derive(Debug, Error)]
pub enum SubscriptionRepositoryError {
    #[error("could not create due to an internal error")]
    DatabaseError,
}

#[derive(Debug)]
pub enum SubscriptionRepositorySuccess {
    Created,
    SubscriptionAlreadyExistsForEmail,
}

#[async_trait]
pub trait SubscriptionRepository: Adapter {
    async fn create(
        &self,
        subscription: &Subscription,
    ) -> Result<SubscriptionRepositorySuccess, SubscriptionRepositoryError>;
}
