#![cfg_attr(not(feature = "std"), no_std)]
#![feature(saturating_div)]

use ink_lang as ink;

mod token_info;

#[ink::contract]
mod solar_nft_market {
    use ink_lang::{Env, EmitEvent};
    use ink_env::{call::{build_call, ExecutionInput, Selector, utils::ReturnType},
                  DefaultEnvironment,
    };
    use ink_prelude::string::{String, ToString};
    use ink_storage::collections::{HashMap, Vec as StorageVec};

    use crate::token_info::TokenInfo;

    type TokenID = u64;

    #[ink(storage)]
    pub struct SolarNFTMarket {
        /// Owner
        owner: AccountId,
        /// FT account
        ft: AccountId,
        /// All tokens.
        token: HashMap<TokenID, TokenInfo>,
        /// Who can mint NFT.
        minter: StorageVec<AccountId>,
        /// Who can profit from trade or mint.
        organiser: AccountId,
        /// Mint fee.
        mint_tax_commission_rate: u16,
        /// Increased token id.
        global_token_id: TokenID,
        /// Default trade fee
        trade_commission: u16,
    }

    #[ink(event)]
    pub struct Transfer {
        from: AccountId,
        to: AccountId,
        token_id: TokenID,
    }

    #[ink(event)]
    pub struct Approval {
        owner: AccountId,
        approval: Option<AccountId>,
        token_id: TokenID,
    }

    #[ink(event)]
    pub struct MakeOffer {
        operator: AccountId,
        token_id: TokenID,
        selling_price: Balance,
        commission: u16,
    }

    #[ink(event)]
    pub struct UpdateOffer {
        operator: AccountId,
        token_id: TokenID,
        selling_price: Balance,
        commission: u16,
    }

    #[ink(event)]
    pub struct CancelOffer {
        operator: AccountId,
        token_id: TokenID,
    }

    #[ink(event)]
    pub struct MintItem {
        to: AccountId,
        token_id: TokenID,
        token_uri: String,
    }

    #[ink(event)]
    pub struct Trade {
        buyer: AccountId,
        seller: AccountId,
        token: TokenID,
        price: Balance,
        commission_rate: u16,
        seller_income: Balance,
        organiser_income: Balance,
    }

    #[ink(event)]
    pub struct Error {
        msg: String,
    }

    /// Market
    impl SolarNFTMarket {
        #[ink(constructor)]
        pub fn new(ft: AccountId, organiser: AccountId, mint_tax_commission_rate: u16) -> Self {
            Self {
                owner: Self::env().caller(),
                ft,
                token: Default::default(),
                minter: Default::default(),
                organiser,
                mint_tax_commission_rate,
                global_token_id: 0,
                trade_commission: 20,
            }
        }

        /// NFT trading.
        #[ink(message)]
        pub fn trade_nft(&mut self, token_id: TokenID, bid: Balance) {
            let buyer = self.env().caller();

            let (price, owner, commission_rate) = if let Some(token) = self.token.get(&token_id) {
                if !token.is_selling() {
                    self.send_error_event("Token is not selling. ".to_string());
                    return;
                }
                (token.price(), token.owner(), token.commission_rate())
            } else {
                return;
            };

            // Not allowed buy self owned token.
            if owner == buyer {
                self.send_error_event("Cannot buy token from self. ".to_string());
                return;
            }

            if price > bid {
                self.send_error_event("Purchasing price is less than selling price.".to_string());
                return;
            }

            let commission = price.saturating_mul(commission_rate as u128)
                .saturating_div(100);

            // Transfer fungible token to organiser and owner.
            assert!(self.transfer_ft(buyer, self.organiser, commission));
            assert!(self.transfer_ft(buyer, owner, price - commission));

            self.transfer_inner(buyer, token_id);

            self.env().emit_event(Trade {
                buyer,
                seller: owner,
                token: token_id,
                price,
                commission_rate,
                seller_income: price - commission,
                organiser_income: commission,
            })
        }

        /// Offered a specific token with price.
        #[ink(message)]
        pub fn offer(&mut self, token_id: TokenID, price: Balance) {
            assert!(self.has_permission(token_id));

            let owner = self.env().caller();
            let approval = self.env().account_id();
            let commission = self.trade_commission;
            if let Some(token) = self.token.get_mut(&token_id) {
                // Complete token info.
                token.set_price(price);
                token.set_commission_rate(commission);
                token.set_approval(Some(approval));

                if token.is_selling() {
                    Self::env().emit_event(UpdateOffer {
                        operator: owner,
                        token_id,
                        selling_price: price,
                        commission,
                    })
                } else {
                    Self::env().emit_event(MakeOffer {
                        operator: owner,
                        token_id,
                        selling_price: price,
                        commission,
                    })
                }
                token.set_selling(true);
            }
        }

        /// Cancel offer by token_id.
        #[ink(message)]
        pub fn cancel_offer(&mut self, token_id: TokenID) {
            assert!(self.has_permission(token_id));

            if let Some(token) = self.token.get_mut(&token_id) {
                token.set_selling(false);
                Self::env().emit_event(CancelOffer { operator: self.env().caller(), token_id })
            }
        }
    }

    /// NFT
    impl SolarNFTMarket {
        pub fn has_permission(&self, token_id: TokenID) -> bool {
            let caller = self.env().caller();
            if let Some(token) = self.token.get(&token_id) {
                return token.owner() == caller || token.approval() == Some(caller);
            }
            self.send_error_event("Token is not found.".to_string());
            false
        }

        /// Approve or cancel someone to operate token.
        #[ink(message)]
        pub fn approval(&mut self, token_id: TokenID, approval: Option<AccountId>) {
            let caller = self.env().caller();
            if let Some(token) = self.token.get_mut(&token_id) {
                assert!(token.owner() == caller);
                assert!(!token.is_selling());

                token.set_approval(approval);
                self.env().emit_event(Approval {
                    owner: caller,
                    approval,
                    token_id,
                });
            }
            self.send_error_event("Token is not found.".to_string());
            assert!(false);
        }

        /// Transfer token.
        #[ink(message)]
        pub fn transfer(&mut self, buyer: AccountId, token_id: TokenID) {
            assert!(self.has_permission(token_id));
            assert!(!self.token.get(&token_id).unwrap().is_selling());

            self.transfer_inner(buyer, token_id);
        }

        /// Burn token.
        #[ink(message)]
        pub fn burn(&mut self, token_id: TokenID) {
            assert!(self.has_permission(token_id));

            let mut token = self.token.take(&token_id).unwrap();

            let owner = token.owner();

            token.set_approval(None);
            token.set_owner(AccountId::from([0u8; 32]));

            self.env().emit_event(Transfer {
                from: owner,
                to: AccountId::from([0u8; 32]),
                token_id,
            })
        }

        #[ink(message)]
        pub fn get_token(&self, token_id: TokenID) -> Option<TokenInfo> {
            self.token.get(&token_id).cloned()
        }

        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, token_uri: String) {
            let caller = self.env().caller();
            assert!(self.has_mint_role(caller));

            let approval = self.env().account_id();

            let token = TokenInfo::new(to, Some(approval), Some(token_uri.clone()));

            let token_id = self.global_token_id;
            self.token.insert(token_id, token);

            self.env().emit_event(Transfer {
                from: AccountId::from([0u8; 32]),
                to,
                token_id,
            });

            self.global_token_id += 1;
            self.env().emit_event(MintItem {
                to,
                token_id,
                token_uri,
            })
        }
    }

    /// Role permission
    impl SolarNFTMarket {
        /// Add `who` to the minter collections, now only contract owner can
        /// call this method.
        #[ink(message)]
        pub fn grant_mint_role(&mut self, who: AccountId) {
            let caller = self.env().caller();
            assert!(caller == self.owner);
            if self.has_mint_role(who) {
                return;
            }
            self.minter.push(who);
        }

        /// Check whether `who` has permission to minted.
        #[ink(message)]
        pub fn has_mint_role(&self, who: AccountId) -> bool {
            who == self.owner || self.minter.iter().any(|&x| x == who)
        }

        /// Move `who` from mint role.
        #[ink(message)]
        pub fn renounce_mint_role(&mut self, who: AccountId) {
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

    /// Inner methods.
    impl SolarNFTMarket {
        fn transfer_ft(&mut self, from: AccountId, to: AccountId, price: Balance) -> bool {
            if let Ok(Ok(())) = build_call::<DefaultEnvironment>()
                .callee(self.ft)
                .exec_input(
                    ExecutionInput::new(Selector::new([0x74, 0x72, 0x66, 0x72]))
                        .push_arg(from)
                        .push_arg(to)
                        .push_arg(price)
                )
                .returns::<ReturnType<Result<(), Error>>>()
                .fire() {
                return true;
            }
            self.send_error_event("Calling ft failed. ".to_string());
            false
        }

        fn send_error_event(&self, msg: String) {
            self.env().emit_event(Error { msg });
        }

        fn transfer_inner(&mut self, to: AccountId, token_id: TokenID) {
            let token = self.token.get_mut(&token_id).unwrap();
            let seller = token.owner();

            token.set_owner(to);
            token.set_approval(None);
            token.set_price(0);
            token.set_selling(false);

            self.env().emit_event(Transfer {
                from: seller,
                to,
                token_id,
            })
        }
    }
}