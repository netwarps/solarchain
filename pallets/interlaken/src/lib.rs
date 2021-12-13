#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        // sp_runtime::traits::Zero,
        dispatch::DispatchResult,
        traits::{Currency, tokens::ExistenceRequirement},
        // storage::{
        //     types::{ValueQuery, StorageValue},
        // },
        pallet_prelude::*,
    };
    // use scale_info::TypeInfo;
    use frame_system::pallet_prelude::*;
    // use sp_core::hashing::blake2_128;

    // use frame_support::serde::{Serialize, Deserialize};
    use pallet_nft::UniqueAssets;
    // use sp_runtime::DispatchResultWithInfo;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn nft_price)]
    pub type NFTPrice<T> = StorageMap<_, Twox64Concat, TokenId<T>, BalanceOf<T>>;
    // pub type NFTPrice<T> = StorageMap<_, Twox64Concat, TokenId<T>, TransactionInfo<BalanceOf<T>>>;

    // pub type NFTTPrice<T> = StorageDoubleMap<_, Twox64Concat, TokenId<T>, Twox64Concat, bool, BalanceOf<T>>;

    // Account
    pub type AccountOf<T> = <T as frame_system::Config>::AccountId;

    /// TokenID
    pub type TokenId<T> =
    <<T as Config>::UniqueAssets as UniqueAssets<<T as frame_system::Config>::AccountId>>::AssetId;
    // <T::UniqueAssets as UniqueAssets<T::AccountId>>::AssetId;

    /// Balance
    type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;
        type UniqueAssets: UniqueAssets<Self::AccountId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// An account cannot own more NFT than `MaxNFTCount`.
        ExceedMaxNFTOwned,
        /// Buyer cannot be the owner.
        BuyerIsNFTOwner,
        /// Cannot transfer a NFT to its owner.
        TransferToSelf,
        /// Handles checking whether the NFT exists.
        NFTNotExist,
        /// Handles checking that the NFT is owned by the account transferring, buying or setting a price for it.
        NotNFTOwner,
        /// Ensures the NFT is for sale.
        NFTNotForSale,
        /// Ensures that the buying price is greater than the asking price.
        NFTBidPriceTooLow,
        /// Ensures that an account has enough funds to purchase a NFT.
        NotEnoughBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        // TODO Part III
        // Success(T::Time, T::Day),
        // A new NFT was successfully created. \[sender, token_id\]
        Created(T::AccountId, T::Hash),
        // NFT price was successfully set. \[sender, token_id, new_price\]
        PriceSet(AccountOf<T>, TokenId<T>, Option<BalanceOf<T>>),
        /// A NFT was successfully transferred. \[from, to, token_id\]
        Transferred(T::AccountId, T::AccountId, TokenId<T>),
        // A NFT was successfully bought. \[buyer, seller, token_id, bid_price\]
        Bought(T::AccountId, T::AccountId, TokenId<T>, BalanceOf<T>),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        pub fn buy(origin: OriginFor<T>, token_id: TokenId<T>) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            let owner = <T>::UniqueAssets::owner_of(&token_id);

            ensure!(owner.is_some(), Error::<T>::NFTNotExist);
            let owner = owner.unwrap();
            ensure!(&owner != &buyer, Error::<T>::TransferToSelf);

            let price = Self::nft_price(&token_id);
            ensure!(price.is_some(), Error::<T>::NFTNotForSale);
            let balance = T::Currency::free_balance(&buyer);
            ensure!(price <= Some(balance), Error::<T>::NotEnoughBalance);
            T::Currency::transfer(&buyer, &owner, price.unwrap(), ExistenceRequirement::KeepAlive)?;
            T::UniqueAssets::transfer(&buyer, &token_id)?;
            Self::deposit_event(Event::Bought(buyer, owner, token_id, price.unwrap()));
            Ok(())
        }

        #[pallet::weight(100)]
        pub fn set_nft_price(origin: OriginFor<T>, token_id: TokenId<T>, price: BalanceOf<T>) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            ensure!(T::UniqueAssets::owner_or_approval(owner.clone(), &token_id), Error::<T>::NotNFTOwner);
            <NFTPrice<T>>::insert(token_id.clone(), price);
            Self::deposit_event(Event::PriceSet(owner, token_id, Some(price)));
            Ok(())
        }

        #[pallet::weight(100)]
        pub fn set_nft_not_for_sale(origin: OriginFor<T>, token_id: TokenId<T>) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            ensure!(T::UniqueAssets::owner_of(&token_id) == Some(owner.clone()), Error::<T>::NotNFTOwner);

            // Remove means that NFT is not for sale.
            <NFTPrice<T>>::remove(&token_id);
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn get_nft_price(id: TokenId<T>) -> BalanceOf<T> {
            Self::nft_price(&id).unwrap_or_default()
        }

        pub fn get_all_nft() -> Vec<(TokenId<T>, BalanceOf<T>)> {
            <NFTPrice<T>>::iter().collect::<Vec<_>>()
        }
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub tokens: u32,
        _dummy: PhantomData<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> GenesisConfig<T> {
            GenesisConfig { tokens: 0, _dummy: Default::default() }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // self.tokens.iter().for_each(|(account, info)|{
            //     for (token, meta) in info {
            //         <Pallet<T>>::mint()
            //     }
            // });
        }
    }
}
