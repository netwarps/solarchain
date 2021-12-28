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

        /// Ask index: Helps find the ask by the colectionId + tokenId
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

        /// Set FT contract address
        #[ink(message)]
        pub fn set_ft_contract(&mut self, ft_contract: AccountId) {
            self.ensure_only_owner();
            self.ft_contract = ft_contract;
        }

        /// User: Place a deposited NFT for sale
        #[ink(message)]
        pub fn ask(&mut self, collection_id: CollectionId, token_id: TokenId, price: Balance) {
            let caller = self.env().caller();
            let owner = self.owner_of(collection_id, token_id);
            if owner.is_none() {
                return;
            }
            // Only the owner can set price for token.
            if owner.unwrap() != caller {
                return;
            }

            // Place an ask (into asks with a new Ask ID)
            let ask_id = self.last_ask_id + 1;
            let ask = (collection_id, token_id, price, caller);
            self.last_ask_id = ask_id;
            let result = self.asks.insert(ask_id, ask.clone());
            if let Some((_, _, old_price, _)) = result {
                Self::env().emit_event(OfferUpdated {
                    seller: caller,
                    collection_id,
                    token_id,
                    old_price,
                    new_price: price,
                });
            } else {
                Self::env().emit_event(OfferCreated {
                    seller: caller,
                    collection_id,
                    token_id,
                    price,
                });
            }

            // Record that token is being sold by this user (in asks_by_token) in reverse lookup index
            self.asks_by_token.insert((collection_id, token_id), ask_id);
        }

        /// Get last ask ID
        #[ink(message)]
        pub fn get_last_ask_id(&self) -> u128 {
            self.last_ask_id
        }

        /// Get ask by ID
        #[ink(message)]
        pub fn get_ask_by_id(&self, ask_id: u128) -> (u64, u64, Balance, AccountId) {
            *self.asks.get(&ask_id).unwrap()
        }

        /// Get ask by token
        #[ink(message)]
        pub fn get_ask_id_by_token(&self, collection_id: CollectionId, token_id: TokenId) -> u128 {
            *self.asks_by_token.get(&(collection_id, token_id)).unwrap()
        }

        /// Cancel an ask
        #[ink(message)]
        pub fn cancel(&mut self, collection_id: CollectionId, token_id: TokenId) {

            // Ensure that sender owns this ask
            let ask_id = *self.asks_by_token.get(&(collection_id, token_id)).unwrap();
            let (_, _, _, user) = *self.asks.get(&ask_id).unwrap();
            if self.env().caller() != self.owner {
                assert_eq!(self.env().caller(), user);
            }

            // Remove ask from everywhere
            self.remove_ask(collection_id, token_id, ask_id);

            // Transfer token back to user through NFT Vault (Emit WithdrawNFT event)
            Self::env().emit_event(OfferCancelled {
                seller: self.env().caller().clone(),
                collection_id,
                token_id,
            });
        }

        /// Match an ask
        #[ink(message)]
        pub fn buy(&mut self, collection_id: CollectionId, token_id: TokenId, new_price: Balance) {
            let buyer = self.env().caller();

            // Get the ask
            let ask_id = *self.asks_by_token.get(&(collection_id, token_id)).unwrap();
            let (_, _, price, seller) = *self.asks.get(&ask_id).unwrap();

            // Check that buyer has enough balance
            let initial_buyer_balance = self.balance_of_or_zero(&buyer);
            assert!(initial_buyer_balance > new_price);
            assert!(new_price >= price);

            // Subtract balance from buyer and increase balance of the seller and owner (due to commission)
            let initial_seller_balance = self.balance_of_or_zero(&seller);
            assert!(initial_seller_balance + price > initial_seller_balance); // overflow protection

            self.transfer_ft(buyer, seller, price);
            self.transfer_nft(buyer, collection_id, token_id, seller);

            // Remove ask from everywhere
            self.remove_ask(collection_id, token_id, ask_id);

            // Transfer NFT token to buyer through NFT Vault (Emit WithdrawNFT event)
            Self::env().emit_event(Traded {
                seller,
                buyer: self.env().caller().clone(),
                collection_id,
                token_id,
                price,
            });
        }

        /// Transfer NFT
        fn transfer_nft(&self, buyer: AccountId, collection_id: CollectionId, token_id: TokenId, seller: AccountId) {
            let call_params = build_call::<DefaultEnvironment>()
                .callee(self.nft_contract)
                .exec_input(
                    ExecutionInput::new(Selector::new([0x74, 0x72, 0x66, 0x72]))
                        .push_arg(seller)
                        .push_arg(buyer)
                        .push_arg(collection_id)
                        .push_arg(token_id)
                )
                .returns::<()>()
                .params();

            self.env().invoke_contract(&call_params).expect("call invocation must succeed");
        }

        /// Transfer FT
        fn transfer_ft(&self, buyer: AccountId, seller: AccountId, price: Balance) {
            let call_params = build_call::<DefaultEnvironment>()
                .callee(self.ft_contract)
                .exec_input(
                    ExecutionInput::new(Selector::new([0x74, 0x72, 0x66, 0x72]))
                        .push_arg(buyer)
                        .push_arg(seller)
                        .push_arg(price)
                )
                .returns::<()>()
                .params();

            self.env().invoke_contract(&call_params).expect("call invocation must succeed");
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
    }


    #[cfg(test)]
    mod tests {}
}

