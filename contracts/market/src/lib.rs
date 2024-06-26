#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod market {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::HashMap as HashMap,
    };
    use ink_env::DefaultEnvironment;
    use ink_env::call::{build_call, ExecutionInput, Selector};
    use ink_env::call::utils::ReturnType;
    use scale::{
        Decode,
        Encode,
    };
    use ink_prelude::string::{String, ToString};
    use ink_lang::{EmitEvent, Env};
    use crate::market::Error::NotNFTOwner;

    /// A token ID.
    pub type TokenId = u64;

    /// A collection ID.
    pub type CollectionId = u64;

    /// Event emitted when a nft token is asked.
    #[ink(event)]
    pub struct OfferCreated {
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        collection_id: CollectionId,
        #[ink(topic)]
        token_id: TokenId,
        price: Balance,
    }

    /// Event emitted when a nft token is asked.
    #[ink(event)]
    pub struct OfferUpdated {
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        collection_id: CollectionId,
        #[ink(topic)]
        token_id: TokenId,
        old_price: Balance,
        new_price: Balance,
    }

    /// Event emitted when a nft token is canceled to ask.
    #[ink(event)]
    pub struct OfferCancelled {
        #[ink(topic)]
        seller: AccountId,
        #[ink(topic)]
        collection_id: CollectionId,
        #[ink(topic)]
        token_id: TokenId,
    }

    /// Event emitted when a nft token is canceled to ask.
    #[ink(event)]
    pub struct Traded {
        seller: AccountId,
        #[ink(topic)]
        buyer: AccountId,
        #[ink(topic)]
        collection_id: CollectionId,
        #[ink(topic)]
        token_id: TokenId,
        price: Balance,
    }

    /// Error
    #[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Invoke NFT failed
        InvokeNFTTransferFailed,
        /// Invoke FT failed
        InvokeFTTransferFailed,
        /// Not owner for NFT
        NotNFTOwner,
        /// Token owner not found
        OwnerNotFound,
        /// Token is not for sale.
        NotForSale,
    }

    /// Event emitted when a error was triggered.
    #[ink(event)]
    pub struct ErrorEvent {
        #[ink(topic)]
        err: Error,
        #[ink(topic)]
        msg: String,
    }

    #[ink(storage)]
    pub struct NFTMarket {
        /// Contract owner
        owner: AccountId,

        // /// Contract admin (server that will input/output quote currency)
        // admin: AccountId,

        /////////////////////////////////////////////////////////////////////////////////
        // Asks

        /// Current asks: ask_id -> (collectionId, tokenId, price, seller)
        asks: HashMap<u128, (u64, u64, Balance, AccountId)>,

        /// Ask index: Helps find the ask by the collectionId + tokenId
        /// (collectionId + tokenId) -> ask_id
        asks_by_token: HashMap<(CollectionId, TokenId), u128>,

        /// Last Ask ID
        last_ask_id: u128,

        /// NFT contract address
        nft_contract: AccountId,

        /// FT contract address
        ft_contract: AccountId,
    }

    impl NFTMarket {
        #[ink(constructor)]
        pub fn new(nft_contract: AccountId, ft_contract: AccountId) -> Self {
            Self {
                owner: Self::env().caller(),
                asks: HashMap::new(),
                asks_by_token: HashMap::new(),
                last_ask_id: 0,
                nft_contract,
                ft_contract,
            }
        }

        /// Returns the contract owner
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner.clone()
        }

        /// Set NFT contract address
        #[ink(message)]
        pub fn set_nft_contract(&mut self, nft_contract: AccountId) {
            self.ensure_only_owner();
            self.nft_contract = nft_contract;
        }

        /// Get NFT contract address
        #[ink(message)]
        pub fn get_nft_contract(&self) -> AccountId {
            self.nft_contract
        }

        /// Set FT contract address
        #[ink(message)]
        pub fn set_ft_contract(&mut self, ft_contract: AccountId) {
            self.ensure_only_owner();
            self.ft_contract = ft_contract;
        }

        /// Get FT contract address
        #[ink(message)]
        pub fn get_ft_contract(&self) -> AccountId {
            self.ft_contract
        }


        /// User: Place a deposited NFT for sale
        #[ink(message)]
        pub fn ask(&mut self, collection_id: CollectionId, token_id: TokenId, price: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            let owner = self.owner_of(collection_id, token_id);
            if owner.is_none() {
                self.send_error_event(Error::OwnerNotFound, "Owner not found. ".to_string());
                return Err(Error::OwnerNotFound);
            }
            // Only the owner can set price for token.
            if owner.unwrap() != caller {
                self.send_error_event(Error::NotNFTOwner, "Caller is not owner for NFT.".to_string());
                return Err(NotNFTOwner);
            }

            // Place an ask (into asks with a new Ask ID)
            let ask_id = self.last_ask_id + 1;
            let ask = (collection_id, token_id, price, caller);
            self.last_ask_id = ask_id;
            self.asks.insert(ask_id, ask.clone());

            // Record that token is being sold by this user (in asks_by_token) in reverse lookup index
            let result = self.asks_by_token.insert((collection_id, token_id), ask_id);

            if let Some(ask_id) = result {
                if let Some((_, _, old_price, _)) = self.asks.get(&ask_id) {
                    Self::env().emit_event(OfferUpdated {
                        seller: caller,
                        collection_id,
                        token_id,
                        old_price: *old_price,
                        new_price: price,
                    });
                }
            } else {
                Self::env().emit_event(OfferCreated {
                    seller: caller,
                    collection_id,
                    token_id,
                    price,
                });
            }
            Ok(())
        }

        /// Get last ask ID
        #[ink(message)]
        pub fn get_last_ask_id(&self) -> u128 {
            self.last_ask_id
        }

        /// Get ask by token
        #[ink(message)]
        pub fn get_ask_by_token_id(&self, collection_id: CollectionId, token_id: TokenId) -> Option<(u64, u64, Balance, AccountId)> {
            if let Some(ask_id) = self.asks_by_token.get(&(collection_id, token_id)) {
                return self.asks.get(ask_id).cloned();
            }
            None
        }

        /// Cancel an ask
        #[ink(message)]
        pub fn cancel(&mut self, collection_id: CollectionId, token_id: TokenId) {
            let caller = self.env().caller();
            // Ensure that sender owns this ask
            let ask_id = *self.asks_by_token.get(&(collection_id, token_id)).unwrap();
            let (_, _, _, user) = *self.asks.get(&ask_id).unwrap();
            if caller != self.owner {
                assert_eq!(self.env().caller(), user);
            }

            // Remove ask from everywhere
            self.remove_ask(collection_id, token_id, ask_id);

            // Transfer token back to user through NFT Vault (Emit WithdrawNFT event)
            Self::env().emit_event(OfferCancelled {
                seller: caller,
                collection_id,
                token_id,
            });
        }

        /// Match an ask
        #[ink(message)]
        pub fn buy(&mut self, collection_id: CollectionId, token_id: TokenId, new_price: Balance) {
            let buyer = self.env().caller();

            // Get the ask
            if self.asks_by_token.get(&(collection_id, token_id)).is_none() {
                self.send_error_event(Error::NotForSale, "Token is not for sale. ".to_string());
                return;
            }
            let ask_id = *self.asks_by_token.get(&(collection_id, token_id)).unwrap();
            let (_, _, price, seller) = *self.asks.get(&ask_id).unwrap();

            // Check that buyer has enough balance
            let initial_buyer_balance = self.balance_of_or_zero(&buyer);
            assert!(initial_buyer_balance > new_price);
            assert!(new_price >= price);

            // Subtract balance from buyer and increase balance of the seller and owner (due to commission)
            let initial_seller_balance = self.balance_of_or_zero(&seller);
            assert!(initial_seller_balance + price > initial_seller_balance); // overflow protection

            let ft_result = self.transfer_ft(buyer, seller, price);
            assert_eq!(Ok(()), ft_result);
            let nft_result = self.transfer_nft(buyer, collection_id, token_id, seller);
            assert_eq!(Ok(()), nft_result);

            // Remove ask from everywhere
            self.remove_ask(collection_id, token_id, ask_id);

            // Transfer NFT token to buyer through NFT Vault (Emit WithdrawNFT event)
            Self::env().emit_event(Traded {
                seller,
                buyer,
                collection_id,
                token_id,
                price,
            });
        }
    }

    impl NFTMarket {
        /// Transfer NFT
        fn transfer_nft(&self, buyer: AccountId, collection_id: CollectionId, token_id: TokenId, seller: AccountId) -> Result<(), Error> {
            if let Ok(Ok(())) = build_call::<DefaultEnvironment>()
                .callee(self.nft_contract)
                .exec_input(
                    ExecutionInput::new(Selector::new([0x74, 0x72, 0x66, 0x72]))
                        .push_arg(seller)
                        .push_arg(buyer)
                        .push_arg(collection_id)
                        .push_arg(token_id)
                )
                .returns::<ReturnType<Result<(), Error>>>()
                .fire() {
                return Ok(());
            }
            self.send_error_event(Error::InvokeNFTTransferFailed, "Call contract NFT failed. ".to_string());
            Err(Error::InvokeNFTTransferFailed)
        }

        /// Transfer FT
        fn transfer_ft(&self, buyer: AccountId, seller: AccountId, price: Balance) -> Result<(), Error> {
            if let Ok(Ok(())) = build_call::<DefaultEnvironment>()
                .callee(self.ft_contract)
                .exec_input(
                    ExecutionInput::new(Selector::new([0x74, 0x72, 0x66, 0x72]))
                        .push_arg(buyer)
                        .push_arg(seller)
                        .push_arg(price)
                )
                .returns::<ReturnType<Result<(), Error>>>()
                .fire() {
                return Ok(());
            }
            self.send_error_event(Error::InvokeFTTransferFailed, "Call contract FT failed. ".to_string());
            Err(Error::InvokeFTTransferFailed)
        }

        /// Owner of token
        fn owner_of(&self, collection_id: CollectionId, token_id: TokenId) -> Option<AccountId> {
            if let Ok(owner) = build_call::<DefaultEnvironment>()
                .callee(self.nft_contract)
                .exec_input(
                    ExecutionInput::new(Selector::new([0x6F, 0x77, 0x6E, 0x65]))
                        .push_arg(collection_id)
                        .push_arg(token_id)
                )
                .returns::<ReturnType<Option<AccountId>>>()
                .fire() {
                return owner;
            }
            None
        }

        /// Panic if the sender is not the contract owner
        fn ensure_only_owner(&self) {
            assert_eq!(self.env().caller(), self.owner);
        }

        /// Return address balance in quote currency or 0
        /// This is checked from ft
        fn balance_of_or_zero(&self, user: &AccountId) -> Balance {
            if let Ok(balance) = build_call::<DefaultEnvironment>()
                .callee(self.ft_contract)
                .exec_input(ExecutionInput::new(Selector::new([0x62, 0x61, 0x6C, 0x61]))
                    .push_arg(user)
                )
                .returns::<ReturnType<Balance>>()
                .fire() {
                return balance;
            }
            0
        }

        fn remove_ask(&mut self, collection_id: CollectionId, token_id: TokenId, ask_id: u128) {
            // Remove the record that token is being sold by this user (from asks_by_token)
            let _ = self.asks_by_token.take(&(collection_id, token_id));

            // Remove an ask (from asks)
            let _ = self.asks.take(&ask_id);
        }
        #[inline]
        fn send_error_event(&self, err: Error, msg: String) {
            self.env().emit_event(ErrorEvent {
                err,
                msg,
            });
        }
    }


    #[cfg(test)]
    mod tests {}
}

