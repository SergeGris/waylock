use std::{
    cell::RefCell,
    ffi::{OsStr, OsString},
};

use nonstick::{AuthnFlags, ConversationAdapter, Result, Transaction, TransactionBuilder};

struct StaticConversation<'a> {
    info: RefCell<&'a mut dyn FnMut(&OsStr)>,
    error: RefCell<&'a mut dyn FnMut(&OsStr)>,

    username: String,
    password: String,
}

impl<'a> ConversationAdapter for StaticConversation<'a> {
    fn prompt(&self, _msg: impl AsRef<OsStr>) -> Result<OsString> {
        Ok(OsString::from(&self.username))
    }

    fn masked_prompt(&self, _msg: impl AsRef<OsStr>) -> Result<OsString> {
        Ok(OsString::from(&self.password))
    }

    fn info_msg(&self, msg: impl AsRef<OsStr>) {
        self.info.borrow_mut()(msg.as_ref());
        eprintln!("inf {:?}", msg.as_ref());
    }

    fn error_msg(&self, msg: impl AsRef<OsStr>) {
        self.error.borrow_mut()(msg.as_ref());
        eprintln!("err {:?}", msg.as_ref());
    }
}

/// Verifies a username and password against the system’s PAM configuration.
/// Returns `Ok(())` if the credentials are valid, `Err(...)` if invalid.
pub fn authenticate(
    mut info: impl FnMut(&OsStr),
    mut error: impl FnMut(&OsStr),
    username: String,
    password: String,
) -> Result<()> {
    let service = "waylock";

    let convo = StaticConversation {
        info: RefCell::new(&mut info),
        error: RefCell::new(&mut error),
        username: username.clone(),
        password,
    };

    convo.info.borrow_mut()(&OsString::from("ahiosf"));

    let mut transaction = TransactionBuilder::new_with_service(service)
        .username(username)
        .build(convo.into_conversation())?;

    // Authenticate and check account restrictions (expiry, locked, etc).
    transaction
        .authenticate(AuthnFlags::empty())
        .and_then(|_| transaction.account_management(AuthnFlags::empty()))
}
