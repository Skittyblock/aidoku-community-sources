use aes::{
	cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit},
	Aes128,
};
use aidoku::std::String;

type Aes128CbcDec = cbc::Decryptor<Aes128>;

const KEY: &[u8] = b"xxxmanga.woo.key";

pub trait EncryptedString {
	fn decrypt(self) -> String;
}

impl EncryptedString for String {
	fn decrypt(self) -> String {
		let encrypted_data = self.as_bytes();
		let iv = &encrypted_data[..16];
		let ciphertext =
			hex::decode(&encrypted_data[16..]).expect("Failed to hex-decode ciphertext.");

		let plaintext_vec = Aes128CbcDec::new(KEY.into(), iv.into())
			.decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
			.expect("Failed to decrypt chapter list");
		String::from_utf8_lossy(&plaintext_vec).into()
	}
}
