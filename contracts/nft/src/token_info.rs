use ink_env::AccountId;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Encode, Decode};
use ink_prelude::string::String;

#[derive(PackedLayout, Encode, Decode, SpreadLayout, Debug, Default)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct TokenInfo {
    collection: u64,
    owner: AccountId,
    approval: Option<AccountId>,
    owned_index: u64,
    url_storage: Option<String>,
}

impl TokenInfo {
    pub fn set_collection(&mut self, collection: u64) {
        self.collection = collection;
    }
    pub fn set_owner(&mut self, owner: AccountId) {
        self.owner = owner;
    }
    pub fn set_approval(&mut self, approval: Option<AccountId>) {
        self.approval = approval;
    }
    pub fn set_owned_index(&mut self, owned_index: u64) {
        self.owned_index = owned_index;
    }
    pub fn set_url_storage(&mut self, url_storage: Option<String>) {
        self.url_storage = url_storage;
    }
    pub fn collection(&self) -> u64 {
        self.collection
    }
    pub fn owner(&self) -> AccountId {
        self.owner
    }
    pub fn approval(&self) -> Option<AccountId> {
        self.approval
    }
    pub fn owned_index(&self) -> u64 {
        self.owned_index
    }
    pub fn url_storage(&self) -> Option<String> {
        self.url_storage.clone()
    }
}