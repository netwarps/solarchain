// Tests to be written here

use crate::{mock::*, nft::UniqueAssets, *};
use frame_support::{assert_err, assert_ok, Hashable};
use sp_core::H256;

#[test]
fn mint() {
	new_test_ext().execute_with(|| {
		assert_eq!(SUT::total(), 0);
		assert_eq!(SUT::total_of_account(1), 0);

		assert_eq!(SUT::total(), 0);
		assert_eq!(SUT::total_of_account(&1), 0);
		assert_eq!(SUT::token_by_id::<H256>(Vec::<u8>::default().blake2_256().into()).owner, 0);

		assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default(), None));

		assert_eq!(SUT::total(), 1);
		assert_eq!(SUT::total(), 1);
		assert_eq!(SUT::burned(), 0);
		assert_eq!(SUT::burned(), 0);
		assert_eq!(SUT::total_of_account(1), 1);
		assert_eq!(SUT::total_of_account(&1), 1);

		let assets_for_account = SUT::assets_of_account(&1);
		assert_eq!(assets_for_account.len(), 1);
		assert_eq!(assets_for_account[0], Vec::<u8>::default().blake2_256().into());
		//assert_eq!(assets_for_account[0].1, Vec::<u8>::default());
		assert_eq!(SUT::token_by_id::<H256>(Vec::<u8>::default().blake2_256().into()).owner, 1);
	});
}

#[test]
fn mint2() {
	new_test_ext().execute_with(|| {
		assert_eq!(SUT::total(), 0);
		assert_eq!(SUT::total_of_account(&1), 0);

		let v1 = vec![1u8];
		let v2 = vec![2u8];

		assert_ok!(SUT::mint(Origin::root(), 1, v1.clone(), None));
		assert_ok!(SUT::mint(Origin::root(), 1, v2.clone(), None));

		assert_eq!(SUT::total(), 2);
		assert_eq!(SUT::total_of_account(&1), 2);

		let assets_for_account = SUT::assets_of_account(&1);
		assert_eq!(assets_for_account.len(), 2);
		assert_eq!(SUT::owner_of(&assets_for_account[0]), 1);
		assert_eq!(SUT::owner_of(&assets_for_account[1]), 1);

		let asset1 = SUT::asset_by_account_by_index(&1, 0).unwrap();
		let asset2 = SUT::asset_by_account_by_index(&1, 1).unwrap();

		assert_eq!(asset1, v1.blake2_256().into());
		assert_eq!(asset2, v2.blake2_256().into());

		// do a transfer
		assert_ok!(SUT::transfer(Origin::signed(1), 2, assets_for_account[1]));
		let assets_for_account2 = SUT::assets_of_account(&2);
		assert_eq!(assets_for_account2.len(), 1);
		assert_eq!(assets_for_account2[0], assets_for_account[1]);
	});
}

#[test]
fn mint_err_non_admin() {
	new_test_ext().execute_with(|| {
		assert_err!(
			SUT::mint(Origin::signed(1), 1, Vec::<u8>::default(), None),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

#[test]
fn mint_err_dupe() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default(), None));

		assert_err!(
			SUT::mint(Origin::root(), 2, Vec::<u8>::default(), None),
			Error::<Test>::TokenExists
		);
	});
}

#[test]
fn mint_err_max_user() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, vec![], None));
		assert_ok!(SUT::mint(Origin::root(), 1, vec![0], None));

		assert_err!(
			SUT::mint(Origin::root(), 1, vec![1], None),
			Error::<Test>::TooManyTokensForAccount
		);
	});
}

#[test]
fn mint_err_max() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, vec![], None));
		assert_ok!(SUT::mint(Origin::root(), 2, vec![0], None));
		assert_ok!(SUT::mint(Origin::root(), 3, vec![1], None));
		assert_ok!(SUT::mint(Origin::root(), 4, vec![2], None));
		assert_ok!(SUT::mint(Origin::root(), 5, vec![3], None));

		assert_err!(SUT::mint(Origin::root(), 6, vec![4], None), Error::<Test>::TooManyTokens);
	});
}

#[test]
fn mint_err_meta() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, vec![], Some("meta1".into())));
		assert_ok!(SUT::mint(Origin::root(), 2, vec![0], Some("meta2".into())));
		assert_err!(
			SUT::mint(Origin::root(), 3, vec![1], Some("012345678901234567890123456789012".into())),
			Error::<Test>::TooLongMetadata
		);
	});
}

#[test]
fn burn() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::from("test"), None));
		assert_eq!(SUT::total_of_account(&1), 1);

		let assets = SUT::assets_of_account(&(1 as u64));

		assert_ok!(SUT::burn(Origin::signed(1), assets[0]));

		assert_eq!(SUT::total(), 0);
		assert_eq!(SUT::burned(), 1);
		assert_eq!(SUT::total_of_account(&1), 0);
		assert_eq!(SUT::assets_of_account(&1), vec![]);
		assert_eq!(SUT::token_by_id::<H256>(Vec::<u8>::default().blake2_256().into()).owner, 0);
	});
}

#[test]
fn burn_err_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default(), None));

		assert_err!(
			SUT::burn(Origin::signed(2), Vec::<u8>::default().blake2_256().into()),
			Error::<Test>::NotTokenOwner
		);
	});
}

#[test]
fn burn_err_not_exist() {
	new_test_ext().execute_with(|| {
		assert_err!(
			SUT::burn(Origin::signed(1), Vec::<u8>::default().blake2_256().into()),
			Error::<Test>::NotTokenOwner
		);
	});
}

#[test]
fn transfer() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, "test".into(), None));

		let assets = SUT::assets_of_account(&(1 as u64));

		assert_ok!(SUT::transfer(Origin::signed(1), 2, assets[0]));

		assert_eq!(SUT::total(), 1);
		assert_eq!(SUT::burned(), 0);
		assert_eq!(SUT::total_of_account(&1), 0);
		assert_eq!(SUT::total_of_account(&2), 1);
		assert_eq!(SUT::assets_of_account(&1), vec![]);
		let assets_for_account = SUT::assets_of_account(&2);
		assert_eq!(assets_for_account.len(), 1);
		assert_eq!(assets_for_account[0], assets[0]);
		//assert_eq!(assets_for_account[0].1, Vec::<u8>::from("test"));
		assert_eq!(SUT::token_by_id::<H256>(assets_for_account[0]).owner, 2);
	});
}

#[test]
fn transfer_err_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default(), None));

		assert_err!(
			SUT::transfer(Origin::signed(0), 2, Vec::<u8>::default().blake2_256().into()),
			Error::<Test>::NotTokenOwner
		);
	});
}

#[test]
fn transfer_err_not_exist() {
	new_test_ext().execute_with(|| {
		assert_err!(
			SUT::transfer(Origin::signed(1), 2, Vec::<u8>::default().blake2_256().into()),
			Error::<Test>::NotTokenOwner
		);
	});
}

#[test]
fn transfer_err_max_user() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, vec![0], None));
		assert_ok!(SUT::mint(Origin::root(), 1, vec![1], None));
		assert_ok!(SUT::mint(Origin::root(), 2, Vec::<u8>::default(), None));
		assert_eq!(SUT::token_by_id::<H256>(Vec::<u8>::default().blake2_256().into()).owner, 2);

		assert_err!(
			SUT::transfer(Origin::signed(2), 1, Vec::<u8>::default().blake2_256().into()),
			Error::<Test>::TooManyTokensForAccount
		);
	});
}
