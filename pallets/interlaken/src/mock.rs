#![cfg(test)]

use crate as pallet_interlaken;

use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::{
    BuildStorage,
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
// use frame_support::traits::GenesisBuild;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
            Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
            Nft: pallet_nft::{Pallet, Call, Storage, Config<T>, Event<T>},
            Interlaken: pallet_interlaken::{Pallet, Call, Storage, Event<T>, Config<T>},
        }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxTokenMetaLength: u32 = 32;
    pub const MaxTokens: u128 = 100;
    pub const MaxTokensPerUser: u64 = 100;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

impl pallet_balances::Config for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

// type AccountStore = dyn StorageMap<u64, AccountData<u64>>;

impl pallet_nft::Config for Test {
    type TokenAdmin = frame_system::EnsureRoot<Self::AccountId>;
    type TokenMetaLimit = MaxTokenMetaLength;
    type TokenLimit = MaxTokens;
    type UserTokenLimit = MaxTokensPerUser;
    type Event = ();
}

parameter_types!(
    pub const MaxOwned: u32 = 128;
);
impl pallet_interlaken::Config for Test {
    type Event = ();
    type Currency = Balances;
    type UniqueAssets = Nft;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    GenesisConfig {
        balances: pallet_balances::GenesisConfig {
            balances: vec![(1, 100), (2, 100)]
        },
        nft: pallet_nft::GenesisConfig {
            tokens: vec![]
        },
        ..Default::default()
    }
        .assimilate_storage(&mut t)
        .unwrap();
    t.into()
    // frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}