#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod solar_ft {
    use ink_lang::Env;
    use ink_storage::{
        lazy::Lazy,
        collections::{HashMap, Vec},
    };
    use ink_prelude::string::{String, ToString};

    /// Fungible token contract.
    #[ink(storage)]
    pub struct SolarFT {
        /// Contract deployer
        owner: AccountId,
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned token.
        balances: HashMap<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: HashMap<(AccountId, AccountId), Balance>,
        /// Minter account collections
        minter: Vec<AccountId>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// Event emitted when a error was triggered.
    #[ink(event)]
    pub struct ErrorEvent {
        #[ink(topic)]
        err: Error,
        #[ink(topic)]
        msg: String,
    }

    /// The ERC-20 error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        /// Not any allowance
        NonExistsAllowance,
    }

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl SolarFT {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let mut solar = Self {
                owner: Self::env().caller(),
                total_supply: initial_supply,
                balances: Default::default(),
                allowances: Default::default(),
                minter: Default::default(),
            };
            let caller = Self::env().caller();
            solar.balances.insert(caller, initial_supply);

            solar
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message, selector = 0x62616C61)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_impl(&owner)
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `balance_of` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).cloned().unwrap_or_default()
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(owner, spender)
        }

        /// Allowed administrator to add amount of token.
        #[ink(message)]
        pub fn mint(&mut self, who: AccountId, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            assert!(self.has_mint_role(caller));
            assert!(who != AccountId::from([0u8; 32]));

            self.total_supply += amount;

            let account = self.balances.get_mut(&who);
            assert!(account.is_some());

            *account.unwrap() += amount;

            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0u8; 32])),
                to: Some(who),
                value: amount,
            });

            Ok(())
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `allowance` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn allowance_impl(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).cloned().unwrap_or_default()
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message, selector = 0x7472616E)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn increase_allowance(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();

            let balance = if let Some(origin_allowance) = self.allowances.get(&(owner, spender)) {
                origin_allowance
            } else {
                return Err(Error::NonExistsAllowance);
            };

            let new_value = balance.clone() + value;

            self.allowances.insert((owner, spender), new_value);

            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_value,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn decrease_allowance(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            let balance = if let Some(origin_allowance) = self.allowances.get(&(owner, spender)) {
                origin_allowance
            } else {
                return Err(Error::NonExistsAllowance);
            };

            assert!(balance >= &value);

            let new_value = balance.clone() - value;

            self.allowances.insert((owner, spender), new_value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value: new_value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the account balance of `from`.
        #[ink(message, selector = 0x74726672)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(from, caller);
            if allowance < value {
                self.send_error_event(Error::InsufficientAllowance, "Allowance is not enough. ".to_string());
                return Err(Error::InsufficientAllowance);
            }
            self.transfer_from_to(from, to, value)?;
            self.allowances
                .insert((from, caller), allowance - value);
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(&from);
            if from_balance < value {
                self.send_error_event(Error::InsufficientBalance, "Balance is not enough. ".to_string());
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of_impl(&to);
            self.balances.insert(to, to_balance + value);
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });
            Ok(())
        }

        #[inline]
        fn send_error_event(&self, err: Error, msg: String) {
            self.env().emit_event(ErrorEvent {
                err,
                msg,
            });
        }
    }

    impl SolarFT {
        /// Add `who` to the minter collections.
        fn grant_mint_role(&mut self, who: AccountId) {
            let caller = self.env().caller();
            assert!(caller == self.owner);
            self.minter.push(who);
        }

        /// Check whether `who` has permission to minted.
        fn has_mint_role(&self, who: AccountId) -> bool {
            self.minter.iter().any(|&x| x == who)
        }

        fn renounce_role(&mut self, who: AccountId) {
            let caller = self.env().caller();
            assert!(caller == self.owner);

            let target_index = if let Some(index) = self.minter.iter().position(|value| value == &who) {
                index
            } else {
                return;
            };

            let last_index = self.minter.len() - 1;
            self.minter.swap(target_index as u32, last_index);
            self.minter.pop();
        }
    }
}