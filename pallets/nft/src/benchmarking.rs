
//! Nft pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{ensure};
use frame_system::RawOrigin;
use sp_std::vec;

use crate::Pallet as Nft;

const SEED: u32 = 0;

benchmarks! {
	mint {
	    let i in 0..1024 * 1024;
		let owner: T::AccountId = account("owner", 0, SEED);
	}: _(RawOrigin::Root, owner, Some(vec![0u8; i as usize]))
	verify {
		ensure!(Nft::<T>::total() == 1, "Time was not set.");
	}
	large_mint {
		let owner: T::AccountId = account("owner", 1, SEED);
	}: mint(RawOrigin::Root, owner, Some(vec![0u8; 1024 * 1024]))
}

impl_benchmark_test_suite!(Nft, crate::mock::new_test_ext(), crate::mock::Test);
