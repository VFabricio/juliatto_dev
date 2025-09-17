use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

use super::Adapter;
use crate::domain::subscription::SubscriptionStatus;

#[derive(Debug, Error)]
pub enum SubscriptionRepositoryError {
    #[error("could not create due to an internal error")]
    DatabaseError,
}

#[derive(Debug)]
pub enum SubscriptionRepositorySuccess {
    Created,
    SubscriptionAlreadyExistsForEmail,
    Updated,
}

#[derive(Debug)]
pub struct CreateSubscription {
    pub name: String,
    pub email: String,
    pub status: SubscriptionStatus,
    pub verification_code: String,
    pub unsubscription_code: String,
}

#[derive(Debug)]
pub struct SubscriptionToSendVerificationEmail {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub verification_code: String,
}

#[async_trait]
pub trait SubscriptionRepository: Adapter {
    async fn create(
        &self,
        subscription: &CreateSubscription,
    ) -> Result<SubscriptionRepositorySuccess, SubscriptionRepositoryError>;

    async fn get_first_subscription_to_send_verification_email_to(
        &self,
    ) -> Result<Option<SubscriptionToSendVerificationEmail>, SubscriptionRepositoryError>;

    async fn update_subscription_status(
        &self,
        id: Uuid,
        status: SubscriptionStatus,
    ) -> Result<(), SubscriptionRepositoryError>;
}
