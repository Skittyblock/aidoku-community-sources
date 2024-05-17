use aes::{
	cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit},
	Aes128,
};
use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::println,
	std::String,
};
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
		// let iv = &encrypted_data[..16];
		let Some(iv) = encrypted_data.get(..16) else {
			println!("Failed to get `iv` from `encrypted_data`");

			return Err(AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			});
		};
		let Some(hex_ciphertext) = encrypted_data.get(16..) else {
			println!("Failed to get `hex_ciphertext` from `encrypted_data`");

			return Err(AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			});
		};
		let mut ciphertext = hex::decode(hex_ciphertext).map_err(|e| {
			println!("{e}");

			AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			}
		})?;

		let plaintext = Aes128CbcDec::new(KEY.into(), iv.into())
			.decrypt_padded_mut::<Pkcs7>(&mut ciphertext)
			.map_err(|e| {
				println!("{e}");

				AidokuError {
					reason: AidokuErrorKind::Unimplemented,
				}
			})?;
		Ok(Self::from_utf8_lossy(plaintext).into())
	}
}
