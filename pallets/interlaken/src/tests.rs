use crate::mock::{
    new_test_ext,
    Interlaken,
    Nft,
    Origin,
    Balances,
};

#[test]
fn mint_token() {
    new_test_ext().execute_with(|| {
        // Mint and set price
        for i in 1..11 {
            let _ = Nft::mint(Origin::root(), 1, None);
            let price = (i + 30) as u64;
            let _ = Interlaken::set_nft_price(Origin::signed(1), i, price);
        }
        assert_eq!(Interlaken::get_nft_price(1), 31);

        // Buy token 1
        let _ = Interlaken::buy(Origin::signed(2), 1, 50);
        assert_eq!(Balances::free_balance(&1), 150);
        assert_eq!(Balances::free_balance(&2), 50);

        // Buy token 5
        let _ = Interlaken::buy(Origin::signed(2), 5, 40);
        assert_eq!(Balances::free_balance(&1), 190);
        assert_eq!(Balances::free_balance(&2), 10);

        // Check balance and owned token
        let balance_1 = Nft::total_of_account(&1);
        assert_eq!(8, balance_1);
        let balance_2 = Nft::total_of_account(&2);
        assert_eq!(2, balance_2);

        // Set price and buy it again
        let set_price_result = Interlaken::set_nft_price(Origin::signed(2), 5, 100);
        assert_eq!(set_price_result, Ok(()));
        let buy_result = Interlaken::buy(Origin::signed(1), 5, 103);
        assert_eq!(buy_result, Ok(()));

        // for i in 0u64..9 {
        //     println!("{:?}", Nft::tokens_for_account(&1, i));
        // }

        // Set token that not for sale
        let _ = Interlaken::set_nft_not_for_sale(Origin::signed(1), 5);

        println!("{:?}", Interlaken::get_all_nft());

        let _ = Interlaken::set_nft_price(Origin::signed(2), 5, 100);

        // Allowed set price for token 2.
        let _ = Nft::approve(Origin::signed(1), Some(2), 5);
        let _ = Interlaken::set_nft_price(Origin::signed(1), 5, 100);
        // println!("{:?}", r);
        assert_eq!(Interlaken::get_nft_price(5), 100);

        let err = Interlaken::set_nft_price(Origin::signed(2), 8, 100);
        assert!(err.is_err());
        assert_eq!(Interlaken::get_nft_price(8), 38);

        let _ = Nft::set_approval_for_all(Origin::signed(1), 2, true);
        let err = Interlaken::set_nft_price(Origin::signed(2), 8, 123);
        assert!(err.is_err());
        assert_eq!(Interlaken::get_nft_price(8), 38);
    });
}