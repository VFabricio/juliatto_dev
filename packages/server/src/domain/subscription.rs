#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    SendVerificationEmail,
    WaitingVerification,
    Active,
    Unsubscribed,
}

// TODO: blog name
const VERIFICATION_EMAIL_BODY: &str = "
    <p>Hello, %name</p>
    <p>Please confirm your subscription to ..... with this link.
        <a href=%link>Confirm email address</a>
    </p>
    <p>If you can't follow the link, copy and paste the following link into your browser:</p>
    <p>%link</p>
";

pub fn create_verification_email_body(name: &str, link: &str) -> String {
    VERIFICATION_EMAIL_BODY
        .replace("%name", name)
        .replace("%link", link)
}
