use hmac::{Hmac, Mac};
use lib_utils::b64::b64u_encode;
use sha2::Sha512;

use super::{Error, Result};
use crate::{
	config::auth_config,
	pwd::{scheme::Scheme, ContentToHash},
};

pub struct Scheme01;

impl Scheme for Scheme01 {
	fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
		let key = &auth_config().PWD_KEY;
		hash(key, to_hash)
	}

	fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: &str) -> Result<()> {
		let raw_pwd_new = self.hash(to_hash)?;
		if raw_pwd_new == raw_pwd_ref {
			Ok(())
		} else {
			Err(Error::PwdValidate)
		}
	}
}

fn hash(key: &[u8], to_hash: &ContentToHash) -> Result<String> {
	let ContentToHash { content, salt } = to_hash;

	// -- Create a HMAC-SHA-512 from key.
	let mut hmac_sha512 =
		Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::Key)?;

	// -- Add content.
	hmac_sha512.update(content.as_bytes());
	hmac_sha512.update(salt.as_bytes());

	// -- Finalize and b64u encode.
	let hmac_result = hmac_sha512.finalize();
	let result_bytes = hmac_result.into_bytes();

	let result = b64u_encode(result_bytes);

	Ok(result)
}
