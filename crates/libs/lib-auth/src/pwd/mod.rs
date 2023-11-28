use lazy_regex::regex_captures;
use std::str::FromStr;
use uuid::Uuid;

mod error;
mod scheme;

pub use self::error::{Error, Result};
use crate::pwd::scheme::{get_scheme, Scheme, DEFAULT_SCHEME};
pub use scheme::SchemeStatus;

pub struct ContentToHash {
	pub content: String, // Clear content.
	pub salt: Uuid,      // Clear salt.
}

/// Hash the password with the default scheme.
pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
	hash_for_scheme(DEFAULT_SCHEME, to_hash)
}

// pub fn validate_pwd(enc_content: &ContentToHash, pwd_ref: &str) -> Result<()> {
pub fn validate_pwd(to_hash: &ContentToHash, pwd_ref: &str) -> Result<SchemeStatus> {
	let PwdParts {
		scheme_name,
		raw: raw_pwd_ref,
	} = pwd_ref.parse()?;
	validate_for_scheme(&scheme_name, to_hash, &raw_pwd_ref)?;
	if scheme_name == DEFAULT_SCHEME {
		Ok(SchemeStatus::Ok)
	} else {
		Ok(SchemeStatus::Outdated)
	}
}

// region:    --- Scheme Infrastructure
fn hash_for_scheme(scheme_name: &str, to_hash: &ContentToHash) -> Result<String> {
	let scheme = get_scheme(scheme_name)?;
	let pwd_raw = scheme.hash(to_hash)?;
	Ok(format!("#{scheme_name}#{}", pwd_raw))
}

fn validate_for_scheme(
	scheme_name: &str,
	to_hash: &ContentToHash,
	raw_pwd_ref: &str,
) -> Result<()> {
	get_scheme(scheme_name)?.validate(to_hash, raw_pwd_ref)?;
	Ok(())
}

struct PwdParts {
	/// The scheme only (e.g. "01")
	scheme_name: String,
	/// The raw password, without the scheme name.
	raw: String,
}

impl FromStr for PwdParts {
	type Err = Error;

	fn from_str(pwd_with_scheme: &str) -> std::result::Result<Self, Self::Err> {
		regex_captures!(
			r#"^#(\w+)#(.*)"#, // a literal regex
			pwd_with_scheme
		)
		.map(|(_whole, scheme, raw)| PwdParts {
			scheme_name: scheme.to_string(),
			raw: raw.to_string(),
		})
		.ok_or(Error::PwdWithSchemeParseFail)
	}
}

// endregion: --- Scheme Infrastructure

// region:    --- Tests
#[cfg(test)]
mod tests {

	use super::*;
	use anyhow::Result;
	use uuid::Uuid;

	#[test]
	fn test_multi_scheme_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
		let fx_to_hash = ContentToHash {
			content: "hello world".to_string(),
			salt: fx_salt,
		};

		// -- Exec
		// hash with Scheme 01
		let pwd_hashed_s01 = hash_for_scheme("01", &fx_to_hash)?;
		// validate with pub function (which will be with Scheme 02)
		let status = validate_pwd(&fx_to_hash, &pwd_hashed_s01)?;

		// -- Check
		assert!(
			matches!(status, SchemeStatus::Outdated),
			"status should be SchemeStatus::Outdated"
		);

		Ok(())
	}
}
// endregion: --- Tests
