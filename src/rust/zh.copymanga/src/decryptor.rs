use crate::helper::to_aidoku_error;
use aes::{
	cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit},
	Aes128,
};
use aidoku::{error::Result, std::String};
use core::marker::Sized;

type Aes128CbcDec = cbc::Decryptor<Aes128>;

const KEY: &[u8] = b"xxxmanga.woo.key";

pub trait EncryptedString {
	fn decrypt(self) -> Result<Self>
	where
		Self: Sized;
}

impl EncryptedString for String {
	fn decrypt(self) -> Result<Self> {
		let encrypted_data = self.as_bytes();
		let iv = encrypted_data
			.get(..16)
			.ok_or_else(|| to_aidoku_error("Failed to get `iv` from `encrypted_data`"))?;
		let hex_ciphertext = encrypted_data.get(16..).ok_or_else(|| {
			to_aidoku_error("Failed to get `hex_ciphertext` from `encrypted_data`")
		})?;
		let mut ciphertext = hex::decode(hex_ciphertext).map_err(to_aidoku_error)?;

		let plaintext = Aes128CbcDec::new(KEY.into(), iv.into())
			.decrypt_padded_mut::<Pkcs7>(&mut ciphertext)
			.map_err(to_aidoku_error)?;
		Ok(Self::from_utf8_lossy(plaintext).into())
	}
}
