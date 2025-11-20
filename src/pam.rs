use nonstick::{AuthnFlags, ConversationAdapter, Result, Transaction, TransactionBuilder};
use std::ffi::{OsStr, OsString};

/// A simple non-interactive PAM conversation handler.
/// PAM can prompt for input; this struct supplies fixed responses.
struct StaticConversation {
    username: String,
    password: String,
}

impl ConversationAdapter for StaticConversation {
    fn prompt(&self, _msg: impl AsRef<OsStr>) -> Result<OsString> {
        Ok(OsString::from(&self.username))
    }

    fn masked_prompt(&self, _msg: impl AsRef<OsStr>) -> Result<OsString> {
        Ok(OsString::from(&self.password))
    }

    fn info_msg(&self, _msg: impl AsRef<OsStr>) {}
    fn error_msg(&self, _msg: impl AsRef<OsStr>) {}
}

/// Verifies a username and password against the system’s PAM configuration.
/// Returns `Ok(())` if the credentials are valid, `Err(...)` if invalid.
pub fn authenticate(username: String, password: String) -> Result<()> {
    let service = "waylock";

    // Build a fixed conversation adapter
    let convo = StaticConversation {
        username: username.clone(),
        password,
    };

    let mut transaction = TransactionBuilder::new_with_service(service)
        .username(username)
        .build(convo.into_conversation())?;

    match transaction.authenticate(AuthnFlags::empty()) {
        Ok(()) => {
            // Check account restrictions (expiry, locked, etc.)
            transaction.account_management(AuthnFlags::empty())?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}
