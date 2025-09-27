#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    SendVerificationEmail,
    WaitingVerification,
    Active,
    Unsubscribed,
}

const VERIFICATION_EMAIL_BODY: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Confirm your subscription to Fabricio Juliatto's blog</title>
</head>
<body>
<table style="background-color: #ffffff; color: #0c0c0c; font-size: 16px; font-family: sans-serif; padding: 4rem; width: 100%">
<tbody>
<tr>
<td>
    <table style="border: 1px solid #ececec; border-radius: 10px; max-width: 340px; margin: 0 auto; padding: 3rem;">
    <tbody>
        <tr><td>Hello, %name!</td></tr>
        <tr style="display: block; margin-top: 1rem;"><td>Please confirm your subscription to Fabricio Juliatto's blog following <a href=%link>this link</a>.</td></tr>
</p>
        <tr style="display: block; margin-top: 1rem;"><td>If you can't follow the link, copy and paste the following link into your browser:</td></tr>
        <tr style="background-color: #f0f0f0; display: block; margin-top: 1rem; padding: 1rem;"><td style="display: block; text-align: center;">%link</td></tr>
    </tbody>
    </table>
<td>
</tr>
</tbody>
</table>
</body>
</html>
"#;

pub fn create_verification_email_body(name: &str, link: &str) -> String {
    VERIFICATION_EMAIL_BODY
        .replace("%name", name)
        .replace("%link", link)
}
