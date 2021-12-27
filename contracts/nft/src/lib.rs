//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!
//! ## Warning
//!
//! This contract is an *example*. It is neither audited nor endorsed for production use.
//! Do **not** rely on it to keep anything of value secure.
//!
//! ## Overview
//!
//! This contract demonstrates how to build non-fungible or unique tokens using ink!.
//!
//! ## Error Handling
//!
//! Any function that modifies the state returns a `Result` type and does not changes the state
//! if the `Error` occurs.
//! The errors are defined as an `enum` type. Any other error or invariant violation
//! triggers a panic and therefore rolls back the transaction.
//!
//! ## Token Management
//!
//! After creating a new token, the function caller becomes the owner.
//! A token can be created, transferred, or destroyed.
//!
//! Token owners can assign other accounts for transferring specific tokens on their behalf.
//! It is also possible to authorize an operator (higher rights) for another account to handle tokens.
//!
//! ### Token Creation
//!
//! Token creation start by calling the `mint(&mut self, id: u64)` function.
//! The token owner becomes the function caller. The Token ID needs to be specified
//! as the argument on this function call.
//!
//! ### Token Transfer
//!
//! Transfers may be initiated by:
//! - The owner of a token
//! - The approved address of a token
//! - An authorized operator of the current owner of a token
//!
//! The token owner can transfer a token by calling the `transfer` or `transfer_from` functions.
//! An approved address can make a token transfer by calling the `transfer_from` function.
//! Operators can transfer tokens on another account's behalf or can approve a token transfer
//! for a different account.
//!
//! ### Token Removal
//!
//! Tokens can be destroyed by burning them. Only the token owner is allowed to burn a token.

#![cfg_attr(not(feature = "std"), no_std)]

mod token_info;

use ink_lang as ink;

pub use self::erc721_extension::{
    Erc721Extension,
    Erc721ExtensionRef,
};

#[ink::contract]
pub mod erc721_extension {
    // #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::collections::{
        hashmap::Entry,
        HashMap as StorageHashMap,
    };
    use ink_storage::lazy::Lazy;
    use ink_prelude::{
        string::String,
        vec::Vec,
    };
    use scale::{
        Decode,
        Encode,
    };
    use crate::token_info::TokenInfo;
    // use ink_env::AccountId;
    // use ink_env::AccountId;

    /// A token ID.
    pub type TokenId = u64;

    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc721Extension {
        /// Owner
        owner: AccountId,
        /// Admin
        admin: AccountId,
        /// Symbols of ERC20 Token, by (name, symbol)
        symbols: Lazy<(String, String)>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: StorageHashMap<AccountId, u64>,
        // /// Mapping from owner to operator approvals.
        // operator_approvals: StorageHashMap<(AccountId, AccountId), bool>,
        /// Mapping from owner to list of owned token IDs
        owned_tokens: StorageHashMap<AccountId, Vec<TokenId>>,
        /// Mapping from token_id to token info, such as owner, approval, etc.
        token_collection: StorageHashMap<TokenId, TokenInfo>,
        // /// Disallow transfers if paused is true.
        // paused: bool,
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

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
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

    impl Erc721Extension {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new(name: String, symbols: String) -> Self {
            Self {
                owner: Self::env().caller(),
                admin: Self::env().caller(),
                symbols: Lazy::new((name, symbols)),
                owned_tokens_count: Default::default(),
                // operator_approvals: Default::default(),
                owned_tokens: Default::default(),
                token_collection: Default::default(),
                // paused: false,
            }
        }

        /// Set the admin for nft contract
        #[ink(message)]
        pub fn set_admin(&mut self, admin: AccountId) {
            assert_eq!(self.env().caller(), self.owner);
            self.admin = admin;
        }

        /// Get the admin for nft contract
        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
            return self.admin;
        }

        /// Returns the name of the token.
        #[ink(message)]
        pub fn name(&self) -> String {
            self.symbols.0.clone()
        }

        /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
        #[ink(message)]
        pub fn token_url(&self, token_id: TokenId) -> Option<String> {
            match self.token_collection.get(&token_id) {
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
        pub fn owner_of(&self, id: TokenId) -> Option<AccountId> {
            if let Some(token_info) = self.token_collection.get(&id) {
                return Some(token_info.owner());
            }
            let a = self.env().account_id();
            Some(a)
            // None
        }

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        pub fn get_approved(&self, id: TokenId) -> Option<AccountId> {
            if let Some(token_info) = self.token_collection.get(&id) {
                return token_info.approval();
            }
            None
        }

        // /// Returns `true` if the operator is approved by the owner.
        // #[ink(message)]
        // pub fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
        //     self.approved_for_all(owner, operator)
        // }

        // /// Approves or disapproves the operator for all tokens of the caller.
        // #[ink(message)]
        // pub fn set_approval_for_all(
        //     &mut self,
        //     to: AccountId,
        //     approved: bool,
        // ) -> Result<(), Error> {
        //     self.approve_for_all(to, approved)?;
        //     Ok(())
        // }

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, id: TokenId) -> Result<(), Error> {
            self.approve_for(&to, id)?;
            Ok(())
        }

        /// Transfers the token from the caller to the given destination.
        /// 0x7472616E means tran
        #[ink(message, selector = 0x7472616E)]
        pub fn transfer(
            &mut self,
            destination: AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &destination, id)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        /// 0x74726672 means `tr`ansfer_`fr`om
        #[ink(message, selector = 0x74726672)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            self.transfer_token_from(&from, &to, id)?;
            Ok(())
        }

        /// Creates a new token.
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, collection_id: u64, id: TokenId) -> Result<(), Error> {
            let _ = self.before_transfer(None, Some(to), id)?;
            self.add_token_to(&to, id, Some(collection_id))?;
            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0x0; 32])),
                to: Some(to),
                id,
            });
            // self.record_uri_by_token_id(id, token_uri)?;
            Ok(())
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        pub fn burn(&mut self, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();

            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            }

            let token_info = self.token_collection.get(&id).unwrap();
            if token_info.owner() != caller {
                return Err(Error::NotOwner);
            }
            self.before_transfer(Some(caller), None, id)?;
            let Self {
                owned_tokens_count,
                token_collection,
                ..
            } = self;

            decrease_counter_of(owned_tokens_count, &caller)?;
            token_collection.take(&id);

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(AccountId::from([0x0; 32])),
                id,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn all_token_by_account(&self, account: AccountId) -> Option<Vec<TokenId>> {
            self.owned_tokens.get(&account).cloned()
        }

        // #[ink(message)]
        // pub fn pause(&mut self) {
        //     self.paused = true;
        // }

        // #[ink(message)]
        // pub fn unpause(&mut self) {
        //     self.paused = false;
        // }

        // #[ink(message)]
        // pub fn paused(&self) -> bool {
        //     self.paused
        // }

        #[ink(message)]
        pub fn set_token_uri(&mut self, token_id: TokenId, uri: String) -> Result<(), Error> {
            let caller = self.env().caller();

            if !self.approved_or_owner(Some(caller), token_id) {
                return Err(Error::NotAllowed);
            }

            return match self.token_collection.get_mut(&token_id) {
                Some(token_info) => {
                    token_info.set_url_storage(Some(uri));
                    Ok(())
                }
                None => Err(Error::TokenNotFound)
            };
        }

        /// Check that transfer can be executed or not.
        #[inline]
        fn before_transfer(&mut self, from: Option<AccountId>,
                           to: Option<AccountId>, token_id: TokenId) -> Result<(), Error> {
            // if self.paused {
            //     return Err(Error::NotAllowed);
            // }

            if from.is_none() {
                self.add_token_to_token_collection(token_id)?;
            } else {
                self.remove_token_from_owner(from.unwrap(), token_id)?;
            }

            if let Some(receiver) = to {
                self.add_token_to_owner(receiver, token_id);
            }

            Ok(())
        }

        /// Append token_id to token_collection, all properties are default.
        #[inline]
        fn add_token_to_token_collection(&mut self, token_id: TokenId) -> Result<(), Error> {
            if self.exists(token_id) {
                return Err(Error::TokenExists);
            }

            let token_info = Default::default();

            let _ = self.token_collection.insert(token_id, token_info);
            Ok(())
        }

        #[inline]
        fn remove_token_from_token_collection(&mut self, token_id: TokenId) {
            let _ = self.token_collection.take(&token_id);
        }

        #[inline]
        fn add_token_to_owner(&mut self, to: AccountId, token_id: TokenId) {
            let token_index = self.balance_of(to);

            match self.owned_tokens.entry(to) {
                Entry::Occupied(mut o) => {
                    o.get_mut().push(token_id);
                }
                Entry::Vacant(v) => {
                    let mut token_vector = Vec::new();
                    token_vector.push(token_id);
                    v.insert(token_vector);
                }
            }
            let token_info = self.token_collection.get_mut(&token_id).unwrap();
            token_info.set_owned_index(token_index);
        }

        #[inline]
        fn remove_token_from_owner(&mut self, from: AccountId, token_id: TokenId) -> Result<(), Error> {
            let last_token_index = if self.balance_of(from) == 0 {
                0
            } else {
                self.balance_of(from) - 1
            };

            if self.token_collection.get(&token_id).is_none() {
                return Err(Error::TokenNotFound);
            }

            let token_index = match self.token_collection.entry(token_id) {
                Entry::Occupied(o) => { o.get().owned_index() }
                Entry::Vacant(_) => return Err(Error::TokenNotFound)
            };

            let last_token_id: TokenId = match self.owned_tokens.entry(from) {
                Entry::Occupied(o) => {
                    if o.get().len() < last_token_index as usize {
                        return Err(Error::TokenNotFound);
                    }
                    o.get()[last_token_index as usize]
                }
                Entry::Vacant(_) => return Err(Error::TokenNotFound)
            };

            let last_token_info = match self.token_collection.get_mut(&last_token_id) {
                Some(info) => info,
                None => return Err(Error::TokenNotFound)
            };

            last_token_info.set_owned_index(token_index);

            match self.owned_tokens.entry(from) {
                Entry::Occupied(mut o) => {
                    if o.get().len() < last_token_index as usize {
                        return Err(Error::TokenNotFound);
                    }
                    o.get_mut()[token_index as usize] = last_token_id;
                }
                _ => return Err(Error::TokenNotFound)
            };

            Ok(())
        }

        /// Transfers token `id` `from` the sender to the `to` `AccountId`.
        #[inline]
        fn transfer_token_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();

            if !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved);
            };

            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };

            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            };

            if from == to {
                self.clear_approval(caller, id);
                self.env().emit_event(Transfer {
                    from: Some(*from),
                    to: Some(*to),
                    id,
                });
                return Ok(());
            }
            let _ = self.before_transfer(Some(*from), Some(*to), id)?;

            // TODO: This may be the caller.
            self.clear_approval(*from, id);
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id, None)?;
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                id,
            });
            Ok(())
        }

        /// Removes token `id` from the owner.
        #[inline]
        fn remove_token_from(
            &mut self,
            from: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let Self {
                token_collection,
                owned_tokens_count,
                ..
            } = self;
            let token_info = token_collection.get_mut(&id).unwrap();
            token_info.set_owner(AccountId::default());
            decrease_counter_of(owned_tokens_count, from)?;
            let v = self.owned_tokens.get_mut(from).unwrap();
            v.pop();
            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        #[inline]
        fn add_token_to(&mut self, to: &AccountId, id: TokenId, collection_id: Option<u64>) -> Result<(), Error> {
            let Self {
                token_collection,
                owned_tokens_count,
                ..
            } = self;

            // Check whether `to` owned token.
            let token_info = token_collection.get_mut(&id).unwrap();
            if token_info.owner() == *to {
                return Err(Error::TokenExists);
            }

            let entry = owned_tokens_count.entry(*to);
            increase_counter_of(entry);
            token_info.set_owner(*to);

            if let Some(coll_id) = collection_id {
                token_info.set_collection(coll_id);
            }

            Ok(())
        }

        /// Records the URI info by TokenId
        #[inline]
        fn record_uri_by_token_id(&mut self, token_id: TokenId, token_url: Option<String>) -> Result<(), Error> {
            let token_info = self.token_collection.get_mut(&token_id).unwrap();
            token_info.set_url_storage(token_url);
            Ok(())
        }

        // /// Approves or disapproves the operator to transfer all tokens of the caller.
        // #[inline]
        // fn approve_for_all(
        //     &mut self,
        //     to: AccountId,
        //     approved: bool,
        // ) -> Result<(), Error> {
        //     let caller = self.env().caller();
        //     self.env().emit_event(ApprovalForAll {
        //         owner: caller,
        //         operator: to,
        //         approved,
        //     });
        //     let key = (caller, to);
        //     if !approved {
        //         self.operator_approvals.take(&key);
        //     } else {
        //         self.operator_approvals.insert(key, approved);
        //     }
        //     Ok(())
        // }

        /// Approve the passed `AccountId` to transfer the specified token on behalf of the message's sender.
        #[inline]
        fn approve_for(&mut self, to: &AccountId, id: TokenId) -> Result<(), Error> {
            // Check token exists or not
            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            }

            let caller = self.env().caller();
            let owner = self.owner_of(id);
            // Check ownership
            if !(owner == Some(caller))
            // || self.approved_for_all(owner.expect("Error with AccountId"), caller))
            {
                return Err(Error::NotAllowed);
            };

            let approval = if *to == AccountId::from([0x0; 32]) {
                None
            } else {
                Some(*to)
            };

            let token_info = self.token_collection.get_mut(&id).unwrap();
            token_info.set_approval(approval);

            self.env().emit_event(Approval {
                from: caller,
                to: Some(*to),
                id,
            });
            Ok(())
        }

        /// Removes existing approval from token `id`.
        #[inline]
        fn clear_approval(&mut self, caller: AccountId, id: TokenId) {
            let token_info = self.token_collection.get_mut(&id).unwrap();
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

        // /// Gets an operator on other Account's behalf.
        // #[inline]
        // fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
        //     *self
        //         .operator_approvals
        //         .get(&(owner, operator))
        //         .unwrap_or(&false)
        // }

        /// Returns true if the `AccountId` `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        #[inline]
        fn approved_or_owner(&self, from: Option<AccountId>, id: TokenId) -> bool {
            if from == Some(self.admin) {
                return true;
            }

            let owner = self.owner_of(id);
            let approval = if let Some(token_info) = self.token_collection.get(&id) {
                token_info.approval()
            } else {
                None
            };
            from != Some(AccountId::from([0x0; 32]))
                && (from == owner
                || from == approval
                //     || self.approved_for_all(
                //     owner.expect("Error with AccountId"),
                //     from.expect("Error with AccountId"),
                // )
            )
        }

        /// Returns true if token `id` exists or false if it does not.
        /// TODO: Why need two conditions?
        #[inline]
        fn exists(&self, id: TokenId) -> bool {
            self.token_collection.get(&id).is_some() && self.token_collection.contains_key(&id)
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

    /// Unit tests
    #[cfg(test)]
    mod tests {}
}
