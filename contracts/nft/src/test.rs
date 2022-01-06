/// Unit tests
#[cfg(test)]
mod tests {
    use ink_env::{AccountId, test, call, DefaultEnvironment};
    use ink_lang as ink;
    use crate::nft::NFT;

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

    fn new() -> NFT {
        NFT::new("hello".to_string(), "world".to_string())
    }

    #[ink::test]
    fn mint_and_check_owner() {
        let account_a = AccountId::from([1u8; 32]);
        // let account_b = AccountId::from([2u8; 32]);
        let mut e = new();
        let _ = e.mint(account_a, 1, 123, None);
        let _ = e.mint(account_a, 1, 223, None);
        let _ = e.mint(account_a, 1, 323, None);
        assert_eq!(e.owner_of(1, 223), Some(account_a));
    }

    #[ink::test]
    fn mint_and_transfer() {
        let account_a = AccountId::from([1u8; 32]);
        let account_b = AccountId::from([2u8; 32]);
        let mut e = new();
        let _ = e.mint(account_a, 1, 123, None);
        let _ = e.mint(account_a, 1, 223, None);
        let _ = e.mint(account_a, 1, 323, None);
        let _ = e.transfer_from(account_a, account_b, 1, 323);
        assert_eq!(e.owner_of(1, 323), Some(account_b));
        assert_eq!(e.all_token_by_account(account_a).unwrap(), vec![(1, 123), (1, 223)])
    }

    #[ink::test]
    fn mint_and_transfer_again() {
        let account_a = AccountId::from([1u8; 32]);
        let account_b = AccountId::from([2u8; 32]);
        let mut e = new();
        let _ = e.mint(account_a, 1, 123, None);
        let _ = e.mint(account_a, 1, 223, None);
        let _ = e.mint(account_a, 1, 323, None);
        let _ = e.transfer_from(account_a, account_b, 1, 223);
        assert_eq!(e.owner_of(1, 223), Some(account_b));
        assert_eq!(e.all_token_by_account(account_a).unwrap(), vec![(1, 123), (1, 323)]);
        set_sender(account_b);
        let r = e.transfer_from(account_b, account_a, 1, 223);
        assert!(r.is_ok());
        assert_eq!(e.owner_of(1, 223), Some(account_a));
        assert_eq!(e.all_token_by_account(account_a).unwrap(), vec![(1, 123), (1, 323), (1, 223)]);
    }

    #[ink::test]
    fn mint_and_burn() {
        let account_a = AccountId::from([7u8; 32]);
        let account_b = AccountId::from([8u8; 32]);

        let mut e = new();
        let _ = e.mint(account_a, 1, 123, Some(123.to_string()));
        let _ = e.mint(account_a, 1, 223, Some(223.to_string()));
        let _ = e.mint(account_a, 1, 323, Some(323.to_string()));
        // Sent to account_b
        set_sender(account_a);
        let transfer = e.transfer_from(account_a, account_b, 1, 223);
        assert!(transfer.is_ok());
        let r1 = e.owner_of(1, 223);
        assert_eq!(r1, Some(account_b));

        // Account_b burns
        set_sender(account_b);
        let r = e.burn(1, 223);
        assert!(r.is_ok());

        let r2 = e.owner_of(1, 223);
        assert!(r2.is_none());

        assert_eq!(e.all_token_by_account(account_a), Some(vec![(1, 123), (1, 323)]));

        // Account_a burns.
        set_sender(account_a);
        let r3 = e.burn(1, 123);
        assert!(r3.is_ok());

        assert_eq!(e.all_token_by_account(account_a), Some(vec![(1, 323)]));
    }
}