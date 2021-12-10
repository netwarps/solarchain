#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	/// The API to interact with contracts without using executive.
	pub trait NftApi<AccountId> where
		AccountId: Codec,
	{
		/// Perform a call from a specified account to a given contract.
		///
		/// See `pallet_nft::Pallet::tokens_of`.
		fn tokens_of(
			account: AccountId,
			limit: u64,
			offset: u64,
		) -> Vec<u128>;
	}
}
