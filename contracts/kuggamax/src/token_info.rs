use crate::token_info::Type::Standard;
use ink_env::AccountId;
use ink_prelude::string::String;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

#[derive(PackedLayout, SpreadLayout, Encode, Decode, Debug, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Type {
	Standard,
	Memo,
}

impl Default for Type {
	fn default() -> Self {
		Standard
	}
}

// #[derive(PackedLayout, SpreadLayout, Encode, Decode, Debug, Clone)]
// #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
// pub enum VisibleLevel {
//     OnlyBackground,
//     OnlyCreator,
//     OnlyAdmin,
//     OnlyMember,
//     All,
// }

#[derive(PackedLayout, Encode, Decode, SpreadLayout, Debug, Default, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct LabInfo {
	// Token owner.
	owner: AccountId,
	metadata: Option<String>,
}

impl LabInfo {
	pub fn set_owner(&mut self, owner: AccountId) {
		self.owner = owner;
	}
	pub fn set_metadata(&mut self, metadata: Option<String>) {
		self.metadata = metadata;
	}

	pub fn owner(&self) -> AccountId {
		self.owner
	}
	pub fn metadata(&self) -> Option<String> {
		self.metadata.clone()
	}
}

impl LabInfo {}

#[derive(PackedLayout, Encode, Decode, SpreadLayout, Debug, Default, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct TokenInfo {
	// Token owner.
	owner: AccountId,
	// Who can operate this token.
	approval: Option<AccountId>,
	metadata: Option<String>,
	// Token author.
	author: AccountId,
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
	pub fn set_author(&mut self, author: AccountId) {
		self.author = author;
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
}
