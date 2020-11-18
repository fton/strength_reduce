
// multiply the 256-bit number 'a' by the 128-bit number 'b' and return the uppermost 128 bits of the product
// ripped directly from num-biguint's long multiplication algorithm (mac3, mac_with_carry, adc), but with fixed-
// size arrays instead of slices
#[inline]
pub(crate) const fn multiply_256_by_128_upperbits(a_hi: u128, a_lo: u128, b: u128) -> u128 {
	// Break a and b into little-endian 64-bit chunks
	let a_chunks = [
		a_lo as u64,
		(a_lo >> 64) as u64,
		a_hi as u64,
		(a_hi >> 64) as u64,
	];
	let b_chunks = [
		b as u64,
		(b >> 64) as u64,
	];

	// Multiply b by a, one chink of b at a time
	let product = multiply_256_by_64_helper([0; 6], 0, &a_chunks, b_chunks[0]);
	let product = multiply_256_by_64_helper(product, 1, &a_chunks, b_chunks[1]);

	// the last 2 elements of the array have the part of the productthat we care about
	((product[5] as u128) << 64) | (product[4] as u128)
}

#[inline]
const fn multiply_256_by_64_helper(mut product: [u64; 6], ibeg: usize, a: &[u64;4], b: u64) -> [u64; 6] {
	if b != 0 {
		let mut carry = 0;

		// Multiply each of the digits in a by b, adding them into the 'product' value.
		// We don't zero out product, because we this will be called multiple times, so it probably contains a
		// previous iteration's partial product, and we're adding + carrying on top of it
		let mut i = ibeg;

		while i < a.len() {
			carry += product[i] as u128;
			carry += (a[i] as u128) * (b as u128);

			product[i] = carry as u64;
			carry >>= 64;
			i += 1;
		}

		// We're done multiplying, we just need to finish carrying through the rest of the product.
		let mut i = ibeg + a.len();

		while carry != 0 && i < product.len() {
			carry += product[i] as u128;

			product[i] = carry as u64;
			carry >>= 64;
			i += 1;
		}
	}
	product
}

// compute product += a * b
#[inline]
pub(crate) const fn long_multiply(a: &[u64], a_len: usize, b: u64, mut product: [u64; 3]) -> [u64; 3] {
	if b != 0 {
		let mut carry = 0;

		// Multiply each of the digits in a by b, adding them into the 'product' value.
		// We don't zero out product, because we this will be called multiple times, so it probably contains a
		// previous iteration's partial product, and we're adding + carrying on top of it
		let mut i = 0;

		while i < a_len {
			carry += product[i] as u128;
			carry += (a[i] as u128) * (b as u128);

			product[i] = carry as u64;
			carry >>= 64;
			i += 1;
		}

		// We're done multiplying, we just need to finish carrying through the rest of the product.
		let mut i = 0;
		let product_hi_len = product.len() - a_len;

		while carry != 0 && i < product_hi_len {
			carry += product[i + a_len] as u128;

			product[i + a_len] = carry as u64;
			carry >>= 64;
			i += 1;
		}
	}
	product
}
