// Tests to be written here

use crate::{mock::*, nft::UniqueAssets, *};
use frame_support::{assert_err, assert_ok};

#[test]
fn mint() {
    new_test_ext().execute_with(|| {
        assert_eq!(SUT::total(), 0);
        assert_eq!(SUT::total_of_account(&1), 0);

        assert_eq!(SUT::total(), 0);
        assert_eq!(SUT::total_of_account(&1), 0);
        assert!(SUT::token_by_id(1).is_none());

        assert_ok!(SUT::mint(Origin::root(), 1, None));

        assert_eq!(SUT::total(), 1, "total 1");
        assert_eq!(SUT::total(), 1, "total 2");
        assert_eq!(SUT::burned(), 0);
        assert_eq!(SUT::burned(), 0);
        assert_eq!(SUT::total_of_account(&1), 1, "total of account 1");
        assert_eq!(SUT::total_of_account(&1), 1, "total of account 2");

        let assets_for_account = SUT::assets_of_account(&1);
        assert_eq!(assets_for_account.len(), 1, "assets for account 1");
        assert_eq!(assets_for_account[0], 1, "assets for account 2");
        //assert_eq!(assets_for_account[0].1, Vec::<u8>::default());
        assert_eq!(SUT::token_by_id(1).map(|t| t.owner), Some(1));
    });
}

#[test]
fn mint2() {
    new_test_ext().execute_with(|| {
        assert_eq!(SUT::total(), 0);
        assert_eq!(SUT::total_of_account(&1), 0);

        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));

        assert_eq!(SUT::total(), 2);
        assert_eq!(SUT::total_of_account(&1), 2);

        let assets_for_account = SUT::assets_of_account(&1);
        assert_eq!(assets_for_account.len(), 2);
        assert_eq!(SUT::owner_of(&assets_for_account[0]), Some(1));
        assert_eq!(SUT::owner_of(&assets_for_account[1]), Some(1));

        let asset1 = SUT::asset_by_account_by_index(&1, 0).unwrap();
        let asset2 = SUT::asset_by_account_by_index(&1, 1).unwrap();

        assert_eq!(asset1, 1);
        assert_eq!(asset2, 2);

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
			SUT::mint(Origin::signed(1), 1, None),
			sp_runtime::DispatchError::BadOrigin
		);
    });
}

/*
#[test]
fn mint_err_dupe() {
	new_test_ext().execute_with(|| {
		assert_ok!(SUT::mint(Origin::root(), 1, None));

		assert_err!(
			SUT::mint(Origin::root(), 2, None),
			Error::<Test>::TokenExists
		);
	});
}
 */

#[test]
fn mint_err_max_user() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));

        assert_err!(
			SUT::mint(Origin::root(), 1, None),
			Error::<Test>::TooManyTokensForAccount
		);
    });
}

#[test]
fn mint_err_max() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::mint(Origin::root(), 3, None));
        assert_ok!(SUT::mint(Origin::root(), 4, None));
        assert_ok!(SUT::mint(Origin::root(), 5, None));

        assert_err!(SUT::mint(Origin::root(), 6, None), Error::<Test>::TooManyTokens);
    });
}

#[test]
fn mint_err_meta() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, Some("meta1".into())));
        assert_ok!(SUT::mint(Origin::root(), 2, Some("meta2".into())));
        assert_err!(
			SUT::mint(Origin::root(), 3, Some(vec![0u8; 1024 * 1024 + 1])),
			Error::<Test>::TooLongMetadata
		);
    });
}

#[test]
fn burn() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_eq!(SUT::total_of_account(&1), 1);

        let assets = SUT::assets_of_account(&(1 as u64));

        assert_ok!(SUT::burn(Origin::signed(1), assets[0]));

        assert_eq!(SUT::total(), 0);
        assert_eq!(SUT::burned(), 1);
        assert_eq!(SUT::total_of_account(&1), 0);
        assert_eq!(SUT::assets_of_account(&1), vec![]);
        assert!(SUT::token_by_id(1).is_none());
    });
}

#[test]
fn burn_err_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));

        assert_err!(
			SUT::burn(Origin::signed(2), 1),
			Error::<Test>::NotTokenOwnerOrApproval
		);
    });
}

#[test]
fn burn_err_not_exist() {
    new_test_ext().execute_with(|| {
        assert_err!(
			SUT::burn(Origin::signed(1), 1),
			Error::<Test>::NotTokenOwnerOrApproval
		);
    });
}

#[test]
fn transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));

        let assets = SUT::assets_of_account(&(1 as u64));

        assert_ok!(SUT::transfer(Origin::signed(1), 2, assets[0]));

        assert_eq!(SUT::total(), 3);
        assert_eq!(SUT::burned(), 0);
        assert_eq!(SUT::total_of_account(&1), 2);
        assert_eq!(SUT::total_of_account(&2), 1);
        assert_eq!(SUT::assets_of_account(&1), vec![3, 1]);
        let assets_for_account = SUT::assets_of_account(&2);
        assert_eq!(assets_for_account.len(), 1);
        assert_eq!(assets_for_account[0], assets[0]);
        //assert_eq!(assets_for_account[0].1, Vec::<u8>::from("test"));
        assert_eq!(SUT::token_by_id(assets_for_account[0]).map(|t| t.owner), Some(2));
    });
}

#[test]
fn transfer_err_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));

        assert_err!(
			SUT::transfer(Origin::signed(0), 2, 1),
			Error::<Test>::NotTokenOwnerOrApproval
		);
    });
}

#[test]
fn transfer_err_not_exist() {
    new_test_ext().execute_with(|| {
        assert_err!(
			SUT::transfer(Origin::signed(1), 2, 1),
			Error::<Test>::NotTokenOwnerOrApproval
		);
    });
}

#[test]
fn transfer_err_max_user() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_eq!(SUT::token_by_id(3).map(|t| t.owner), Some(1));

        assert_err!(
			SUT::transfer(Origin::signed(2), 1, 4),
			Error::<Test>::TooManyTokensForAccount
		);
    });
}

#[test]
fn approve() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::approve(Origin::signed(1), Some(2), 1));
        assert_eq!(SUT::token_by_id(1).map(|t| t.approval).unwrap(), Some(2));
    });
}

#[test]
fn approval() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::set_approval_for_all(Origin::signed(1), 2, true));
        assert_eq!(SUT::approval_for_all(&1, &2), Some(true));
    });
}

#[test]
fn transfer_success_by_approve(){
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::approve(Origin::signed(1), Some(2), 1));

        assert_ok!(SUT::transfer(Origin::signed(2), 2, 1));
        assert_eq!(SUT::total_of_account(&2), 3);
    });
}

#[test]
fn transfer_success_by_approve_all(){
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 1, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::mint(Origin::root(), 2, None));
        assert_ok!(SUT::set_approval_for_all(Origin::signed(1), 2, true));

        assert_ok!(SUT::transfer(Origin::signed(2), 2, 3));
        assert_eq!(SUT::total_of_account(&2), 3);
    });
}