#![cfg(test)]
use crate::{
	self as pallet_xnft,
	test::relay::currency::{CENTS, MILLICENTS},
};
use frame_support::traits::HandleMessage;
use cumulus_primitives_core::relay_chain::CandidateHash;
use sp_runtime::BuildStorage;
use frame_support::traits::TransformOrigin;
use sp_runtime::traits::IdentityLookup;
use primitives::AccountIndex;
use frame_support::traits::QueueFootprint;
use frame_support::{assert_ok, traits::AsEnsureOriginWithArg};
use sp_core::bounded_vec::BoundedSlice;
use polkadot_runtime_common::{
	paras_sudo_wrapper,
	xcm_sender::{ChildParachainRouter, ExponentialPrice},
};
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use primitives::CoreIndex;
use frame_support::pallet_prelude::ValueQuery;
use sp_std::collections::vec_deque::VecDeque;
use frame_support::pallet_prelude::OptionQuery;
use frame_support::derive_impl;
pub use polkadot_runtime_parachains::hrmp;
use polkadot_runtime_parachains::{
	dmp as parachains_dmp, schedule_para_initialize,
};
use frame_support::{
	construct_runtime,
	parameter_types,
	traits::{
		ConstU32, ConstU64, Currency, Everything, Nothing,
	},
	weights::Weight,
};
use xcm_simulator::UmpQueueId;
use crate::test::para::ParachainSystem;
use polkadot_parachain_primitives::primitives::ValidationCode;
use sp_runtime::traits::AccountIdConversion;
use crate::test::relay::currency::DOLLARS;
use cumulus_primitives_core::{
	relay_chain::{AuthorityDiscoveryId, SessionIndex, ValidatorIndex},
	ParaId,
};
use frame_support::traits::ValidatorSetWithIdentification;
use frame_system::EnsureRoot;
use sp_runtime::{transaction_validity::TransactionPriority, Permill};
use std::{cell::RefCell, collections::HashMap};
pub mod currency {
	pub type Balance = u64;

	pub const MILLICENTS: Balance = 1_000_000_000;
	pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
	pub const DOLLARS: Balance = 100 * CENTS;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}
use sp_runtime::
	MultiSignature
;
pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as sp_runtime::traits::Verify>::Signer;
pub type AccountId = <AccountPublic as sp_runtime::traits::IdentifyAccount>::AccountId;
use frame_support::traits::ValidatorSet;
use xcm_builder::{EnsureXcmOrigin, NativeAsset};
use pallet_nfts::PalletFeatures;
use polkadot_runtime_parachains::{disputes, inclusion, paras, scheduler, session_info};

use polkadot_runtime_parachains::{
	configuration,
	origin, shared,
};
pub type BlockNumber = u64;
pub type Index = u32;
use xcm::v3::prelude::*;
use xcm_builder::{
	AllowTopLevelPaidExecutionFrom, ChildParachainConvertsVia, FixedWeightBounds,
	SignedToAccountId32, TakeWeightCredit,
};
use xcm_executor::XcmExecutor;
type Origin = <Test as frame_system::Config>::RuntimeOrigin;
type Balance = u64;

pub fn root_user() -> Origin {
	RuntimeOrigin::root()
}
pub fn who(who: AccountId) -> Origin {
	RuntimeOrigin::signed(who)
}

pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Test>,
	frame_system::CheckSpecVersion<Test>,
	frame_system::CheckTxVersion<Test>,
	frame_system::CheckGenesis<Test>,
	frame_system::CheckEra<Test>,
	frame_system::CheckNonce<Test>,
	frame_system::CheckWeight<Test>,
);

type UncheckedExtrinsics = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test
	
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances,
		ParasOrigin: origin,
		MessageQueue: pallet_message_queue,
		XcmPallet: pallet_xcm,
		ParaInclusion: inclusion,
		Paras: paras,
		Xnft: pallet_xnft,
		Disputes: disputes,
		Scheduler: scheduler,
		Configuration: configuration,
		ParasShared: shared,
		ParasSudoWrapperCall:paras_sudo_wrapper,
		Dmp: parachains_dmp,
		NFT:pallet_nfts,
		CumulusXcm: cumulus_pallet_xcm,
		DmpQueue: cumulus_pallet_dmp_queue,
		XcmpQueue: cumulus_pallet_xcmp_queue,
		Hrmp: hrmp,
		MockAssigner: mock_assigner,
		Indices: pallet_indices,
	}

);
impl<C> frame_system::offchain::SendTransactionTypes<C> for Test
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsics;
	type OverarchingCall = RuntimeCall;
}
impl paras_sudo_wrapper::Config for Test {}

pub mod mock_assigner {
	use super::*;
	use crate::test::relay::scheduler::common::AssignmentProvider;
	pub use pallet::*;
	use crate::test::relay::scheduler::common::Assignment;

	#[frame_support::pallet]
	pub mod pallet {
		use frame_support::pallet_prelude::StorageValue;
		use super::*;

		#[pallet::pallet]
		#[pallet::without_storage_info]
		pub struct Pallet<T>(_);

		#[pallet::config]
		pub trait Config: frame_system::Config + configuration::Config + paras::Config {}

		#[pallet::storage]
		pub(super) type MockAssignmentQueue<T: Config> =
			StorageValue<_,VecDeque<Assignment>, ValueQuery>;

		#[pallet::storage]
		pub(super) type MockCoreCount<T: Config> = StorageValue<_, u32, OptionQuery>;
	}

	impl<T: Config> Pallet<T> {
		/// Adds a claim to the `MockAssignmentQueue` this claim can later be popped by the
		/// scheduler when filling the claim queue for tests.
		pub fn add_test_assignment(assignment: Assignment) {
			MockAssignmentQueue::<T>::mutate(|queue| queue.push_back(assignment));
		}

		// Allows for customized core count in scheduler tests, rather than a core count
		// derived from on-demand config + parachain count.
		pub fn set_core_count(count: u32) {
			MockCoreCount::<T>::set(Some(count));
		}
	}

	impl<T: Config> AssignmentProvider<BlockNumber> for Pallet<T> {
		// With regards to popping_assignments, the scheduler just needs to be tested under
		// the following two conditions:
		// 1. An assignment is provided
		// 2. No assignment is provided
		// A simple assignment queue populated to fit each test fulfills these needs.
		fn pop_assignment_for_core(_core_idx: CoreIndex) -> Option<Assignment> {
			let mut queue: VecDeque<Assignment> = MockAssignmentQueue::<T>::get();
			let front = queue.pop_front();
			// Write changes to storage.
			MockAssignmentQueue::<T>::set(queue);
			front
		}

		// We don't care about core affinity in the test assigner
		fn report_processed(_assignment: Assignment) {}

		// The results of this are tested in assigner_on_demand tests. No need to represent it
		// in the mock assigner.
		fn push_back_assignment(_assignment: Assignment) {}

		fn session_core_count() -> u32 {
			MockCoreCount::<T>::get().unwrap_or(5)
		}
	}
}

impl mock_assigner::pallet::Config for Test {}

parameter_types! {
	pub storage IndexDeposit: Balance = 1 * DOLLARS;
}

impl pallet_indices::Config for Test {
	type AccountIndex = AccountIndex;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}


#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_types! {
	pub const CollectionDeposit: Balance = 100 * DOLLARS;
	pub const ItemDeposit: Balance = 1 * DOLLARS;
	pub const KeyLimit: u32 = 32;
	pub const ValueLimit: u32 = 256;
	pub const ApprovalsLimit: u32 = 20;
	pub const ItemAttributesApprovalsLimit: u32 = 20;
	pub const MaxTips: u32 = 10;

}

parameter_types! {
	pub Features: PalletFeatures = PalletFeatures::all_enabled();
	pub const MaxAttributesPerCall: u32 = 10;
}

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = u32;
	type ItemId = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type Locker = ();
	type CollectionDeposit = ConstU64<2>;
	type ItemDeposit = ConstU64<1>;
	type MetadataDepositBase = ConstU64<1>;
	type AttributeDepositBase = ConstU64<1>;
	type DepositPerByte = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type KeyLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type ApprovalsLimit = ConstU32<10>;
	type ItemAttributesApprovalsLimit = ConstU32<2>;
	type MaxTips = ConstU32<10>;
	type MaxDeadlineDuration = ();
	type MaxAttributesPerCall = ConstU32<2>;
	type Features = Features;
	type OffchainSignature = Signature;
	type OffchainPublic = <Signature as sp_runtime::traits::Verify>::Signer;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
	type RuntimeHoldReason = ();
}
parameter_types! {
	pub const ParasUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
}

pub struct TestNextSessionRotation;

impl frame_support::traits::EstimateNextSessionRotation<u32> for TestNextSessionRotation {
	fn average_session_length() -> u32 {
		10
	}

	fn estimate_current_session_progress(_now: u32) -> (Option<Permill>, Weight) {
		(None, Weight::zero())
	}

	fn estimate_next_session_rotation(_now: u32) -> (Option<u32>, Weight) {
		(None, Weight::zero())
	}
}

impl paras::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = polkadot_runtime_parachains::paras::TestWeightInfo;
	type UnsignedPriority = ParasUnsignedPriority;
	type QueueFootprinter = ParaInclusion;
	type NextSessionRotation = ();
	type OnNewHead = ();
	type AssignCoretime = ();
}

thread_local! {
	pub static BACKING_REWARDS: RefCell<HashMap<ValidatorIndex, usize>>
		= RefCell::new(HashMap::new());

	pub static AVAILABILITY_REWARDS: RefCell<HashMap<ValidatorIndex, usize>>
		= RefCell::new(HashMap::new());
}

pub struct TestRewardValidators;

impl inclusion::RewardValidators for TestRewardValidators {
	fn reward_backing(v: impl IntoIterator<Item = ValidatorIndex>) {
		BACKING_REWARDS.with(|r| {
			let mut r = r.borrow_mut();
			for i in v {
				*r.entry(i).or_insert(0) += 1;
			}
		})
	}
	fn reward_bitfields(v: impl IntoIterator<Item = ValidatorIndex>) {
		AVAILABILITY_REWARDS.with(|r| {
			let mut r = r.borrow_mut();
			for i in v {
				*r.entry(i).or_insert(0) += 1;
			}
		})
	}
}

impl inclusion::Config for Test {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type DisputesHandler = Disputes;
	type RewardValidators = TestRewardValidators;
	type MessageQueue = MessageQueue;
}

pub struct ValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<AccountId>> for ValidatorIdOf {
	fn convert(a: AccountId) -> Option<AccountId> {
		Some(a)
	}
}

pub struct MockValidatorSet;

impl ValidatorSet<AccountId> for MockValidatorSet {
	type ValidatorId = AccountId;
	type ValidatorIdOf = ValidatorIdOf;
	fn session_index() -> SessionIndex {
		0
	}
	fn validators() -> Vec<Self::ValidatorId> {
		Vec::new()
	}
}

pub struct FoolIdentificationOf;
impl sp_runtime::traits::Convert<AccountId, Option<()>> for FoolIdentificationOf {
	fn convert(_: AccountId) -> Option<()> {
		Some(())
	}
}

impl ValidatorSetWithIdentification<AccountId> for MockValidatorSet {
	type Identification = ();
	type IdentificationOf = FoolIdentificationOf;
}

impl scheduler::Config for Test {
	type AssignmentProvider = MockAssigner;
}

impl disputes::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RewardValidators = Self;
	type SlashingHandler = ();
	type WeightInfo = disputes::TestWeightInfo;
}

thread_local! {
	pub static REWARD_VALIDATORS: RefCell<Vec<(SessionIndex, Vec<ValidatorIndex>)>> = RefCell::new(Vec::new());
	pub static PUNISH_VALIDATORS_FOR: RefCell<Vec<(SessionIndex, Vec<ValidatorIndex>)>> = RefCell::new(Vec::new());
	pub static PUNISH_VALIDATORS_AGAINST: RefCell<Vec<(SessionIndex, Vec<ValidatorIndex>)>> = RefCell::new(Vec::new());
	pub static PUNISH_BACKERS_FOR: RefCell<Vec<(SessionIndex, Vec<ValidatorIndex>)>> = RefCell::new(Vec::new());
}

impl disputes::RewardValidators for Test {
	fn reward_dispute_statement(
		session: SessionIndex,
		validators: impl IntoIterator<Item = ValidatorIndex>,
	) {
		REWARD_VALIDATORS.with(|r| r.borrow_mut().push((session, validators.into_iter().collect())))
	}
}

impl disputes::SlashingHandler<BlockNumber> for Test {
	fn punish_for_invalid(
		session: SessionIndex,
		_: CandidateHash,
		losers: impl IntoIterator<Item = ValidatorIndex>,
		backers: impl IntoIterator<Item = ValidatorIndex>,
	) {
		PUNISH_VALIDATORS_FOR
			.with(|r| r.borrow_mut().push((session, losers.into_iter().collect())));
		PUNISH_BACKERS_FOR.with(|r| r.borrow_mut().push((session, backers.into_iter().collect())));
	}

	fn punish_against_valid(
		session: SessionIndex,
		_: CandidateHash,
		losers: impl IntoIterator<Item = ValidatorIndex>,
		_backers: impl IntoIterator<Item = ValidatorIndex>,
	) {
		PUNISH_VALIDATORS_AGAINST
			.with(|r| r.borrow_mut().push((session, losers.into_iter().collect())))
	}

	fn initializer_initialize(_now: BlockNumber) -> Weight {
		Weight::zero()
	}

	fn initializer_finalize() {}

	fn initializer_on_new_session(_: SessionIndex) {}
}

thread_local! {
	pub static DISCOVERY_AUTHORITIES: RefCell<Vec<AuthorityDiscoveryId>> = RefCell::new(Vec::new());
}

pub fn discovery_authorities() -> Vec<AuthorityDiscoveryId> {
	DISCOVERY_AUTHORITIES.with(|r| r.borrow().clone())
}

impl session_info::AuthorityDiscoveryConfig for Test {
	fn authorities() -> Vec<AuthorityDiscoveryId> {
		discovery_authorities()
	}
}

impl session_info::Config for Test {
	type ValidatorSet = MockValidatorSet;
}

impl hrmp::Config for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type Currency = pallet_balances::Pallet<Test>;
	type WeightInfo = hrmp::TestWeightInfo;
	type ChannelManager = frame_system::EnsureRoot<AccountId>;
	type DefaultChannelSizeAndCapacityWithSystem = ();
}

impl shared::Config for Test {
	type DisabledValidators = ();
}

impl configuration::Config for Test {
	type WeightInfo = configuration::TestWeightInfo;
}

impl cumulus_pallet_xcm::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = ();
	type WeightInfo = ();
	type XcmpQueue = TransformOrigin<MessageQueue, polkadot_runtime_parachains::inclusion::AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
}

pub struct ParaIdToSibling;
impl sp_runtime::traits::Convert<ParaId, polkadot_runtime_parachains::inclusion::AggregateMessageOrigin> for ParaIdToSibling {
	fn convert(para_id: ParaId) -> polkadot_runtime_parachains::inclusion::AggregateMessageOrigin {
		polkadot_runtime_parachains::inclusion::AggregateMessageOrigin::Ump(UmpQueueId::Para(para_id))
		}}

parameter_types! {
	pub const RelayLocation: cumulus_primitives_core::Location = cumulus_primitives_core::Location::parent();
	pub const RelayNetwork: Option<NetworkId> = None;
	pub const AnyNetwork: Option<cumulus_primitives_core::NetworkId> = None;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub const UniversalLocation: xcm::latest::InteriorLocation = xcm::latest::Junctions::Here;
}

pub type SovereignAccountOf = (ChildParachainConvertsVia<ParaId, AccountId>,);
parameter_types! {
	pub const MaxAssetsForTransfer: usize = 2;
	pub const BaseXcmWeight: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
}

parameter_types! {
	/// The amount of weight an XCM operation takes. This is a safe overestimate.
	/// The asset ID for the asset that we use to pay for message delivery fees.
	pub FeeAssetId: cumulus_primitives_core::AssetId = cumulus_primitives_core::AssetId(RelayLocation::get());
	/// The base fee for the message delivery fees.
	pub const BaseDeliveryFee: u64 = CENTS.saturating_mul(3);
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
	/// This value increases the priority of `Operational` transactions by adding
	/// a "virtual tip" that's equal to the `OperationalFeeMultiplier * final_fee`.
	pub const OperationalFeeMultiplier: u8 = 5;
}

pub type XcmRouter = (
	// Only one router so far - use DMP to communicate with child parachains.
	ChildParachainRouter<
		Test,
		XcmPallet,
		ExponentialPrice<FeeAssetId, BaseDeliveryFee, TransactionByteFee, Dmp>,
	>,
);

pub type Barrier = (TakeWeightCredit, AllowTopLevelPaidExecutionFrom<Everything>);

parameter_types! {
	pub const UnitWeightCost: Weight = Weight::from_parts(10, 10);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub static RecordedMessages: Vec<Vec<u8>> = vec![];
}


/// Can be used as [`Config::DmpSink`] to record all messages that came in.
pub struct RecordingDmpSink;
impl HandleMessage for RecordingDmpSink {
	type MaxMessageLen = ConstU32<16>;

	fn handle_message(msg: BoundedSlice<u8, Self::MaxMessageLen>) {
		RecordedMessages::mutate(|n| n.push(msg.to_vec()));
	}

	fn handle_messages<'a>(_: impl Iterator<Item = BoundedSlice<'a, u8, Self::MaxMessageLen>>) {
		unimplemented!()
	}

	fn sweep_queue() {
		unimplemented!()
	}

	fn footprint() -> QueueFootprint {
		unimplemented!()
	}
}

impl cumulus_pallet_dmp_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type DmpSink = RecordingDmpSink;
	type WeightInfo = ();
}

impl parachains_dmp::Config for Test {}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type OriginConverter = ();
	type IsReserve = NativeAsset;
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type ResponseHandler = ();
	type AssetTrap = ();
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = ();
	type SubscriptionService = ();
	type PalletInstancesInfo = ();
	type AssetTransactor = ();
	type Trader = ();
	type FeeManager = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = ();
	type TransactionalProcessor = ();
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, AnyNetwork>;

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}

impl origin::Config for Test {}

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Weight::from_parts(1_000_000_000, 1_000_000);
	pub const MessageQueueHeapSize: u32 = 65_536;
	pub const MessageQueueMaxStale: u32 = 16;
}

impl pallet_message_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Size = u32;
	type HeapSize = MessageQueueHeapSize;
	type MaxStale = MessageQueueMaxStale;
	type ServiceWeight = MessageQueueServiceWeight;
	type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<
		polkadot_runtime_parachains::inclusion::AggregateMessageOrigin,
	>;
	type QueueChangeHandler = ();
	type QueuePausedQuery = ();
	type WeightInfo = ();
	type IdleMaxServiceWeight = ();
}

impl pallet_xnft::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmSender = XcmRouter;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let storage = <frame_system::GenesisConfig<Test> as BuildStorage>::build_storage(&frame_system::GenesisConfig::default()).unwrap();
		storage.into()
	}
}

pub fn assert_last_events<E>(generic_events: E)
where
	E: DoubleEndedIterator<Item = RuntimeEvent> + ExactSizeIterator,
{
	for (i, (got, want)) in frame_system::Pallet::<Test>::events()
		.into_iter()
		.rev()
		.map(|e| e.event)
		.zip(generic_events.rev().map(<Test as frame_system::Config>::RuntimeEvent::from))
		.rev()
		.enumerate()
	{
		assert_eq!((i, got), (i, want));
	}
}

#[derive(Default)]
pub struct MockGenesisConfig {
	pub system: frame_system::GenesisConfig<Test>,
	pub configuration: configuration::GenesisConfig<Test>,
	pub paras: paras::GenesisConfig<Test>,
}


pub(crate) fn register_parachain_with_balance(id: ParaId, balance: Balance) {
	let validation_code: ValidationCode = vec![1].into();
	assert_ok!(schedule_para_initialize::<Test>(
		id,
		paras::ParaGenesisArgs {
			para_kind: paras::ParaKind::Parachain,
			genesis_head: vec![1].into(),
			validation_code: validation_code.clone(),
		},
	));

	assert_ok!(Paras::add_trusted_validation_code(RuntimeOrigin::root(), validation_code));
	<Test as hrmp::Config>::Currency::make_free_balance_be(
		&id.into_account_truncating(),
		balance.try_into().unwrap(),
	);
}