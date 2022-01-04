//! This is a NFT contract.

#![cfg_attr(not(feature = "std"), no_std)]

mod token_info;
mod test;

use ink_lang as ink;

#[ink::contract]
pub mod nft {
    // #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::collections::{
        hashmap::Entry,
        HashMap as StorageHashMap,
    };
    use ink_storage::lazy::Lazy;
    use ink_prelude::{
        string::{String, ToString},
        vec::Vec,
    };
    use scale::{
        Decode,
        Encode,
    };
    use ink_lang::{EmitEvent, Env};
    use crate::token_info::TokenInfo;

    /// A token ID.
    pub type TokenId = u64;

    /// A collection ID.
    pub type CollectionId = u64;

    #[ink(storage)]
    #[derive(Default)]
    pub struct NFT {
        /// Symbols of ERC20 Token, by (name, symbol)
        symbols: Lazy<(String, String)>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: StorageHashMap<AccountId, u64>,
        /// Mapping from owner to list of owned token IDs
        owned_tokens: StorageHashMap<AccountId, Vec<(CollectionId, TokenId)>>,
        /// Mapping from token_id to token info, such as owner, approval, etc.
        token_collection: StorageHashMap<(CollectionId, TokenId), TokenInfo>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenURIExists,
        TokenNotFound,
        CannotInsert,
        CannotRemove,
        CannotFetchValue,
        NotAllowed,
    }

    /// Event emitted when a error was triggered.
    #[ink(event)]
    pub struct ErrorEvent {
        #[ink(topic)]
        err: Error,
        #[ink(topic)]
        msg: String,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        collection_id: CollectionId,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a token minted occurs.
    #[ink(event)]
    pub struct Minted {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        collection_id: CollectionId,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a token burned occurs.
    #[ink(event)]
    pub struct Burned {
        caller: AccountId,
        #[ink(topic)]
        collection_id: CollectionId,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    impl NFT {
        /// Creates a new NFT contract.
        #[ink(constructor)]
        pub fn new(name: String, symbols: String) -> Self {
            Self {
                symbols: Lazy::new((name, symbols)),
                owned_tokens_count: Default::default(),
                owned_tokens: Default::default(),
                token_collection: Default::default(),
            }
        }
        /// Returns the name of the token.
        #[ink(message)]
        pub fn name(&self) -> String {
            self.symbols.0.clone()
        }

        /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
        #[ink(message)]
        pub fn token_url(&self, collection_id: CollectionId, token_id: TokenId) -> Option<String> {
            match self.token_collection.get(&(collection_id, token_id)) {
                Some(token_info) => token_info.url_storage(),
                _ => None
            }
        }

        /// Returns the symbol of the token, usually a shorter version of the name.
        #[ink(message)]
        pub fn symbol(&self) -> String {
            self.symbols.1.clone()
        }

        /// Returns the balance of the owner.
        ///
        /// This represents the amount of unique tokens the owner has.
        #[ink(message, selector = 0x62616C61)]
        pub fn balance_of(&self, owner: AccountId) -> u64 {
            self.balance_of_or_zero(&owner)
        }

        /// Returns the owner of the token.
        #[ink(message, selector = 0x6F776E65)]
        pub fn owner_of(&self, collection_id: CollectionId, id: TokenId) -> Option<AccountId> {
            if let Some(token_info) = self.token_collection.get(&(collection_id, id)) {
                return Some(token_info.owner());
            }
            None
        }

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        pub fn get_approved(&self, collection_id: CollectionId, id: TokenId) -> Option<AccountId> {
            if let Some(token_info) = self.token_collection.get(&(collection_id, id)) {
                return token_info.approval();
            }
            None
        }

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        pub fn approve(&mut self, to: Option<AccountId>, collection_id: CollectionId, id: TokenId) -> Result<(), Error> {
            self.approve_for(to, collection_id, id)?;
            Ok(())
        }

        /// Transfers the token from the caller to the given destination.
        /// 0x7472616E means tran
        #[ink(message, selector = 0x7472616E)]
        pub fn transfer(
            &mut self,
            destination: AccountId,
            collection_id: CollectionId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &destination, collection_id, id)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        /// 0x74726672 means `tr`ansfer_`fr`om
        #[ink(message, selector = 0x74726672)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            collection_id: CollectionId,
            id: TokenId,
        ) -> Result<(), Error> {
            self.transfer_token_from(&from, &to, collection_id, id)?;
            Ok(())
        }

        /// Creates a new token.
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, collection_id: CollectionId, id: TokenId) -> Result<(), Error> {
            let _ = self.before_transfer(None, Some(to), collection_id, id)?;
            self.add_token_to(&to, collection_id, id)?;
            self.env().emit_event(Minted {
                owner: to,
                collection_id,
                id,
            });
            Ok(())
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        pub fn burn(&mut self, collection_id: CollectionId, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();

            if !self.exists(collection_id, id) {
                self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                return Err(Error::TokenNotFound);
            }

            let token_info = self.token_collection.get(&(collection_id, id)).unwrap();
            if token_info.owner() != caller {
                self.send_error_event(Error::NotOwner, "Caller is not the owner for token. ".to_string());
                return Err(Error::NotOwner);
            }
            self.before_transfer(Some(caller), None, collection_id, id)?;
            let Self {
                owned_tokens_count,
                token_collection,
                ..
            } = self;

            decrease_counter_of(owned_tokens_count, &caller)?;
            token_collection.take(&(collection_id, id));

            self.env().emit_event(Burned {
                caller,
                collection_id,
                id,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn all_token_by_account(&self, account: AccountId) -> Option<Vec<(CollectionId, TokenId)>> {
            self.owned_tokens.get(&account).cloned()
        }

        #[ink(message)]
        pub fn set_token_uri(&mut self, collection_id: CollectionId, token_id: TokenId, uri: String) -> Result<(), Error> {
            let caller = self.env().caller();

            if !self.approved_or_owner(Some(caller), collection_id, token_id) {
                self.send_error_event(Error::NotAllowed, "Caller is not the owner or approval for token. ".to_string());
                return Err(Error::NotAllowed);
            }

            return match self.token_collection.get_mut(&(collection_id, token_id)) {
                Some(token_info) => {
                    token_info.set_url_storage(Some(uri));
                    Ok(())
                }
                None => {
                    self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                    Err(Error::TokenNotFound)
                }
            };
        }
    }

    // Inline methods.
    impl NFT {
        /// Check that transfer can be executed or not.
        #[inline]
        fn before_transfer(&mut self, from: Option<AccountId>,
                           to: Option<AccountId>, collection_id: CollectionId, token_id: TokenId) -> Result<(), Error> {
            if from.is_none() {
                self.add_token_to_token_collection(collection_id, token_id)?;
            } else {
                self.remove_token_from_owner(from.unwrap(), collection_id, token_id)?;
            }

            if let Some(receiver) = to {
                self.add_token_to_owner(receiver, collection_id, token_id);
            }

            Ok(())
        }

        /// Append token_id to token_collection, all properties are default.
        #[inline]
        fn add_token_to_token_collection(&mut self, collection_id: CollectionId, token_id: TokenId) -> Result<(), Error> {
            if self.exists(collection_id, token_id) {
                self.send_error_event(Error::TokenExists, "Token is exists. ".to_string());
                return Err(Error::TokenExists);
            }

            let token_info = Default::default();

            let _ = self.token_collection.insert((collection_id, token_id), token_info);
            Ok(())
        }

        #[inline]
        fn add_token_to_owner(&mut self, to: AccountId, collection_id: CollectionId, token_id: TokenId) {
            let token_index = self.balance_of(to);

            match self.owned_tokens.entry(to) {
                Entry::Occupied(mut o) => {
                    o.get_mut().push((collection_id, token_id));
                }
                Entry::Vacant(v) => {
                    let mut token_vector = Vec::new();
                    token_vector.push((collection_id, token_id));
                    v.insert(token_vector);
                }
            }
            let token_info = self.token_collection.get_mut(&(collection_id, token_id)).unwrap();
            token_info.set_owned_index(token_index);
        }

        #[inline]
        fn remove_token_from_owner(&mut self, from: AccountId, collection_id: CollectionId, token_id: TokenId) -> Result<(), Error> {
            let last_token_index = if self.balance_of(from) == 0 {
                0
            } else {
                self.balance_of(from) - 1
            };

            let token_index = match self.token_collection.entry((collection_id, token_id)) {
                Entry::Occupied(o) => { o.get().owned_index() }
                Entry::Vacant(_) => {
                    self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                    return Err(Error::TokenNotFound);
                }
            };

            // Get the last token info inorder to swap.
            let (last_token_collection_id, last_token_id) = match self.owned_tokens.entry(from) {
                Entry::Occupied(o) => {
                    if o.get().len() < last_token_index as usize {
                        self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                        return Err(Error::TokenNotFound);
                    }
                    o.get()[last_token_index as usize]
                }
                Entry::Vacant(_) => {
                    self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                    return Err(Error::TokenNotFound);
                }
            };

            let last_token_info = match self.token_collection.get_mut(&(last_token_collection_id, last_token_id)) {
                Some(info) => info,
                None => {
                    self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                    return Err(Error::TokenNotFound);
                }
            };

            last_token_info.set_owned_index(token_index);

            match self.owned_tokens.entry(from) {
                Entry::Occupied(mut o) => {
                    if o.get().len() < last_token_index as usize {
                        self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                        return Err(Error::TokenNotFound);
                    }
                    o.get_mut()[token_index as usize] = (last_token_collection_id, last_token_id);
                }
                _ => {
                    self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                    return Err(Error::TokenNotFound);
                }
            };

            Ok(())
        }

        /// Transfers token `id` `from` the sender to the `to` `AccountId`.
        #[inline]
        fn transfer_token_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            collection_id: CollectionId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();

            if !self.approved_or_owner(Some(caller), collection_id, id) {
                self.send_error_event(Error::NotApproved, "Caller is not the owner or approval for token. ".to_string());
                return Err(Error::NotApproved);
            };

            if *to == AccountId::from([0x0; 32]) {
                self.send_error_event(Error::NotAllowed, "Transfer token to zero address is not allowed. ".to_string());
                return Err(Error::NotAllowed);
            };

            if !self.exists(collection_id, id) {
                self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                return Err(Error::TokenNotFound);
            };

            if from == to {
                self.clear_approval(caller, collection_id, id);
                self.env().emit_event(Transfer {
                    from: Some(*from),
                    to: Some(*to),
                    collection_id,
                    id,
                });
                return Ok(());
            }
            let _ = self.before_transfer(Some(*from), Some(*to), collection_id, id)?;

            // TODO: This may be the caller.
            self.clear_approval(*from, collection_id, id);
            self.remove_token_from(from, collection_id, id)?;
            self.add_token_to(to, collection_id, id)?;
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                collection_id,
                id,
            });
            Ok(())
        }

        /// Removes token `id` from the owner.
        #[inline]
        fn remove_token_from(
            &mut self,
            from: &AccountId,
            collection_id: CollectionId,
            id: TokenId,
        ) -> Result<(), Error> {
            let Self {
                token_collection,
                owned_tokens_count,
                ..
            } = self;
            let token_info = token_collection.get_mut(&(collection_id, id)).unwrap();
            token_info.set_owner(AccountId::default());
            decrease_counter_of(owned_tokens_count, from)?;
            let v = self.owned_tokens.get_mut(from).unwrap();
            v.pop();
            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        #[inline]
        fn add_token_to(&mut self, to: &AccountId, collection_id: CollectionId, id: TokenId) -> Result<(), Error> {
            let Self {
                token_collection,
                owned_tokens_count,
                ..
            } = self;

            // Check whether `to` owned token.
            let token_info = token_collection.get_mut(&(collection_id, id)).unwrap();
            if token_info.owner() == *to {
                self.send_error_event(Error::TokenExists, "Token is exists. ".to_string());
                return Err(Error::TokenExists);
            }

            let entry = owned_tokens_count.entry(*to);
            increase_counter_of(entry);
            token_info.set_owner(*to);

            Ok(())
        }

        /// Approve the passed `AccountId` to transfer the specified token on behalf of the message's sender.
        #[inline]
        fn approve_for(&mut self, to: Option<AccountId>, collection_id: CollectionId, id: TokenId) -> Result<(), Error> {
            // Check token exists or not
            if !self.exists(collection_id, id) {
                self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
                return Err(Error::TokenNotFound);
            }

            let caller = self.env().caller();
            let owner = self.owner_of(collection_id, id);
            // Check ownership
            if owner != Some(caller) {
                self.send_error_event(Error::NotAllowed, "Caller is not the owner of token. ".to_string());
                return Err(Error::NotAllowed);
            };

            let approval = if to == Some(AccountId::from([0x0; 32])) {
                None
            } else {
                to
            };

            let token_info = self.token_collection.get_mut(&(collection_id, id)).unwrap();
            token_info.set_approval(approval);

            self.env().emit_event(Approval {
                from: caller,
                to,
                id,
            });
            Ok(())
        }

        /// Removes existing approval from token `id`.
        #[inline]
        fn clear_approval(&mut self, caller: AccountId, collection_id: CollectionId, id: TokenId) {
            let token_info = self.token_collection.get_mut(&(collection_id, id)).unwrap();
            token_info.set_approval(None);
            self.env().emit_event(Approval {
                from: caller,
                to: None,
                id,
            });
        }

        // Returns the total number of tokens from an account.
        #[inline]
        fn balance_of_or_zero(&self, of: &AccountId) -> u64 {
            *self.owned_tokens_count.get(of).unwrap_or(&0)
        }

        /// Returns true if the `AccountId` `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        #[inline]
        fn approved_or_owner(&self, from: Option<AccountId>, collection_id: CollectionId, id: TokenId) -> bool {
            let owner = self.owner_of(collection_id, id);
            let approval = if let Some(token_info) = self.token_collection.get(&(collection_id, id)) {
                token_info.approval()
            } else {
                None
            };
            from != Some(AccountId::from([0x0; 32]))
                && (from == owner
                || from == approval
            )
        }

        /// Returns true if token `id` exists or false if it does not.
        /// TODO: Why need two conditions?
        #[inline]
        fn exists(&self, collection_id: CollectionId, id: TokenId) -> bool {
            self.token_collection.get(&(collection_id, id)).is_some() &&
                self.token_collection.contains_key(&(collection_id, id))
        }

        #[inline]
        fn send_error_event(&self, err: Error, msg: String) {
            self.env().emit_event(ErrorEvent {
                err,
                msg,
            });
        }
    }

    fn decrease_counter_of(
        hmap: &mut StorageHashMap<AccountId, u64>,
        of: &AccountId,
    ) -> Result<(), Error> {
        let count = (*hmap).get_mut(of).ok_or(Error::CannotFetchValue)?;
        *count -= 1;
        Ok(())
    }

    /// Increase token counter from the `of` `AccountId`.
    fn increase_counter_of(entry: Entry<AccountId, u64>) {
        entry.and_modify(|v| *v += 1).or_insert(1);
    }
}
