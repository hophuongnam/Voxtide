use voxtide_core::{Error, Result};

#[test]
fn error_displays_user_friendly_message() {
    let err: Error = Error::Soniox("auth failed".into());
    assert_eq!(err.to_string(), "Soniox: auth failed");
}

#[test]
fn result_is_alias_for_error() {
    let _: Result<()> = Ok(());
}
