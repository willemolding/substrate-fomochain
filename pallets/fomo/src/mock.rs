pub use crate as pallet_fomo;
use frame_support::{
    parameter_types,
    traits::{OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        FOMOModule: pallet_fomo::{Module, Call, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

pub(crate) type Balance = u128;

pub(crate) const PRICE_INCREMENT: Balance = 1;
pub(crate) const BLOCKS_TO_WIN: u32 = 10;
pub(crate) const INITIAL_BALANCE: Balance = 100;

parameter_types! {
    pub const MaxLocks: u32 = 1024;
    pub const PriceIncrement: Balance = PRICE_INCREMENT;
    pub const BlocksToWin: u32 = BLOCKS_TO_WIN;
    pub static ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl pallet_fomo::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type PriceIncrement = PriceIncrement;
	type BlocksToWin = BlocksToWin;
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        FOMOModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        FOMOModule::on_initialize(System::block_number());
    }
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    pallet_balances::GenesisConfig::<Test>{
        // Assign initial balances to accounts
        balances: vec![
            (0, INITIAL_BALANCE),
            (1, INITIAL_BALANCE),
            (2, INITIAL_BALANCE)
        ],
    }.assimilate_storage(&mut t).unwrap();
    t.into()
}
