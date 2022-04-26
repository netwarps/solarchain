/// Unit tests
#[cfg(test)]
mod tests {
	use crate::kugga_max::{Error, KuggaMax};
	use ink_env::{
		call,
		hash::{HashOutput, Sha2x256},
		hash_bytes, test, AccountId, DefaultEnvironment,
	};
	use ink_lang as ink;
	use ink_prelude::vec::Vec;
	use scale::Compact;

	fn set_sender(sender: AccountId) {
		let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap();
		test::push_execution_context::<DefaultEnvironment>(
			sender,
			callee,
			1000000,
			1000000,
			test::CallData::new(call::Selector::new([0x00; 4])), // dummy
		);
	}

	fn new() -> KuggaMax {
		KuggaMax::new("hello".to_string(), "world".to_string())
	}

	// #[ink::test]
	// fn mint_and_check_owner() {
	// 	let account_a = AccountId::from([1u8; 32]);
	// 	// let account_b = AccountId::from([2u8; 32]);
	// 	let mut e = new();
	// 	let _ = e.mint(1, None, "Hello".to_string(), 1, 1, Type::Standard, "".to_string());
	// 	let _ = e.mint(2, None, "World".to_string(), 1, 1, Type::Standard, "".to_string());
	// 	let _ = e.mint(3, None, "Hello World".to_string(), 1, 1, Type::Standard, "".to_string());
	// 	assert_eq!(e.owner_of(1), Some(account_a));
	// }

	#[ink::test]
	fn mint_and_transfer() {
		let account_a = AccountId::from([1u8; 32]);
		let account_b = AccountId::from([2u8; 32]);
		let account_c = AccountId::from([3u8; 32]);
		let mut e = new();
		let _ = e.mint(account_a, 1, None);
		let r = e.mint(account_a, 1, None);
		assert_eq!(r, Err(Error::TokenExists));
		let _ = e.mint(account_a, 2, None);
		let _ = e.mint(account_a, 3, None);

		let _ = e.transfer(account_b, 2);
		let token = e.get_token_info(2);
		assert!(token.is_some());
		assert_eq!(token.unwrap().owner(), account_b);

		let r = e.approve(Some(account_c), 2);
		assert_eq!(r, Err(Error::NotApproved));

		set_sender(account_b);
		let _ = e.approve(Some(account_c), 2);
		assert_eq!(e.get_approved(2), Some(account_c));
	}
}
