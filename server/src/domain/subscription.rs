#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    WaitingValidation,
    Active,
    Unsubscribed,
}

pub struct Subscription {
    pub name: String,
    pub email: String,
    pub status: SubscriptionStatus,
    pub verification_code: String,
    pub unsubscription_code: String,
}
