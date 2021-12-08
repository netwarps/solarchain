use crate::mock::{
    new_test_ext,
    Interlaken,
    Nft,
    Origin
};

#[test]
fn mint_token() {
    new_test_ext().execute_with(|| {
        let v1 = vec![1u8];
        // let r = Interlaken::create_nft(Origin::signed(1), 30, Vec::<u8>::default(), None).unwrap();
        // println!("{:?}", r);
        // assert_eq!(Interlaken::total(), 1);
        // assert_eq!(Interlaken::get_nft_price(r), 30)

        let token = Nft::mint(Origin::root(), Origin::signed(1), Vec::<u8>::default(), None);
        Interlaken::set_nft_price(token, 30);
        assert_eq!(Interlaken::get_nft_price(token), 30);
    });
}