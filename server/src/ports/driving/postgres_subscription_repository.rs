use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{PgPool, Type, postgres::PgPoolOptions, query};
use tracing::{error, info, instrument};

use crate::adapters::{
    Adapter,
    subscription_repository::{
        SubscriptionRepository, SubscriptionRepositoryError, SubscriptionRepositorySuccess,
    },
};
use crate::cross_cutting::error::LogError;
use crate::domain::subscription::{Subscription, SubscriptionStatus};

#[derive(Clone, Debug)]
pub struct PostgresSubscriptionRepository {
    pool: PgPool,
}

impl Adapter for PostgresSubscriptionRepository {}

#[derive(Type)]
#[sqlx(type_name = "subscription_status")]
#[sqlx(rename_all = "snake_case")]
enum DbSubscriptionStatus {
    SendVerificationEmail,
    WaitingVerification,
    Active,
    Unsubscribed,
}

impl From<SubscriptionStatus> for DbSubscriptionStatus {
    fn from(value: SubscriptionStatus) -> Self {
        match value {
            SubscriptionStatus::SendVerificationEmail => {
                DbSubscriptionStatus::SendVerificationEmail
            }
            SubscriptionStatus::WaitingVerification => DbSubscriptionStatus::WaitingVerification,
            SubscriptionStatus::Active => DbSubscriptionStatus::Active,
            SubscriptionStatus::Unsubscribed => DbSubscriptionStatus::Unsubscribed,
        }
    }
}

const MAX_CONNECTIONS: u32 = 100;

impl PostgresSubscriptionRepository {
    pub async fn new<A: AsRef<str>>(connection_string: A) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect(connection_string.as_ref())
            .await
            .context("Could not create Postgres connection pool.")?;
        Ok(Self { pool })
    }
}

fn parse_error(
    error: sqlx::Error,
) -> Result<SubscriptionRepositorySuccess, SubscriptionRepositoryError> {
    if let sqlx::Error::Database(e) = &error
        && e.kind() == sqlx::error::ErrorKind::UniqueViolation
        && e.constraint() == Some("subscriptions_email_key")
    {
        info!("Attempted to create subscription for email that is already registered.");

        return Ok(SubscriptionRepositorySuccess::SubscriptionAlreadyExistsForEmail);
    };
    error!(
        database_error = %error,
        "Failed to insert subscription into database."
    );
    Err(SubscriptionRepositoryError::DatabaseError)
}

#[async_trait]
impl SubscriptionRepository for PostgresSubscriptionRepository {
    #[instrument]
    async fn create(
        &self,
        Subscription {
            name,
            email,
            status,
            verification_code,
            unsubscription_code,
        }: &Subscription,
    ) -> Result<SubscriptionRepositorySuccess, SubscriptionRepositoryError> {
        let status = DbSubscriptionStatus::from(*status);

        query!(
            "INSERT INTO subscriptions (name, email, status, verification_code, unsubscription_code) VALUES ($1, $2, $3, $4, $5);",
            name,
            email,
            status as DbSubscriptionStatus,
            verification_code,
            unsubscription_code
        ).execute(&self.pool).await.map_or_else(parse_error, |_| Ok(SubscriptionRepositorySuccess::Created)).log_error()
    }
}
