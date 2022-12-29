extern crate alloc;
use alloc::vec::Vec;
use digest::{Digest, FixedOutputReset, HashMarker};

pub fn evpkdf<D: Default + FixedOutputReset + HashMarker>(
	pass: &[u8],
	salt: &[u8],
	count: usize,
	output: &mut [u8],
) {
	let mut hasher = D::default();
	let mut derived_key = Vec::with_capacity(output.len());
	let mut block = Vec::new();

	while derived_key.len() < output.len() {
		if !block.is_empty() {
			hasher.update(&block);
		}
		hasher.update(pass);
		hasher.update(salt.as_ref());
		block = hasher.finalize_reset().to_vec();

		// avoid subtract with overflow
		if count > 1 {
			for _ in 0..(count - 1) {
				hasher.update(&block);
				block = hasher.finalize_reset().to_vec();
			}
		}

		derived_key.extend_from_slice(&block);
	}

	output.copy_from_slice(&derived_key[0..output.len()]);
}
