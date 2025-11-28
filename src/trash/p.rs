// use nonstick::{AuthnFlags, ConversationAdapter, Result as PamResult, TransactionBuilder, ErrorCode};
// use zeroize::Zeroizing;
// use thiserror::Error;
// use rand::{thread_rng, Rng};
// use std::ffi::OsString;
// use std::sync::Arc;
// use std::time::Duration;
// use std::thread::sleep;

// /// High-level error type for authentication
// #[derive(Debug, Error)]
// pub enum AuthError {
//     #[error("Invalid credentials")]
//     InvalidCredentials,
//     #[error("PAM error: {0:?}")]
//     Pam(ErrorCode),
//     #[error("IO / System error: {0}")]
//     System(String),
// }

// /// Type alias for convenience
// pub type AuthResult<T> = std::result::Result<T, AuthError>;

// /// Optional callback used to ask user for prompt responses (e.g. OTP).
// /// The bool argument indicates whether the prompt should be masked (password).
// /// The callback should block until user answers and return `None` to cancel.
// pub type PromptCallback = Arc<dyn Fn(&str, bool) -> Option<String> + Send + Sync + 'static>;

// /// Conversation that either uses provided username/password (non-interactive),
// /// or uses a prompt callback for additional prompts.
// pub struct GuiConversation {
//     username: String,
//     password: Zeroizing<String>, // zeroized on drop
//     /// Optional GUI prompt callback (for OTP or other prompts)
//     prompt_cb: Option<PromptCallback>,
// }

// impl GuiConversation {
//     /// Create a conversation that will answer PAM prompts using given username/password.
//     /// If you expect PAM to ask further prompts (OTP, etc.), provide `prompt_cb`.
//     pub fn new<S: Into<String>>(username: S, password: S, prompt_cb: Option<PromptCallback>) -> Self {
//         Self {
//             username: username.into(),
//             password: Zeroizing::new(password.into()),
//             prompt_cb,
//         }
//     }
// }

// impl ConversationAdapter for GuiConversation {
//     fn prompt(&self, _msg: impl AsRef<std::ffi::OsStr>) -> PamResult<OsString> {
//         // non-masked prompt: normally username; reply with username or use prompt_cb if present
//         if let Some(cb) = &self.prompt_cb {
//             // convert message to &str if possible
//             let m = _msg.as_ref().to_string_lossy();
//             if let Some(resp) = cb(&m, false) {
//                 return Ok(OsString::from(resp));
//             } else {
//                 // user cancelled
//                 return Err(ErrorCode::Abort.into());
//             }
//         }

//         Ok(OsString::from(self.username.clone()))
//     }

//     fn masked_prompt(&self, _msg: impl AsRef<std::ffi::OsStr>) -> PamResult<OsString> {
//         // masked prompt: by default return provided password, otherwise call prompt_cb
//         if let Some(cb) = &self.prompt_cb {
//             let m = _msg.as_ref().to_string_lossy();
//             if let Some(resp) = cb(&m, true) {
//                 return Ok(OsString::from(resp));
//             } else {
//                 return Err(ErrorCode::Abort.into());
//             }
//         }

//         Ok(OsString::from(self.password.clone().into_inner()))
//     }

//     fn info_msg(&self, _msg: impl AsRef<std::ffi::OsStr>) {
//         // informational messages from PAM - in GUI you may want to show them
//         // We do nothing here; the prompt callback could handle them if desired.
//         let _ = _msg.as_ref();
//     }

//     fn error_msg(&self, _msg: impl AsRef<std::ffi::OsStr>) {
//         // PAM error message - log or show in GUI as appropriate (not here)
//         let _ = _msg.as_ref();
//     }
// }

// /// Authenticate user using PAM with a robust flow:
// /// authenticate -> account_management -> open_session (optional) -> close_session (on error)
// ///
// /// `service` is the PAM service name, e.g. "myapp" (recommended). Keep that file in /etc/pam.d/myapp.
// /// `prompt_cb` is optional and allows GUI to provide responses to additional prompts (OTP).
// ///
// /// Returns Ok(()) on success or Err(AuthError).
// pub fn authenticate_with_pam(
//     service: &str,
//     username: String,
//     password: String,
//     prompt_cb: Option<PromptCallback>,
// ) -> AuthResult<()> {
//     // Build conversation (password will be zeroized when GuiConversation dropped)
//     let convo = GuiConversation::new(username.clone(), password, prompt_cb);

//     // Build transaction
//     let mut txn = TransactionBuilder::new_with_service(service)
//         .username(username.clone())
//         .build(convo.into_conversation())
//         .map_err(|e| AuthError::Pam(e))?;

//     // Authenticate
//     match txn.authenticate(AuthnFlags::empty()) {
//         Ok(()) => {
//             // Account management: check expiry, locked, etc.
//             txn.account_management(AuthnFlags::empty()).map_err(|e| {
//                 // Close session if it was opened previously (defensive)
//                 let _ = txn.close_session(AuthnFlags::empty());
//                 AuthError::Pam(e)
//             })?;

//             // Optionally open a session (if your app intends to start a login session)
//             // Some apps do not call open_session here; choose depending on desired PAM behavior.
//             if let Err(e) = txn.open_session(AuthnFlags::empty()) {
//                 // if open_session fails, signal error but attempt to close if needed
//                 let _ = txn.close_session(AuthnFlags::empty());
//                 return Err(AuthError::Pam(e));
//             }

//             // Success
//             Ok(())
//         }
//         Err(nonstick::ErrorCode::AuthenticationError) => {
//             // Mitigate timing attacks by adding a small random delay
//             let mut rng = thread_rng();
//             let delay_ms = rng.gen_range(50..200);
//             sleep(Duration::from_millis(delay_ms));

//             Err(AuthError::InvalidCredentials)
//         }
//         Err(e) => Err(AuthError::Pam(e)),
//     }
// }

// /// Helper to safely close a PAM session when your app logs out a user.
// /// You must call this when ending a session that was opened earlier with open_session.
// pub fn close_pam_session(service: &str, username: &str, prompt_cb: Option<PromptCallback>) -> AuthResult<()> {
//     let convo = GuiConversation::new(username.to_string(), "".to_string(), prompt_cb);
//     let mut txn = TransactionBuilder::new_with_service(service)
//         .username(username)
//         .build(convo.into_conversation())
//         .map_err(|e| AuthError::Pam(e))?;

//     txn.close_session(AuthnFlags::empty()).map_err(|e| AuthError::Pam(e))?;
//     Ok(())
// }
