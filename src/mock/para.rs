#![cfg(test)]
use crate::{self as pallet_xnft};
use frame_support::derive_impl;
use std::sync::Arc;
use sp_runtime::BuildStorage;
use crate::mock::message_queue::pallet as mock_msg_queue;
use frame_support::traits::TransformOrigin;
use parachains_common::message_queue::ParaIdToSibling;
use cumulus_primitives_core::AggregateMessageOrigin as MessageOriginAggregate;
use polkadot_runtime_common::NORMAL_DISPATCH_RATIO;
use frame_system::limits::BlockLength;
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use frame_system::limits::BlockWeights;
use frame_support::weights::constants::BlockExecutionWeight;
use frame_support::weights::constants::ExtrinsicBaseWeight;
use frame_support::pallet_prelude::DispatchClass;
use xcm_builder::FrameTransactionalProcessor;
use frame_support::traits::Nothing;
use polkadot_runtime_common::MAXIMUM_BLOCK_WEIGHT;
use polkadot_runtime_common::AVERAGE_ON_INITIALIZE_RATIO;
use cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
use frame_support::{
	construct_runtime, match_types,
	parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU32, ConstU64, Everything,
	},
	weights::constants::WEIGHT_REF_TIME_PER_SECOND,
};
use frame_system::{EnsureRoot};
pub use sp_runtime::{
	testing::Header,
	traits::{AccountIdLookup, BlakeTwo256, IdentityLookup},
	DispatchError, MultiSignature,
};
use crate::test::para::currency::DOLLARS;
pub type CollectionId = u64;
type Origin = <Test as frame_system::Config>::RuntimeOrigin;
type Balance = u64;
use cumulus_primitives_core::ParaId;
use pallet_nfts::PalletFeatures;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain_primitives::primitives::Sibling;
use xcm::v3::{prelude::*, Weight};
use xcm_builder::{
	AccountId32Aliases, AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds,
	NativeAsset, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::XcmExecutor;
pub mod currency {
	pub type Balance = u64;
	pub const MILLICENTS: Balance = 1_000_000_000;
	pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
	pub const DOLLARS: Balance = 100 * CENTS;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}
pub fn root_user() -> Origin {
	RuntimeOrigin::root()
}
pub fn who(who: AccountId) -> Origin {
	RuntimeOrigin::signed(who)
}

pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as sp_runtime::traits::Verify>::Signer;
pub type AccountId = <AccountPublic as sp_runtime::traits::IdentifyAccount>::AccountId;
pub type BlockNumber = u32;
pub type Index = u32;
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
		XcmpQueue: cumulus_pallet_xcmp_queue,
		DmpQueue: cumulus_pallet_dmp_queue,
		CumulusXcm: cumulus_pallet_xcm,
		MessageQueue: pallet_message_queue,
		PolkadotXcm: pallet_xcm,
		Xnft: pallet_xnft,
		MsgQueue:mock_msg_queue,
		ParachainSystem: cumulus_pallet_parachain_system,
		NFT:pallet_nfts::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 42;
	pub const RelayOrigin: MessageOriginAggregate = MessageOriginAggregate::Parent;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}



#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

impl cumulus_pallet_parachain_system::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = ();
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type WeightInfo = ();
}

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
		cumulus_primitives_core::AggregateMessageOrigin,
	>;
	type QueueChangeHandler = ();
	type QueuePausedQuery = ();
	type WeightInfo = ();
	type IdleMaxServiceWeight = ();
}

impl mock_msg_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
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

parameter_types! {
	pub const AssetDeposit: Balance = 100 * DOLLARS;
	pub const ApprovalDeposit: Balance = 1 * DOLLARS;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 10 * DOLLARS;
	pub const MetadataDepositPerByte: Balance = 1 * DOLLARS;
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
	pub const ReservedXcmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
	pub const ReservedDmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
}

parameter_types! {
	pub const RelayLocation: Location = Here.into_location();
	pub const RelayNetwork: Option<NetworkId> = None;
	pub const AnyNetwork: Option<cumulus_primitives_core::NetworkId> = None;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub const UniversalLocation: xcm::latest::InteriorLocation = xcm::latest::Junctions::Here;
}

pub type LocationToAccountId = (
	ParentIsPreset<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type XcmOriginToCallOrigin = (
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	XcmPassthrough<RuntimeOrigin>,
);

pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);
pub type Barrier = (TakeWeightCredit, AllowTopLevelPaidExecutionFrom<Everything>);

parameter_types! {
	pub const UnitWeightCost: Weight = Weight::from_parts(10, 10);
	pub const BaseXcmWeight: Weight = Weight::from_parts(100_000_000, 100_000_000);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

parameter_types! {
	// pub const Roc: xcm_simulator::AssetFilter = Wild(AllOf { fun: WildFungible, id: xcm_simulator::AssetId(RelayLocation::get()) });
	// pub const Para: cumulus_primitives_core::Location = cumulus_primitives_core::Location {
	// 	parents: 1,
	// 	interior: cumulus_primitives_core::Junctions::X1(Arc::new([cumulus_primitives_core::Junction::Parachain(2000); 1])),
	// };
	// pub const Para2: cumulus_primitives_core::Location = cumulus_primitives_core::Location {
	// 	parents: 1,
	// 	interior: cumulus_primitives_core::Junctions::X1(Arc::new([cumulus_primitives_core::Junction::Parachain(2001); 1])),
	// };
	// pub const OurchainPara: (MultiAssetFilter, cumulus_primitives_core::Location) = (Roc::get(), Para::get());
	// pub const OurchainPara2: (MultiAssetFilter, cumulus_primitives_core::Location) = (Roc::get(), Para2::get());
	// pub Tick: Location = Parachain(2000).into_location();
	// pub Trick: Location = Parachain(2001).into_location();
	// pub OurchainPara: (xcm_simulator::AssetFilter, Location) = (Roc::get(), Tick::get());
	// pub OurchainPara2: (xcm_simulator::AssetFilter, Location) = (Roc::get(), Trick::get());

}

// pub type TrustedTeleporters = (xcm_builder::Case<OurchainPara>, xcm_builder::Case<OurchainPara2>);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = ();
	type OriginConverter = ();
	type IsReserve = NativeAsset;
	type IsTeleporter = NativeAsset;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = ();
	type ResponseHandler = ();
	type AssetTrap = ();
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = ();
	type SubscriptionService = ();
	type PalletInstancesInfo = ();
	type FeeManager = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type MessageExporter = ();
	type UniversalAliases = Everything;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
}

impl cumulus_pallet_xcmp_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = ();
	type WeightInfo = ();
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type XcmpQueue = TransformOrigin<MessageQueue,MessageOriginAggregate, ParaId, ParaIdToSibling>;
}

impl cumulus_pallet_dmp_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type DmpSink = crate::test::relay::RecordingDmpSink;
	type WeightInfo = ();
}

impl cumulus_pallet_xcm::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
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
	type XcmTeleportFilter = Everything;
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

match_types! {
	pub type ParentOrParachains: impl Contains<MultiLocation> = {
		MultiLocation { parents: 0, interior: X1(Junction::AccountId32 { .. }) } |
		MultiLocation { parents: 1, interior: X1(Junction::AccountId32 { .. }) } |
		MultiLocation { parents: 1, interior: X1(Parachain(2000)) } |
		MultiLocation { parents: 1, interior: X1(Parachain(2001)) } |
		MultiLocation { parents: 1, interior: X2(Parachain(2000), Junction::AccountId32 { .. }) } |
		MultiLocation { parents: 1, interior: X2(Parachain(2001), Junction::AccountId32 { .. }) } |
		MultiLocation { parents: 1, interior: X2(Parachain(100), Junction::AccountId32 { .. }) }
	};
}

parameter_types! {
	pub const MaxAssetsForTransfer: usize = 2;
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
pub fn new_test_ext() -> sp_io::TestExternalities {
	<frame_system::GenesisConfig<Test> as BuildStorage>::build_storage(&frame_system::GenesisConfig::default()).unwrap().into()
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

