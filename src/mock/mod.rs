#![cfg(test)]

use super::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_io::TestExternalities;
use sp_runtime::{AccountId32, BoundedVec};
pub mod para;
pub mod relay;
use crate as xnft;
use frame_support::traits::GenesisBuild;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};

pub const ALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u64 = 1_000_000_000;
pub const BOB: AccountId32 = AccountId32::new([1u8; 32]);
pub const CHARLIE: AccountId32 = AccountId32::new([1u8; 32]);
pub const DAVE: AccountId32 = AccountId32::new([1u8; 32]);

pub type Balance = u128;
pub type Amount = i128;

decl_test_parachain! {
	pub struct Para1 {
		Runtime = para::Test,
		XcmpMessageHandler = para::XcmpQueue,
		DmpMessageHandler = para::DmpQueue,
		new_ext = para_ext(4),
	}
}

decl_test_parachain! {
	pub struct Para2 {
		Runtime = para::Test,
		XcmpMessageHandler = para::XcmpQueue,
		DmpMessageHandler = para::DmpQueue,
		new_ext = para_ext(4),
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = relay::Test,
		RuntimeCall = relay::RuntimeCall,
		RuntimeEvent = relay::RuntimeEvent,
		XcmConfig = relay::XcmConfig,
		MessageQueue = relay::MessageQueue,
		System = relay::System,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct TestNet {
		relay_chain = Relay,
		parachains = vec![
			(2000, Para1),
			(2001, Para2),
		],
	}
}
pub type RelayBalances = pallet_balances::Pallet<relay::Test>;
pub type ParaChain1 = xnft::Pallet<para::Test>;
pub type NFT = pallet_nfts::Pallet<para::Test>;
pub fn para_ext(para_id: u32) -> TestExternalities {
	use para::{System, Test};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let parachain_info_config = parachain_info::GenesisConfig { parachain_id: para_id.into() };
	<parachain_info::GenesisConfig as GenesisBuild<Test, _>>::assimilate_storage(
		&parachain_info_config,
		&mut t,
	)
	.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn para_teleport_ext(para_id: u32) -> TestExternalities {
	use para::{System, Test};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let parachain_info_config = parachain_info::GenesisConfig { parachain_id: para_id.into() };
	<parachain_info::GenesisConfig as GenesisBuild<Test, _>>::assimilate_storage(
		&parachain_info_config,
		&mut t,
	)
	.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}


pub fn relay_ext() -> sp_io::TestExternalities {
	use relay::{System, Test};

	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	pallet_balances::GenesisConfig::<Test> { balances: vec![(ALICE, INITIAL_BALANCE)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}