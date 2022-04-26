use ink_prelude::string::String;
use ink_storage::{
    traits::{PackedLayout, SpreadLayout},
    collections::Vec,
};
use scale::{Decode, Encode};

#[derive(PackedLayout, SpreadLayout, Decode, Encode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CollectionDetails<SubCollectionID, AccountId> {
    owner: AccountId,
    sub_collection: Vec<SubCollectionID>,
    current_total_count: u64,
    description: Option<String>,
    is_freeze: bool,
    next_sub_collection_id: SubCollectionID,
}

impl<AccountId, SubCollectionID> CollectionDetails<AccountId, SubCollectionID>
    where
        AccountId: Clone,
        SubCollectionID: Clone + Default + PackedLayout
{
    pub fn new(owner: AccountId,
               description: Option<String>,
               next_sub_collection_id: SubCollectionID) -> Self {
        CollectionDetails {
            owner,
            sub_collection: Default::default(),
            current_total_count: 0,
            description,
            is_freeze: false,
            next_sub_collection_id,
        }
    }

    pub fn append_sub_collection(&mut self, sub_collection_id: SubCollectionID) {
        self.sub_collection.push(sub_collection_id);
    }

    pub fn get_sub_collection(&self) -> Vec<SubCollectionID> {
        self.sub_collection
    }

    pub fn sub_collection_exists(&self, sub_collection_id: SubCollectionID) -> bool {
        self.sub_collection.iter().contains(&sub_collection_id)
    }

    pub fn get_sub_collection_id(&self)-> SubCollectionID{
        self.next_sub_collection_id
    }
}