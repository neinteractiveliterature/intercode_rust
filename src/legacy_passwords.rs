use sha1::{Digest, Sha1};

pub fn verify_legacy_sha1_password(
  password: &str,
  legacy_sha1_password: &str,
  legacy_sha1_password_salt: &str,
) -> bool {
  let legacy_hash = calculate_legacy_password_sha1(password, legacy_sha1_password_salt);
  bcrypt::verify(legacy_hash, legacy_sha1_password).unwrap_or(false)
}

fn calculate_legacy_password_sha1(password: &str, salt: &str) -> String {
  let mut digest = "".to_string();
  for _stretch in 0..10 {
    let mut hasher = Sha1::new();
    hasher.update(format!(
      "--{}--{}--{}--{}--",
      salt, digest, password, "" /* pepper */
    ));
    digest = hex::encode(hasher.finalize());
  }

  digest
}

pub fn verify_legacy_md5_password(password: &str, legacy_md5_password: &str) -> bool {
  let digest = hex::encode(md5::compute(password).0);
  bcrypt::verify(digest, legacy_md5_password).unwrap_or(false)
}

#[cfg(test)]
mod tests {
  use super::{calculate_legacy_password_sha1, verify_legacy_sha1_password};

  #[test]
  fn verify_sha1_matches_devise_algorithm() {
    assert_eq!(
      calculate_legacy_password_sha1("hello", "abc123"),
      "af7316ba26406f21120e7642fd3d2fa705d20692"
    )
  }

  #[test]
  fn verify_correct_sha1_password_is_valid() {
    assert!(verify_legacy_sha1_password(
      "hello",
      "$2a$12$vE.NShwULcotCMu9/PqUOO5nqdLNrPiJ5DUofv6VEcEvK8P1d0FDG",
      "abc123"
    ))
  }

  #[test]
  fn verify_incorrect_sha1_password_is_invalid() {
    assert!(!verify_legacy_sha1_password(
      "Hello",
      "$2a$12$vE.NShwULcotCMu9/PqUOO5nqdLNrPiJ5DUofv6VEcEvK8P1d0FDG",
      "abc123"
    ))
  }
}
