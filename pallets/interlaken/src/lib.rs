#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
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

    #[cfg(feature = "std")]
    // use frame_support::serde::{Serialize, Deserialize};
    use pallet_nft::UniqueAssets;
    use sp_runtime::DispatchResultWithInfo;
    use frame_support::weights::PostDispatchInfo;
    // use sp_runtime::DispatchResultWithInfo;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn nft_price)]
    pub type NFTPrice<T> = StorageMap<_, Twox64Concat, TokenId<T>, BalanceOf<T>>;

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
        /// An account cannot own more Kitties than `MaxKittyCount`.
        ExceedMaxNFTOwned,
        /// Buyer cannot be the owner.
        BuyerIsNFTOwner,
        /// Cannot transfer a kitty to its owner.
        TransferToSelf,
        /// Handles checking whether the Kitty exists.
        NFTNotExist,
        /// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
        NotNFTOwner,
        /// Ensures the Kitty is for sale.
        NFTNotForSale,
        /// Ensures that the buying price is greater than the asking price.
        NFTBidPriceTooLow,
        /// Ensures that an account has enough funds to purchase a Kitty.
        NotEnoughBalance,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        // TODO Part III
        // Success(T::Time, T::Day),
        // A new Kitty was successfully created. \[sender, kitty_id\]
        Created(T::AccountId, T::Hash),
        // Kitty price was successfully set. \[sender, kitty_id, new_price\]
        PriceSet(AccountOf<T>, TokenId<T>, Option<BalanceOf<T>>),
        /// A Kitty was successfully transferred. \[from, to, kitty_id\]
        Transferred(T::AccountId, T::AccountId, TokenId<T>),
        // A Kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
        Bought(AccountOf<T>, AccountOf<T>, TokenId<T>, BalanceOf<T>),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        pub fn buy(origin: OriginFor<T>, token_id: TokenId<T>, receiver: T::AccountId) -> DispatchResult {
            let from = ensure_signed(origin)?;

            ensure!(<T>::UniqueAssets::owner_of(&token_id) == from, <Error<T>>::NotNFTOwner);
            ensure!(from != receiver, <Error<T>>::TransferToSelf);

            let price = Self::nft_price(&token_id);
            let balance = T::Currency::free_balance(&receiver);
            ensure!(price <= Some(balance), <Error<T>>::NotEnoughBalance);
            T::Currency::transfer(&receiver, &from, price.unwrap_or_default(), ExistenceRequirement::KeepAlive);
            T::UniqueAssets::transfer(&receiver, &token_id)
        }

        #[pallet::weight(100)]
        pub fn set_nft_price(origin: OriginFor<T>, token_id: TokenId<T>, price: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(T::UniqueAssets::owner_of(&token_id) == sender, <Error<T>>::NotNFTOwner);
            <NFTPrice<T>>::insert(token_id.clone(), price);
            Self::deposit_event(Event::PriceSet(sender, token_id, Some(price)));
            Ok(())
        }
    }

    // TODO Parts II: helper function for Kitty struct

    impl<T: Config> Pallet<T> {
        pub fn get_assets_by_account(account: T::AccountId) -> Result<Vec<TokenId<T>>, DispatchError> {
            Ok(T::UniqueAssets::assets_of_account(&account))
        }

        pub fn get_nft_price(id: TokenId<T>) -> BalanceOf<T> {
            Self::nft_price(&id).unwrap_or_default()
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
