use ink_env::AccountId;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Encode, Decode};
use ink_prelude::string::String;

type Price = u128;

#[derive(PackedLayout, Encode, Decode, SpreadLayout, Debug, Default, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct TokenInfo {
    owner: AccountId,
    approval: Option<AccountId>,
    metadata: Option<String>,
    is_selling: bool,
    price: Price,
    commission_rate: u16,
}

impl TokenInfo {
    pub fn new(owner: AccountId,
               approval: Option<AccountId>,
               metadata: Option<String>) -> Self {
        TokenInfo { owner, approval, metadata, is_selling: false, price: 0, commission_rate: 0 }
    }
}

impl TokenInfo {
    pub fn set_owner(&mut self, owner: AccountId) {
        self.owner = owner;
    }
    pub fn set_approval(&mut self, approval: Option<AccountId>) {
        self.approval = approval;
    }
    pub fn set_metadata(&mut self, metadata: Option<String>) {
        self.metadata = metadata;
    }
    pub fn set_selling(&mut self, is_selling: bool) {
        self.is_selling = is_selling;
    }
    pub fn set_price(&mut self, price: Price) {
        self.price = price;
    }
    pub fn set_commission_rate(&mut self, fee_percent: u16) {
        self.commission_rate = fee_percent;
    }
    pub fn owner(&self) -> AccountId {
        self.owner
    }
    pub fn approval(&self) -> Option<AccountId> {
        self.approval
    }
    pub fn metadata(&self) -> Option<String> {
        self.metadata.clone()
    }
    pub fn is_selling(&self) -> bool {
        self.is_selling
    }
    pub fn price(&self) -> Price {
        self.price
    }
    pub fn commission_rate(&self) -> u16 {
        self.commission_rate
    }
}