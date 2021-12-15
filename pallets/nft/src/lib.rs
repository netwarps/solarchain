//! # Unique Assets Implementation: NFTs
//!
//! This pallet exposes capabilities for managing unique assets, also known as
//! non-fungible tokens (NFTs).
//!
//! - [`pallet_nft::Trait`](./trait.Trait.html)
//! - [`Calls`](./enum.Call.html)
//! - [`Errors`](./enum.Error.html)
//! - [`Events`](./enum.Event.html)
//!
//! ## Overview
//!
//! Assets that share a common metadata structure may be created and distributed
//! by an asset admin. Asset owners may burn assets or transfer their
//! ownership. Configuration parameters are used to limit the total number of a
//! type of asset that may exist as well as the number that any one account may
//! own. Assets are uniquely identified by the hash of the info that defines
//! them, as calculated by the runtime system's hashing algorithm.
//!
//! This pallet implements the [`UniqueAssets`](./nft/trait.UniqueAssets.html)
//! trait in a way that is optimized for assets that are expected to be traded
//! frequently.
//!
//! ### Dispatchable Functions
//!
//! * [`mint`](./enum.Call.html#variant.mint) - Use the provided token info to create a new token
//!   for the specified user. May only be called by the token admin.
//!
//! * [`burn`](./enum.Call.html#variant.burn) - Destroy a token. May only be called by token owner.
//!
//! * [`transfer`](./enum.Call.html#variant.transfer) - Transfer ownership of a token to another
//!   account. May only be called by current token owner.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use scale_info::TypeInfo;

use codec::{Decode, Encode};
use frame_support::{dispatch, ensure, traits::{EnsureOrigin, Get}, BoundedVec};
use frame_system::ensure_signed;
use sp_std::{fmt::Debug, vec::Vec};

pub mod nft;

pub use crate::nft::UniqueAssets;
use sp_std::convert::TryInto;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
#[scale_info(skip_type_params(TMetaLimit))]
pub struct Token<TAccountId, TMetaLimit: Get<u32>> {
    /// Token owner
    pub owner: TAccountId,
    /// Token position in owner's storage
    pub pos: u64,
    /// Token meta data
    pub meta: Option<BoundedVec<u8, TMetaLimit>>,
    /// Other approved user
    pub approval: Option<TAccountId>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use crate::UniqueAssets;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The dispatch origin that is able to mint new instances of this type of token.
        type TokenAdmin: EnsureOrigin<Self::Origin>;
        /// The maximum length of this type of token that may exist.
        #[pallet::constant]
        type TokenMetaLimit: Get<u32>;
        /// The maximum number of this type of token that may exist (minted - burned).
        type TokenLimit: Get<u128>;
        /// The maximum number of this type of token that any single account may own.
        type UserTokenLimit: Get<u64>;
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    /// The runtime system's hashing algorithm is used to uniquely identify tokens.
    pub type TokenId = u128;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    /// Next available token ID.
    #[pallet::storage]
    #[pallet::getter(fn next_token_id)]
    pub type NextTokenId<T: Config> = StorageValue<_, TokenId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total)]
    /// The total number of this type of token that exists (minted - burned).
    pub type Total<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn burned)]
    /// The total number of this type of token that has been burned (may overflow).
    pub type Burned<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_of_account)]
    /// The total number of this type of token owned by an account.
    pub type TotalOfAccount<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn tokens_for_account)]
    /// A mapping from an account to a list of all of the tokens of this type that are owned by it.
    pub type TokensForAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u64,
        TokenId,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn approval_for_all)]
    pub type ApprovalForAll<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        bool
    >;

    #[pallet::storage]
    #[pallet::getter(fn token_by_id)]
    /// A mapping from a token ID to the token's data, including the owner, position to owner's
    /// repository and meta data.
    pub type TokenById<T: Config> =
    StorageMap<_, Identity, TokenId, Token<T::AccountId, T::TokenMetaLimit>, OptionQuery>;

    // decl_storage! {
    // trait Store for Module<T: Config<I>, I: Instance = DefaultInstance> as Token {
    //     /// The total number of this type of token that exists (minted - burned).
    //     Total get(fn total): u128 = 0;
    //     /// The total number of this type of token that has been burned (may overflow).
    //     Burned get(fn burned): u128 = 0;
    //     /// The total number of this type of token owned by an account.
    //     TotalOfAccount get(fn total_for_account): map hasher(blake2_128_concat) T::AccountId =>
    // u64 = 0;     /// A mapping from an account to a list of all of the tokens of this type that
    // are owned by it.     TokensForAccount get(fn tokens_for_account): map
    // hasher(blake2_128_concat) T::AccountId => Vec<Token<T, I>>;     /// A mapping from a token ID
    // to the account that owns it.     TokenById get(fn _token_by_id): map hasher(identity)
    // TokenId<T> => T::AccountId; }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub tokens: Vec<(T::AccountId, Vec<Option<Vec<u8>>>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig { tokens: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            self.tokens.iter().for_each(|(account_id, infos)| {
                for meta in infos {
                    <Pallet<T> as UniqueAssets<_>>::mint(account_id, meta.clone())
                        .expect("Token mint cannot fail during genesis");
                }
            })
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The token has been burned.
        Burned(TokenId),
        /// The token has been minted and distributed to the account.
        Minted(TokenId, T::AccountId),
        /// Ownership of the token has been transferred to the account.
        Transferred(TokenId, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        // Thrown when there is an attempt to mint a duplicate token.
        TokenExists,
        // Thrown when there is an attempt to burn or transfer a nonexistent token.
        NonexistentToken,
        // Thrown when someone who is not the owner of a token attempts to transfer or burn it.
        NotTokenOwnerOrApproval,
        // Thrown when the token admin attempts to mint a token and the maximum number of this
        // type of token already exists.
        TooManyTokens,
        // Thrown when an attempt is made to mint or transfer a token to an account that already
        // owns the maximum number of this type of token.
        TooManyTokensForAccount,
        // Thrown when the token admin attempts to mint a token and the maximum length of this
        // type of token exceeds the limit.
        TooLongMetadata,
        // Thrown when token will be transferred to default account.
        TransferToDefault,
        // Thrown when token owner sets approval to self.
        ApprovedToSelf,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new token from the provided token info and identify the specified
        /// account as its owner. The ID of the new token will be equal to the hash of the info
        /// that defines it, as calculated by the runtime system's hashing algorithm.
        ///
        /// The dispatch origin for this call must be the token admin.
        ///
        /// This function will throw an error if it is called with token info that describes
        /// an existing (duplicate) token, if the maximum number of this type of token already
        /// exists or if the specified owner already owns the maximum number of this type of
        /// token.
        ///
        /// - `owner_account`: Receiver of the token.
        /// - `token_info`: The information that defines the token.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn mint(
            origin: OriginFor<T>,
            owner_account: T::AccountId,
            token_meta: Option<Vec<u8>>,
        ) -> DispatchResult {
            T::TokenAdmin::ensure_origin(origin)?;

            let token_id = <Self as UniqueAssets<_>>::mint(&owner_account, token_meta)?;
            Self::deposit_event(Event::Minted(token_id, owner_account.clone()));
            Ok(())
        }

        /// Destroy the specified token.
        ///
        /// The dispatch origin for this call must be the token owner.
        ///
        /// - `token_id`: The hash (calculated by the runtime system's hashing algorithm) of the
        ///   info that defines the token to destroy.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn burn(origin: OriginFor<T>, token_id: TokenId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(Some(who) == Self::owner_of(&token_id), Error::<T>::NotTokenOwnerOrApproval);
            // ensure!(Self::owner_or_approval(who, &token_id), Error::<T>::NotTokenOwnerOrApproval);

            <Self as UniqueAssets<_>>::burn(&token_id)?;
            Self::deposit_event(Event::Burned(token_id.clone()));
            Ok(())
        }

        /// Transfer a token to a new owner.
        ///
        /// The dispatch origin for this call must be the token owner.
        ///
        /// This function will throw an error if the new owner already owns the maximum
        /// number of this type of token.
        ///
        /// - `dest_account`: Receiver of the token.
        /// - `token_id`: The hash (calculated by the runtime system's hashing algorithm) of the
        ///   info that defines the token to destroy.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn transfer(
            origin: OriginFor<T>,
            dest_account: T::AccountId,
            token_id: TokenId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // ensure!(Some(who) == Self::owner_of(&token_id), Error::<T>::NotTokenOwnerOrApproval);
            ensure!(Self::owner_or_approval(who, &token_id), Error::<T>::NotTokenOwnerOrApproval);

            <Self as UniqueAssets<_>>::transfer(&dest_account, &token_id)?;
            Self::deposit_event(Event::Transferred(token_id.clone(), dest_account.clone()));
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn approve(
            origin: OriginFor<T>,
            dest_account: Option<T::AccountId>,
            token_id: TokenId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let owner = Self::owner_of(&token_id);
            // Check whether token is exists.
            ensure!(owner.is_some(), Error::<T>::NonexistentToken);
            // Not allowed to approve self.
            ensure!(dest_account != owner, Error::<T>::ApprovedToSelf);
            // Check whether caller has authority to operate token.
            ensure!(owner == Some(who.clone()) ||
                Self::is_approve_for_all(owner.unwrap(), who),
                Error::<T>::NotTokenOwnerOrApproval);

            <Self as UniqueAssets<_>>::approve(dest_account, &token_id)?;
            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_approval_for_all(
            origin: OriginFor<T>,
            target_account: T::AccountId,
            approved: bool,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            <Self as UniqueAssets<_>>::set_approval_for_all(owner, target_account, approved)?;
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn tokens_of(account: T::AccountId, limit: u64, offset: u64) -> Vec<u128> {
        TokensForAccount::<T>::iter_prefix(&account)
            .skip(offset as usize)
            .take(limit as usize)
            .map(|(_, token_id)| token_id).collect()
    }
}

impl<T: Config> UniqueAssets<T::AccountId> for Pallet<T> {
    type AssetId = TokenId;
    type AssetLimit = T::TokenLimit;
    type UserAssetLimit = T::UserTokenLimit;

    fn total() -> u128 {
        Self::total()
    }

    fn burned() -> u128 {
        Self::burned()
    }

    fn total_of_account(account: &T::AccountId) -> u64 {
        Self::total_of_account(account)
    }

    fn assets_of_account(account: &T::AccountId) -> Vec<Self::AssetId> {
        TokensForAccount::<T>::iter_prefix(account)
            .map(|(_key, value)| value)
            .collect::<Vec<_>>()
    }

    fn asset_by_account_by_index(account: &T::AccountId, index: u64) -> Option<Self::AssetId> {
        Self::tokens_for_account(account, index)
    }

    fn owner_of(token_id: &TokenId) -> Option<T::AccountId> {
        Self::token_by_id(token_id).map(|t| t.owner)
    }

    fn token_metadata(token_id: &TokenId) -> Option<Vec<u8>> {
        Self::token_by_id(token_id).map_or(None, |t| t.meta.map(|m| m.into()))
    }

    fn mint(
        owner_account: &T::AccountId,
        token_meta: Option<Vec<u8>>,
    ) -> dispatch::result::Result<TokenId, dispatch::DispatchError> {
        ensure!(
			Self::total_of_account(owner_account) < T::UserTokenLimit::get(),
			Error::<T>::TooManyTokensForAccount
		);

        ensure!(Self::total() < T::TokenLimit::get(), Error::<T>::TooManyTokens);

        if let Some(ref meta) = token_meta {
            ensure!(meta.len() <= T::TokenMetaLimit::get() as usize, Error::<T>::TooLongMetadata);
        }
        let bounded_meta: Option<BoundedVec<u8, T::TokenMetaLimit>> = {
            if let Some(meta) = token_meta {
                Some(meta.try_into().map_err(|_| Error::<T>::TooLongMetadata)?)
            } else {
                None
            }
        };

        let token_id = NextTokenId::<T>::try_mutate(|id| -> Result<TokenId, dispatch::DispatchError> {
            *id = id.checked_add(1).ok_or(Error::<T>::TooManyTokens)?;
            Ok(*id)
        })?;

        let mut index: u64 = 0;
        Total::<T>::mutate(|total| *total += 1);
        TotalOfAccount::<T>::mutate(owner_account, |total| {
            index = *total;
            *total += 1
        });

        // construct the new token
        let token =
            Token { owner: owner_account.clone(), pos: index, meta: bounded_meta, approval: None };

        // put onto the owner's account
        TokensForAccount::<T>::insert(owner_account, index, token_id);
        // insert into token_by_id map
        TokenById::<T>::insert(&token_id, token);

        Ok(token_id)
    }

    fn burn(token_id: &TokenId) -> dispatch::DispatchResult {
        let token = Self::token_by_id(token_id);
        ensure!(token.is_some(), Error::<T>::NonexistentToken);
        let token = token.unwrap();

        Total::<T>::mutate(|total| *total -= 1);
        Burned::<T>::mutate(|total| *total += 1);
        TotalOfAccount::<T>::mutate(&token.owner, |total| *total -= 1);
        // remove from tokens_by_account map
        TokensForAccount::<T>::remove(&token.owner, token.pos);
        // remove from token_by_id map
        TokenById::<T>::remove(token_id);

        Ok(())
    }

    fn transfer(dest_account: &T::AccountId, token_id: &TokenId) -> dispatch::DispatchResult {
        let token = Self::token_by_id(&token_id);
        ensure!(token.is_some(), Error::<T>::NonexistentToken);

        ensure!(dest_account != &T::AccountId::default(), Error::<T>::TransferToDefault);

        let mut token = token.unwrap();

        // Each account has a max tokens limit.
        ensure!(
			Self::total_of_account(dest_account) < T::UserTokenLimit::get(),
			Error::<T>::TooManyTokensForAccount
		);

        // step 1: get balance of the target token owner
        let last_token_index = TotalOfAccount::<T>::get(&token.owner) - 1;
        // If account has many tokens, swap it.
        if last_token_index != 0 {
            // step 2: get last token of target token owner by balance
            let last_token_id = TokensForAccount::<T>::get(&token.owner, &last_token_index).unwrap();
            // step 3: swap last token to the position of target token
            TokensForAccount::<T>::insert(token.owner.clone(), token.pos, last_token_id);
            // step 4: reset last token index
            TokenById::<T>::mutate(&last_token_id, |option| {
                if let Some(last_token) = option {
                    last_token.pos = token.pos;
                }
            });
        }
        // step 5: remove last token
        TokensForAccount::<T>::remove(&token.owner, last_token_index);
        // step 6: Origin owner's balance sub 1
        TotalOfAccount::<T>::mutate(&token.owner, |total| *total -= 1);
        // step 7: Newest owner's balance add 1
        let mut new_index: u64 = 0;
        TotalOfAccount::<T>::mutate(dest_account, |total| {
            new_index = *total;
            *total += 1
        });
        // step 8: push token to the new owner
        TokensForAccount::<T>::insert(&dest_account, new_index, token_id);
        // step 9: update token_by_id
        token.owner = dest_account.clone();
        token.pos = new_index;

        TokenById::<T>::insert(token_id, token);
        Ok(())
    }

    fn approve(approval: Option<T::AccountId>, asset_id: &Self::AssetId) -> dispatch::DispatchResult {
        TokenById::<T>::mutate(&asset_id, |option| {
            if let Some(token) = option {
                token.approval = approval;
            }
        });
        Ok(())
    }

    fn set_approval_for_all(owner: T::AccountId, operator: T::AccountId, approved: bool) -> dispatch::DispatchResult {
        // Approved to self is not allowed
        ensure!(owner != operator, Error::<T>::ApprovedToSelf);
        ApprovalForAll::<T>::insert(owner, operator, approved);
        Ok(())
    }

    fn owner_or_approval(target_account: T::AccountId, asset_id: &Self::AssetId) -> bool {
        if let Some(token) = Self::token_by_id(asset_id) {
            let owner = token.owner;
            return &owner == &target_account
                || Self::get_approved(asset_id) == Some(target_account.clone())
                || Self::is_approve_for_all(owner, target_account);
        }
        return false;
    }

    fn get_approved(asset_id: &Self::AssetId) -> Option<T::AccountId> {
        if let Some(t) = Self::token_by_id(asset_id) {
            return t.approval;
        }
        None
    }

    fn is_approve_for_all(owner: T::AccountId, operator: T::AccountId) -> bool {
        Self::approval_for_all(&owner, &operator).unwrap_or(false)
    }
}
