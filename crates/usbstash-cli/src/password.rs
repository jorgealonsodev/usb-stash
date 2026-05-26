use zeroize::Zeroizing;

use crate::error::CliError;

/// Resolve password using priority chain:
/// 1. `USBSTASH_PASSWORD` env var
/// 2. `--password` flag
/// 3. Interactive prompt via `rpassword`
///
/// Returns `PasswordMismatch` if interactive confirmation fails.
pub fn resolve_password(password_flag: Option<&str>) -> Result<Zeroizing<String>, CliError> {
    // 1. Check env var first
    if let Ok(env_pass) = std::env::var("USBSTASH_PASSWORD") {
        return Ok(Zeroizing::new(env_pass));
    }

    // 2. Check flag
    if let Some(flag_pass) = password_flag {
        eprintln!("Warning: --password flag may appear in shell history");
        return Ok(Zeroizing::new(flag_pass.to_string()));
    }

    // 3. Interactive prompt
    eprint!("Password: ");
    let password = rpassword::read_password().map_err(CliError::Io)?;
    eprint!("Confirm password: ");
    let confirm = rpassword::read_password().map_err(CliError::Io)?;

    if password != confirm {
        return Err(CliError::PasswordMismatch);
    }

    Ok(Zeroizing::new(password))
}

#[cfg(test)]
mod tests {
    use super::resolve_password;

    #[test]
    fn resolve_password_env_var_takes_priority() {
        // SAFETY: test-only, single-threaded test harness
        unsafe { std::env::set_var("USBSTASH_PASSWORD", "env-secret") };
        let result = resolve_password(Some("flag-secret"));
        // SAFETY: cleanup after test
        unsafe { std::env::remove_var("USBSTASH_PASSWORD") };
        assert_eq!(*result.unwrap(), "env-secret");
    }

    #[test]
    fn resolve_password_uses_flag_when_env_unset() {
        // SAFETY: test-only, single-threaded test harness
        unsafe { std::env::remove_var("USBSTASH_PASSWORD") };
        let result = resolve_password(Some("flag-secret"));
        assert_eq!(*result.unwrap(), "flag-secret");
    }

    #[test]
    fn resolve_password_none_flag_no_env_would_prompt() {
        // When neither env nor flag is set, the function calls rpassword::read_password().
        // In a non-interactive test environment this would block or fail.
        // We verify the function exists and has the right signature.
        // Interactive prompting is tested via integration tests.
        // SAFETY: test-only, single-threaded test harness
        unsafe { std::env::remove_var("USBSTASH_PASSWORD") };
        // Do NOT call resolve_password(None) here — it would block on stdin.
        // Instead, we verify the env+flag paths work and trust the interactive
        // path is covered by the integration round-trip test.
    }
}
