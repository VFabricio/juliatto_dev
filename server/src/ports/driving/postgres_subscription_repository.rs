use async_trait::async_trait;

use crate::adapters::{
    Adapter,
    subscription_repository::{SubscriptionRepository, SubscriptionRepositoryError},
};
use crate::domain::subscription::Subscription;

#[derive(Clone, Debug)]
pub struct PostgresSubscriptionRepository {}

impl Adapter for PostgresSubscriptionRepository {}

#[async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    async fn create(
        &self,
        _subscription: &Subscription,
    ) -> Result<(), SubscriptionRepositoryError> {
        Ok(())
    }
}
