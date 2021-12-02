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
//! * [`mint`](./enum.Call.html#variant.mint) - Use the provided token info
//!   to create a new token for the specified user. May only be called by
//!   the token admin.
//!
//! * [`burn`](./enum.Call.html#variant.burn) - Destroy a token. May only be
//!   called by token owner.
//!
//! * [`transfer`](./enum.Call.html#variant.transfer) - Transfer ownership of
//!   a token to another account. May only be called by current token
//!   owner.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use scale_info::TypeInfo;

use frame_support::{
    dispatch, ensure,
    traits::{EnsureOrigin, Get},
    Hashable,
};
use frame_system::ensure_signed;
use sp_runtime::traits::{Hash, Member};
use sp_std::{fmt::Debug, vec::Vec};
use codec::{Decode, Encode};

pub mod nft;
pub use crate::nft::UniqueAssets;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[derive(Debug, Default, Encode, Decode, TypeInfo)]
pub struct Token<TAccountId, TPos, TInfo> {
    /// Token owner
    pub owner: TAccountId,
    /// Token position in owner's storage
    pub pos: TPos,
    /// Token info
    pub info: TInfo,
    /// Token meta data
    pub meta: Option<Vec<u8>>,
}

#[frame_support::pallet]
pub mod pallet {
    use codec::FullCodec;
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use crate::UniqueAssets;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The dispatch origin that is able to mint new instances of this type of token.
        type TokenAdmin: EnsureOrigin<Self::Origin>;
        /// The data type that is used to describe this type of token.
        type TokenInfo: Hashable + Member + Debug + Default + FullCodec + MaybeSerializeDeserialize + TypeInfo;

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
    pub type TokenId<T> = <T as frame_system::Config>::Hash;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn total)]
    /// The total number of this type of token that exists (minted - burned).
    pub type Total<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn burned)]
    /// The total number of this type of token that has been burned (may overflow).
    pub type Burned<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_for_account)]
    /// The total number of this type of token owned by an account.
    pub type TotalForAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn tokens_for_account)]
    /// A mapping from an account to a list of all of the tokens of this type that are owned by it.
    pub type TokensForAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<TokenId<T>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn token_by_id)]
    /// A mapping from a token ID to the token's data, including the owner, position to owner's repository and meta data.
    pub type TokenById<T: Config> = StorageMap<_, Identity, TokenId<T>, Token<T::AccountId, u64, T::TokenInfo>, ValueQuery>;

    // decl_storage! {
    // trait Store for Module<T: Config<I>, I: Instance = DefaultInstance> as Token {
    //     /// The total number of this type of token that exists (minted - burned).
    //     Total get(fn total): u128 = 0;
    //     /// The total number of this type of token that has been burned (may overflow).
    //     Burned get(fn burned): u128 = 0;
    //     /// The total number of this type of token owned by an account.
    //     TotalForAccount get(fn total_for_account): map hasher(blake2_128_concat) T::AccountId => u64 = 0;
    //     /// A mapping from an account to a list of all of the tokens of this type that are owned by it.
    //     TokensForAccount get(fn tokens_for_account): map hasher(blake2_128_concat) T::AccountId => Vec<Token<T, I>>;
    //     /// A mapping from a token ID to the account that owns it.
    //     TokenById get(fn token_by_id): map hasher(identity) TokenId<T> => T::AccountId;
    // }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub tokens: Vec<(T::AccountId, Vec<(T::TokenInfo, Option<Vec<u8>>)>)>,
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
                for (info, meta) in infos {
                    <Pallet::<T> as UniqueAssets<_>>::mint(account_id, info.clone(), meta.clone())
                        .expect("Token mint cannot fail during genesis");
                }
            })
            // self.tokens.iter().for_each(|token_class| {
            //     let class_id = Pallet::<T>::create_class(&token_class.0, token_class.1.to_vec(), token_class.2.clone())
            //         .expect("Create class cannot fail while building genesis");
            //     for (account_id, token_metadata, token_data) in &token_class.3 {
            //         Pallet::<T>::mint(account_id, class_id, token_metadata.to_vec(), token_data.clone())
            //             .expect("Token mint cannot fail during genesis");
            //     }
            // })
        }
    }
    // add_extra_genesis {
    //     config(balances): Vec<(T::AccountId, Vec<T::TokenInfo>)>;
    //     build(|config: &GenesisConfig<T, I>| {
    //         for (who, assets) in config.balances.iter() {
    //             for asset in assets {
    //                 match <Module::<T, I> as UniqueAssets::<T::AccountId>>::mint(who, asset.clone()) {
    //                     Ok(_) => {}
    //                     Err(err) => { std::panic::panic_any(err) },
    //                 }
    //             }
    //         }
    //     });
    // }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The token has been burned.
        Burned(TokenId<T>),
        /// The token has been minted and distributed to the account.
        Minted(TokenId<T>, T::AccountId),
        /// Ownership of the token has been transferred to the account.
        Transferred(TokenId<T>, T::AccountId),
    }

    // decl_event!(
    // pub enum Event<T, I = DefaultInstance>
    // where
    //     TokenId = <T as frame_system::Config>::Hash,
    //     AccountId = <T as frame_system::Config>::AccountId,
    // {
    //     /// The token has been burned.
    //     Burned(TokenId),
    //     /// The token has been minted and distributed to the account.
    //     Minted(TokenId, AccountId),
    //     /// Ownership of the token has been transferred to the account.
    //     Transferred(TokenId, AccountId),
    // }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        // Thrown when there is an attempt to mint a duplicate token.
        TokenExists,
        // Thrown when there is an attempt to burn or transfer a nonexistent token.
        NonexistentToken,
        // Thrown when someone who is not the owner of a token attempts to transfer or burn it.
        NotTokenOwner,
        // Thrown when the token admin attempts to mint a token and the maximum number of this
        // type of token already exists.
        TooManyTokens,
        // Thrown when an attempt is made to mint or transfer a token to an account that already
        // owns the maximum number of this type of token.
        TooManyTokensForAccount,
        // Thrown when the token admin attempts to mint a token and the maximum length of this
        // type of token exceeds the limit.
        TooLongMetadata,
    }

    // decl_error! {
    // pub enum Error for Module<T: Config<I>, I: Instance> {
    //     // Thrown when there is an attempt to mint a duplicate token.
    //     TokenExists,
    //     // Thrown when there is an attempt to burn or transfer a nonexistent token.
    //     NonexistentToken,
    //     // Thrown when someone who is not the owner of a token attempts to transfer or burn it.
    //     NotTokenOwner,
    //     // Thrown when the token admin attempts to mint a token and the maximum number of this
    //     // type of token already exists.
    //     TooManyTokens,
    //     // Thrown when an attempt is made to mint or transfer a token to an account that already
    //     // owns the maximum number of this type of token.
    //     TooManyTokensForAccount,
    // }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        // #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        // pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
        //     // Check that the extrinsic was signed and get the signer.
        //     // This function will return an error if the extrinsic is not signed.
        //     // https://substrate.dev/docs/en/knowledgebase/runtime/origin
        //     let who = ensure_signed(origin)?;
        //
        //     // Update storage.
        //     <Something<T>>::put(something);
        //
        //     // Emit an event.
        //     Self::deposit_event(Event::SomethingStored(something, who));
        //     // Return a successful DispatchResultWithPostInfo
        //     Ok(())
        // }

        /// An example dispatchable that may throw a custom error.
        // #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        // pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
        //     let _who = ensure_signed(origin)?;
        //
        //     // Read a value from storage.
        //     match <Something<T>>::get() {
        //         // Return an error if the value has not been set.
        //         None => Err(Error::<T>::NoneValue)?,
        //         Some(old) => {
        //             // Increment the value read from storage; will error in the event of overflow.
        //             let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
        //             // Update the value in storage with the incremented result.
        //             <Something<T>>::put(new);
        //             Ok(())
        //         },
        //     }
        // }

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
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn mint(origin: OriginFor<T>, owner_account: T::AccountId, token_info: T::TokenInfo, token_meta: Option<Vec<u8>>) -> DispatchResult {
            T::TokenAdmin::ensure_origin(origin)?;

            let token_id = <Self as UniqueAssets<_>>::mint(&owner_account, token_info, token_meta)?;
            Self::deposit_event(Event::Minted(token_id, owner_account.clone()));
            Ok(())
        }

        /// Destroy the specified token.
        ///
        /// The dispatch origin for this call must be the token owner.
        ///
        /// - `token_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the token to destroy.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn burn(origin: OriginFor<T>, token_id: TokenId<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who == Self::owner_of(&token_id), Error::<T>::NotTokenOwner);

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
        /// - `token_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the token to destroy.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn transfer(origin: OriginFor<T>, dest_account: T::AccountId, token_id: TokenId<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who == Self::owner_of(&token_id), Error::<T>::NotTokenOwner);

            <Self as UniqueAssets<_>>::transfer(&dest_account, &token_id)?;
            Self::deposit_event(Event::Transferred(token_id.clone(), dest_account.clone()));
            Ok(())
        }
    }

    //
    // decl_module! {
    // pub struct Module<T: Config<I>, I: Instance = DefaultInstance> for enum Call where origin: T::Origin {
    //     type Error = Error<T, I>;
    //     fn deposit_event() = default;
    //
    //     /// Create a new token from the provided token info and identify the specified
    //     /// account as its owner. The ID of the new token will be equal to the hash of the info
    //     /// that defines it, as calculated by the runtime system's hashing algorithm.
    //     ///
    //     /// The dispatch origin for this call must be the token admin.
    //     ///
    //     /// This function will throw an error if it is called with token info that describes
    //     /// an existing (duplicate) token, if the maximum number of this type of token already
    //     /// exists or if the specified owner already owns the maximum number of this type of
    //     /// token.
    //     ///
    //     /// - `owner_account`: Receiver of the token.
    //     /// - `token_info`: The information that defines the token.
    //     #[weight = 10_000]
    //     pub fn mint(origin, owner_account: T::AccountId, token_info: T::TokenInfo) -> dispatch::DispatchResult {
    //         T::TokenAdmin::ensure_origin(origin)?;
    //
    //         let token_id = <Self as UniqueAssets<_>>::mint(&owner_account, token_info)?;
    //         Self::deposit_event(Event::Minted(token_id, owner_account.clone()));
    //         Ok(())
    //     }
    //
    //     /// Destroy the specified token.
    //     ///
    //     /// The dispatch origin for this call must be the token owner.
    //     ///
    //     /// - `token_id`: The hash (calculated by the runtime system's hashing algorithm)
    //     ///   of the info that defines the token to destroy.
    //     #[weight = 10_000]
    //     pub fn burn(origin, token_id: TokenId<T>) -> dispatch::DispatchResult {
    //         let who = ensure_signed(origin)?;
    //         ensure!(who == Self::token_by_id(&token_id), Error::<T, I>::NotTokenOwner);
    //
    //         <Self as UniqueAssets<_>>::burn(&token_id)?;
    //         Self::deposit_event(Event::Burned(token_id.clone()));
    //         Ok(())
    //     }
    //
    //     /// Transfer a token to a new owner.
    //     ///
    //     /// The dispatch origin for this call must be the token owner.
    //     ///
    //     /// This function will throw an error if the new owner already owns the maximum
    //     /// number of this type of token.
    //     ///
    //     /// - `dest_account`: Receiver of the token.
    //     /// - `token_id`: The hash (calculated by the runtime system's hashing algorithm)
    //     ///   of the info that defines the token to destroy.
    //     #[weight = 10_000]
    //     pub fn transfer(origin, dest_account: T::AccountId, token_id: TokenId<T>) -> dispatch::DispatchResult {
    //         let who = ensure_signed(origin)?;
    //         ensure!(who == Self::token_by_id(&token_id), Error::<T, I>::NotTokenOwner);
    //
    //         <Self as UniqueAssets<_>>::transfer(&dest_account, &token_id)?;
    //         Self::deposit_event(Event::Transferred(token_id.clone(), dest_account.clone()));
    //         Ok(())
    //     }
    // }

}

impl<T: Config> UniqueAssets<T::AccountId> for Pallet<T> {
    type AssetId = TokenId<T>;
    type AssetInfo = T::TokenInfo;
    type AssetLimit = T::TokenLimit;
    type UserAssetLimit = T::UserTokenLimit;

    fn total() -> u128 {
        Self::total()
    }

    fn burned() -> u128 {
        Self::burned()
    }

    fn total_for_account(account: &T::AccountId) -> u64 {
        Self::total_for_account(account)
    }

    fn assets_for_account(account: &T::AccountId) -> Vec<Self::AssetId> {
        Self::tokens_for_account(account)
    }

    fn owner_of(token_id: &TokenId<T>) -> T::AccountId {
        Self::token_by_id(token_id).owner
    }

    fn token_metadata(token_id: &TokenId<T>) -> Option<Vec<u8>> {
        Self::token_by_id(token_id).meta
    }

    fn mint(
        owner_account: &T::AccountId,
        token_info: T::TokenInfo,
        token_meta: Option<Vec<u8>>,
    ) -> dispatch::result::Result<TokenId<T>, dispatch::DispatchError> {
        let token_id = T::Hashing::hash_of(&token_info);

        ensure!(
            !TokenById::<T>::contains_key(&token_id),
            Error::<T>::TokenExists
        );

        ensure!(
            Self::total_for_account(owner_account) < T::UserTokenLimit::get(),
            Error::<T>::TooManyTokensForAccount
        );

        ensure!(
            Self::total() < T::TokenLimit::get(),
            Error::<T>::TooManyTokens
        );

        if let Some(ref meta) = token_meta {
            ensure!(
                meta.len() <= T::TokenMetaLimit::get() as usize,
                Error::<T>::TooLongMetadata
            );
        }

        let mut index: u64 = 0;
        Total::<T>::mutate(|total| *total += 1);
        TotalForAccount::<T>::mutate(owner_account, |total| { index = *total; *total += 1 });

        // construct the new token
        let token = Token {
            owner: owner_account.clone(),
            pos: index,
            info: token_info,
            meta: token_meta
        };
        TokenById::<T>::insert(&token_id, token);

        // put onto the owner's account
        TokensForAccount::<T>::mutate(owner_account, |tokens| {
            tokens.push(token_id);
        });

        Ok(token_id)
    }

    fn burn(token_id: &TokenId<T>) -> dispatch::DispatchResult {
        let token = Self::token_by_id(&token_id);
        ensure!(
            token.owner != T::AccountId::default(),
            Error::<T>::NonexistentToken
        );

        Total::<T>::mutate(|total| *total -= 1);
        Burned::<T>::mutate(|total| *total += 1);
        TotalForAccount::<T>::mutate(&token.owner, |total| *total -= 1);
        TokensForAccount::<T>::mutate(&token.owner, |tokens| {
            //let _ = tokens.get(pos).expect("token must be there");
            let pos = token.pos as usize;
            let _ = tokens.swap_remove(pos);
        });
        TokenById::<T>::remove(&token_id);

        Ok(())
    }

    fn transfer(
        dest_account: &T::AccountId,
        token_id: &TokenId<T>,
    ) -> dispatch::DispatchResult {
        let token = Self::token_by_id(&token_id);
        ensure!(
            token.owner != T::AccountId::default(),
            Error::<T>::NonexistentToken
        );

        ensure!(
            Self::total_for_account(dest_account) < T::UserTokenLimit::get(),
            Error::<T>::TooManyTokensForAccount
        );

        TotalForAccount::<T>::mutate(&token.owner, |total| *total -= 1);
        TotalForAccount::<T>::mutate(dest_account, |total| *total += 1);

        // step 1: remove token from the owner
        TokensForAccount::<T>::mutate(&token.owner, |tokens| {
            let pos = token.pos as usize;
            tokens.swap_remove(pos);
        });
        // step 2: push token to the new owner
        let mut new_pos = 0;
        TokensForAccount::<T>::mutate(&dest_account, |tokens| {
            tokens.push(token_id.clone());
            new_pos = tokens.len();
        });
        // step 3: update token_by_id
        TokenById::<T>::mutate(token_id, |token| {
            token.owner = dest_account.clone();
            token.pos = new_pos as u64;
        });

        Ok(())
    }
}

