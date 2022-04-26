//! This is a KuggaMax contract.

#![cfg_attr(not(feature = "std"), no_std)]

// mod class;
// mod test;
mod token_info;

use ink_lang as ink;

#[ink::contract]
pub mod kugga_max {
	// #[cfg(not(feature = "ink-as-dependency"))]

	use crate::{kugga_max::Error::NotApproved, token_info::TokenInfo};
	use ink_env::{
		hash::{HashOutput, Sha2x256},
		hash_bytes, set_contract_storage,
	};
	use ink_lang::{EmitEvent, Env};
	use ink_prelude::{
		string::{String, ToString},
		vec::Vec,
	};
	use ink_primitives::Key;
	use ink_storage::{collections::HashMap as StorageHashMap, lazy::Lazy};
	use scale::{Decode, Encode};

	pub type ItemId = u64;

	pub type SnapShot = String;

	#[ink(storage)]
	#[derive(Default)]
	pub struct KuggaMax {
		/// Contract owner.
		owner: AccountId,
		/// Symbols of KuggaMax Token, by (name, symbol)
		symbols: Lazy<(String, String)>,
		// /// Mapping from lab_id to
		// lab_collection: StorageHashMap<LabId, LabInfo>,
		/// Mapping from item_id to token info, such as owner, approval, etc.
		token_collection: StorageHashMap<ItemId, TokenInfo>,
		/// SnapShot key.
		key: Key,
		/// SnapShot key to search.
		key_to_string: String,
		/// Current snapshot content.
		current_snapshot: String,
	}

	#[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum Error {
		NotContractOwner,
		/// Token error.
		NotTokenOwner,
		NotApproved,
		TokenExists,
		// TokenURIExists,
		TokenNotFound,
		CannotFetchValue,
		NotAllowed,
		// Decode error.
		Other,
	}

	/// Event emitted when a error was triggered.
	#[ink(event)]
	#[derive(Debug)]
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
		#[ink(topic)]
		item_id: ItemId,
	}

	/// Event emitted when set metadata to token.
	#[ink(event)]
	pub struct SetUrl {
		#[ink(topic)]
		item_id: ItemId,
		metadata: String,
	}

	/// Event emitted when a token minted occurs.
	#[ink(event)]
	pub struct Minted {
		#[ink(topic)]
		owner: AccountId,
		item_id: ItemId,
	}

	/// Event emitted when create a new snapshot.
	#[ink(event)]
	pub struct SnapShotEvent {
		timestamp: u64,
	}

	/// Event emitted when a token burned occurs.
	#[ink(event)]
	pub struct Burned {
		caller: AccountId,
		item_id: ItemId,
	}

	/// Event emitted when a token approve occurs.
	#[ink(event)]
	pub struct Approval {
		#[ink(topic)]
		from: AccountId,
		#[ink(topic)]
		to: Option<AccountId>,
		#[ink(topic)]
		item_id: ItemId,
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

	/// Exec methods
	impl KuggaMax {
		/// Creates a new KuggaMax contract.
		#[ink(constructor)]
		pub fn new(name: String, symbols: String) -> Self {
			let mut output = <Sha2x256 as HashOutput>::Type::default();
			hash_bytes::<Sha2x256>("SnapShot".as_bytes(), &mut output);

			let key = Key::from(output);
			Self {
				owner: Self::env().caller(),
				symbols: Lazy::new((name, symbols)),
				token_collection: Default::default(),
				key,
				key_to_string: "0x0c0e3be0209b7e6b1097bfa586305ecb0e539f5b18bee42a31451c0cf547beef"
					.to_string(),
				current_snapshot: "".to_string(),
			}
		}

		/// Approves the account to transfer the specified token on behalf of the caller.
		#[ink(message)]
		pub fn approve(&mut self, to: Option<AccountId>, item_id: ItemId) -> Result<(), Error> {
			self.approve_for(to, item_id)?;
			Ok(())
		}

		/// Transfers the token from the caller to the given destination.
		/// 0x7472616E means tran
		#[ink(message, selector = 0x7472616E)]
		pub fn transfer(&mut self, destination: AccountId, item_id: ItemId) -> Result<(), Error> {
			let caller = self.env().caller();
			self.transfer_token_from(&caller, &destination, item_id)?;
			Ok(())
		}

		/// Transfer approved or owned token.
		/// 0x74726672 means `tr`ansfer_`fr`om
		#[ink(message, selector = 0x74726672)]
		pub fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			item_id: ItemId,
		) -> Result<(), Error> {
			self.transfer_token_from(&from, &to, item_id)?;
			Ok(())
		}

		#[ink(message)]
		pub fn snapshot(&mut self, value: SnapShot) -> Result<(), Error> {
			let caller = self.env().caller();
			if caller != self.owner {
				self.send_error_event(
					Error::NotApproved,
					"Only admin can create snapshot. ".to_string(),
				);
				return Err(NotApproved)
			}

			set_contract_storage(&self.key, &value);
			self.current_snapshot = value.clone();
			let timestamp = self.env().block_timestamp();
			self.env().emit_event(SnapShotEvent { timestamp });
			Ok(())
		}

		// #[ink(message)]
		pub fn get_snapshot(&self, hex_string: String) -> Result<String, Error> {
			let v = hex_string[2..].as_bytes();
			let mut encode_u8 = Vec::new();
			let mut buf_1 = 0;
			let mut buf_2 = 0;

			for i in 0..v.len() / 2 {
				match v[2 * i] {
					b'A'..=b'F' => buf_1 = v[2 * i] - b'A' + 10,
					b'a'..=b'f' => buf_1 = v[2 * i] - b'a' + 10,
					b'0'..=b'9' => buf_1 = v[2 * i] - b'0',
					_ => {},
				}
				match v[2 * i + 1] {
					b'A'..=b'F' => buf_2 = v[2 * i + 1] - b'A' + 10,
					b'a'..=b'f' => buf_2 = v[2 * i + 1] - b'a' + 10,
					b'0'..=b'9' => buf_2 = v[2 * i + 1] - b'0',
					_ => {},
				}
				encode_u8.push((buf_1 << 4) + buf_2);

				buf_1 = 0;
				buf_2 = 0;
			}
			// let origin_string: Result<String, scale::Error> =
			scale::Decode::decode(&mut &encode_u8[..]).map_err(|_| Error::Other)
		}

		/// Creates a new token.
		#[ink(message)]
		pub fn mint(
			&mut self,
			to: AccountId,
			item_id: ItemId,
			metadata: Option<String>,
		) -> Result<(), Error> {
			if self.token_exists(item_id) {
				self.send_error_event(Error::TokenExists, "Token is exists. ".to_string());
				return Err(Error::TokenExists)
			}

			let caller = self.env().caller();
			if caller != self.owner {
				self.send_error_event(
					Error::NotContractOwner,
					"Only admin can minted in KuggaMax. ".to_string(),
				);
				return Err(Error::NotContractOwner)
			}

			let mut token_info: TokenInfo = Default::default();
			token_info.set_author(caller);
			let _ = self.token_collection.insert(item_id, token_info);

			self.add_token_to(&to, item_id, metadata)?;

			self.env().emit_event(Minted { owner: to, item_id });
			Ok(())
		}

		/// Deletes an existing token. Only the owner can burn the token.
		#[ink(message)]
		pub fn burn(&mut self, item_id: ItemId) -> Result<(), Error> {
			if !self.token_exists(item_id) {
				self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
				return Err(Error::TokenNotFound)
			}

			let caller = self.env().caller();
			let token_info = self.token_collection.get(&item_id).unwrap();
			let token_owner = token_info.owner();
			if token_owner != caller {
				self.send_error_event(
					Error::NotTokenOwner,
					"Caller is not the owner for token. ".to_string(),
				);
				return Err(Error::NotTokenOwner)
			}

			let Self { token_collection, .. } = self;
			token_collection.take(&item_id);
			self.env().emit_event(Burned { caller, item_id });
			Ok(())
		}
	}

	/// Read methods.
	impl KuggaMax {
		/// Returns the name of the token.
		#[ink(message)]
		pub fn name(&self) -> String {
			self.symbols.0.clone()
		}

		/// Returns the symbol of the token, usually a shorter version of the name.
		#[ink(message)]
		pub fn symbol(&self) -> String {
			self.symbols.1.clone()
		}

		#[ink(message)]
		pub fn snapshot_key(&self) -> String {
			self.key_to_string.clone()
		}

		/// Returns the approved account ID for this token if any.
		#[ink(message)]
		pub fn get_approved(&self, item_id: ItemId) -> Option<AccountId> {
			if let Some(token_info) = self.token_collection.get(&item_id) {
				return token_info.approval()
			}
			None
		}

		#[ink(message)]
		pub fn get_token_info(&self, item_id: ItemId) -> Option<TokenInfo> {
			if !self.token_exists(item_id) {
				self.send_error_event(Error::TokenNotFound, "Token is not found. ".to_string());
				return None
			}
			self.token_collection.get(&item_id).cloned()
		}

		pub fn set_token_url(&mut self, item_id: ItemId, metadata: String) -> Result<(), Error> {
			if !self.token_exists(item_id) {
				self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
				return Err(Error::TokenNotFound)
			}

			let caller = self.env().caller();
			if !self.approved_or_owner(Some(caller), item_id) {
				self.send_error_event(
					Error::NotApproved,
					"Caller is not the owner or approval for token. ".to_string(),
				);
				return Err(Error::NotApproved)
			}

			return match self.token_collection.get_mut(&item_id) {
				Some(token_info) => {
					token_info.set_metadata(Some(metadata.clone()));
					self.env().emit_event(SetUrl { item_id, metadata });
					Ok(())
				},
				None => {
					self.send_error_event(
						Error::TokenNotFound,
						"Token is not exists. ".to_string(),
					);
					Err(Error::TokenNotFound)
				},
			}
		}
	}

	// Inline methods.
	impl KuggaMax {
		#[inline]
		fn owner_of(&self, item_id: ItemId) -> Option<AccountId> {
			if let Some(token_info) = self.token_collection.get(&item_id) {
				return Some(token_info.owner())
			}
			None
		}

		/// Transfers token `id` `from` the sender to the `to` `AccountId`.
		#[inline]
		fn transfer_token_from(
			&mut self,
			from: &AccountId,
			to: &AccountId,
			item_id: ItemId,
		) -> Result<(), Error> {
			if from == to {
				self.send_error_event(Error::NotAllowed, "Cannot transfer to self. ".to_string());
				return Err(Error::NotAllowed)
			}

			let caller = self.env().caller();
			if !self.token_exists(item_id) {
				self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
				return Err(Error::TokenNotFound)
			};

			if !self.approved_or_owner(Some(caller), item_id) {
				self.send_error_event(
					Error::NotApproved,
					"Caller is not the owner or approval for token. ".to_string(),
				);
				return Err(Error::NotApproved)
			};

			if *to == AccountId::from([0x0; 32]) {
				self.send_error_event(
					Error::NotAllowed,
					"Transfer token to zero address is not allowed. ".to_string(),
				);
				return Err(Error::NotAllowed)
			};

			self.clear_approval(*from, item_id);
			self.remove_token_from(item_id)?;
			self.add_token_to(to, item_id, None)?;
			self.env().emit_event(Transfer { from: Some(*from), to: Some(*to), item_id });
			Ok(())
		}

		/// Removes token `id` from the owner.
		#[inline]
		fn remove_token_from(&mut self, item_id: ItemId) -> Result<(), Error> {
			let Self { token_collection, .. } = self;
			let token_info = token_collection.get_mut(&item_id).unwrap();
			token_info.set_owner(AccountId::default());
			Ok(())
		}

		/// Adds the token `id` to the `to` AccountID.
		#[inline]
		fn add_token_to(
			&mut self,
			to: &AccountId,
			item_id: ItemId,
			metadata: Option<String>,
		) -> Result<(), Error> {
			let token_info = self.token_collection.get_mut(&item_id).unwrap();
			if token_info.owner() == *to {
				self.send_error_event(Error::TokenExists, "Token is exists. ".to_string());
				return Err(Error::TokenExists)
			}
			token_info.set_owner(*to);
			if metadata.is_some() {
				token_info.set_metadata(metadata);
			}

			Ok(())
		}

		/// Approve the passed `AccountId` to transfer the specified token on behalf of the
		/// message's sender.
		#[inline]
		fn approve_for(&mut self, to: Option<AccountId>, item_id: ItemId) -> Result<(), Error> {
			// Check token exists or not
			if !self.token_exists(item_id) {
				self.send_error_event(Error::TokenNotFound, "Token is not exists. ".to_string());
				return Err(Error::TokenNotFound)
			}

			let caller = self.env().caller();
			let owner = self.owner_of(item_id);
			// Check ownership
			if owner != Some(caller) {
				self.send_error_event(
					Error::NotApproved,
					"Caller is not the owner of token. ".to_string(),
				);
				return Err(Error::NotApproved)
			};

			let token_info = self.token_collection.get_mut(&item_id).unwrap();
			let approval = if to == Some(AccountId::from([0x0; 32])) { None } else { to };
			token_info.set_approval(approval);

			self.env().emit_event(Approval { from: caller, to, item_id });
			Ok(())
		}

		/// Removes existing approval from token `id`.
		#[inline]
		fn clear_approval(&mut self, caller: AccountId, item_id: ItemId) {
			let token_info = self.token_collection.get_mut(&item_id).unwrap();
			token_info.set_approval(None);
			self.env().emit_event(Approval { from: caller, to: None, item_id });
		}

		/// Returns true if the `AccountId` `from` is the owner of token `id`
		/// or it has been approved on behalf of the token `id` owner.
		#[inline]
		fn approved_or_owner(&self, from: Option<AccountId>, item_id: ItemId) -> bool {
			let owner = self.owner_of(item_id);
			let approval = if let Some(token_info) = self.token_collection.get(&item_id) {
				token_info.approval()
			} else {
				None
			};
			from != Some(AccountId::from([0x0; 32])) && (from == owner || from == approval)
		}

		/// Returns true if token `id` exists or false if it does not.
		#[inline]
		fn token_exists(&self, item_id: ItemId) -> bool {
			self.token_collection.get(&item_id).is_some() &&
				self.token_collection.contains_key(&item_id)
		}

		// #[inline]
		// fn lab_exists(&self, lab_id: LabId) -> bool {
		// 	self.lab_collection.get(&lab_id).is_some() && self.lab_collection.contains_key(&lab_id)
		// }

		/// A method that using to send error event.
		#[inline]
		fn send_error_event(&self, err: Error, msg: String) {
			self.env().emit_event(ErrorEvent { err, msg });
		}
	}
}
