#![feature(test)]
extern crate test;
extern crate strength_reduce;

macro_rules! bench_unsigned {
    ($module_name:ident, $struct_name:ident, $primitive_type:ident) => (
    	mod $module_name {
    		const REPETITIONS: usize = 1000;

    		#[inline(never)]
    		fn compute_repeated_division_primitive(numerators: &[$primitive_type], divisor: $primitive_type) -> $primitive_type {
    			let mut sum = 0;
    			for numerator in numerators {
	    			sum += *numerator / divisor;
	    		}
	    		sum
    		}

    		#[inline(never)]
    		fn compute_repeated_division(numerators: &[$primitive_type], divisor: strength_reduce::$struct_name) -> $primitive_type {
    			let mut sum = 0;
    			for numerator in numerators {
	    			sum += *numerator / divisor;
	    		}
	    		sum
    		}

    		#[inline(never)]
    		fn compute_single_division(divisors: &[$primitive_type]) -> $primitive_type {
    			let mut sum = 0;
    			for divisor in divisors {
    				let reduced_divisor = strength_reduce::$struct_name::new(*divisor);
	    			sum += 100 / reduced_divisor;
	    		}
	    		sum
    		}

    		#[inline(never)]
    		fn compute_repeated_modulo_primitive(numerators: &[$primitive_type], divisor: $primitive_type) -> $primitive_type {
    			let mut sum = 0;
    			for numerator in numerators {
	    			sum += *numerator % divisor;
	    		}
	    		sum
    		}

    		#[inline(never)]
    		fn compute_repeated_modulo(numerators: &[$primitive_type], divisor: strength_reduce::$struct_name) -> $primitive_type {
    			let mut sum = 0;
    			for numerator in numerators {
	    			sum += *numerator % divisor;
	    		}
	    		sum
    		}

    		#[inline(never)]
    		fn gen_numerators() -> Vec<$primitive_type> {
    			(0..std::$primitive_type::MAX).cycle().take(REPETITIONS).collect::<Vec<$primitive_type>>()
    		}

			#[bench]
			fn division_standard(b: &mut test::Bencher) {
			    let numerators = gen_numerators();
			    let divisor = 6;
			    b.iter(|| { compute_repeated_division_primitive(&numerators, divisor); });
			}

			#[bench]
			fn repeated_division_reduced_shift(b: &mut test::Bencher) {
				let reduced_divisor = strength_reduce::$struct_name::new(8);
				let numerators = gen_numerators();
			    b.iter(|| { compute_repeated_division(&numerators, reduced_divisor); });
			}

			#[bench]
			fn repeated_division_reduced_multiply(b: &mut test::Bencher) {
				let reduced_divisor = strength_reduce::$struct_name::new(6);
				let numerators = gen_numerators();
			    b.iter(|| { compute_repeated_division(&numerators, reduced_divisor); });
			}

			#[bench]
			fn repeated_division_reduced_extrabit(b: &mut test::Bencher) {
				let reduced_divisor = strength_reduce::$struct_name::new(7);
				let numerators = gen_numerators();
			    b.iter(|| { compute_repeated_division(&numerators, reduced_divisor); });
			}

			#[bench]
			fn modulo_standard(b: &mut test::Bencher) {
			    let numerators = gen_numerators();
			    let divisor = 6;
			    b.iter(|| { compute_repeated_modulo_primitive(&numerators, divisor); });
			}

			#[bench]
			fn repeated_modulo_reduced_shift(b: &mut test::Bencher) {
				let reduced_divisor = strength_reduce::$struct_name::new(8);
				let numerators = gen_numerators();
			    b.iter(|| { compute_repeated_modulo(&numerators, reduced_divisor); });
			}

			#[bench]
			fn repeated_modulo_reduced_multiply(b: &mut test::Bencher) {
				let reduced_divisor = strength_reduce::$struct_name::new(6);
				let numerators = gen_numerators();
			    b.iter(|| { compute_repeated_modulo(&numerators, reduced_divisor); });
			}

			#[bench]
			fn repeated_modulo_reduced_extrabit(b: &mut test::Bencher) {
				let reduced_divisor = strength_reduce::$struct_name::new(7);
				let numerators = gen_numerators();
			    b.iter(|| { compute_repeated_modulo(&numerators, reduced_divisor); });
			}
			
			#[bench]
			fn single_division_reduced_shift(b: &mut test::Bencher) {
				let divisors = test::black_box(vec![8; REPETITIONS as usize]);
			    b.iter(|| { compute_single_division(&divisors); });
			}

			#[bench]
			fn single_division_reduced_multiply(b: &mut test::Bencher) {
				let divisors = test::black_box(vec![6; REPETITIONS as usize]);
			    b.iter(|| { compute_single_division(&divisors); });
			}

			#[bench]
			fn single_division_reduced_extrabit(b: &mut test::Bencher) {
				let divisors = test::black_box(vec![7; REPETITIONS as usize]);
			    b.iter(|| { compute_single_division(&divisors); });
			}
		}
    )
}

bench_unsigned!(bench_u08, StrengthReducedU8, u8);
bench_unsigned!(bench_u16, StrengthReducedU16, u16);
bench_unsigned!(bench_u32, StrengthReducedU32, u32);
bench_unsigned!(bench_u64, StrengthReducedU64, u64);