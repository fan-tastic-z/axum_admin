use uuid::Uuid;

use crate::config;

mod error;
mod hmac_hasher;

pub use self::error::{Error, Result};
use self::hmac_hasher::hmac_sha512_hash;

pub struct ContentToHash {
	pub content: String, // Clear content.
	pub salt: Uuid,      // Clear salt.
}

pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
	let key = &config().PWD_KEY;

	let encrypted = hmac_sha512_hash(key, to_hash)?;

	Ok(format!("#01#{encrypted}"))
}

pub fn validate_pwd(enc_content: &ContentToHash, pwd_ref: &str) -> Result<()> {
	let pwd = hash_pwd(enc_content)?;

	if pwd == pwd_ref {
		Ok(())
	} else {
		Err(Error::NotMatching)
	}
}
