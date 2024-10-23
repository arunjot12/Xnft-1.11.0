#![cfg(test)]
use crate::{self as pallet_xnft};
use frame_support::derive_impl;
use crate::test::relay::BaseDeliveryFee;
use frame_support::traits::TransformOrigin;
use xcm_builder::ParentAsSuperuser;
use sp_runtime::BuildStorage;
use cumulus_primitives_core::AggregateMessageOrigin;
use crate::test::relay::FeeAssetId;
use parachains_common::message_queue::ParaIdToSibling;
use crate::test::relay::TransactionByteFee;
use frame_support::{
	construct_runtime, match_types,
	pallet_prelude::Get,
	parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU32, ConstU64, Everything,
	},
	weights::constants::WEIGHT_REF_TIME_PER_SECOND,
};
use frame_system::EnsureRoot;
use cumulus_pallet_parachain_system::ParachainSetCode;
pub use sp_runtime::{
	testing::Header,
	traits::{AccountIdLookup, BlakeTwo256, IdentityLookup},
	DispatchError, MultiSignature,
};
use polkadot_runtime_common::BlockHashCount;
use crate::test::para::currency::DOLLARS;
use crate::test::relay::MessageQueue;
pub type CollectionId = u64;
type Origin = <Test as frame_system::Config>::RuntimeOrigin;
type Balance = u128;
use sp_version::create_runtime_str;
use cumulus_primitives_core::ParaId;
use pallet_nfts::PalletFeatures;
use sp_version::RuntimeVersion;
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
use cumulus_pallet_parachain_system::AnyRelayNumber;
pub mod currency {
	use node_primitives::Balance;

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
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsics,
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances,
		ParachainInfo: parachain_info,
		XcmpQueue: cumulus_pallet_xcmp_queue,
		DmpQueue: cumulus_pallet_dmp_queue,
		CumulusXcm: cumulus_pallet_xcm,
		PolkadotXcm: pallet_xcm,
		Xnft: pallet_xnft,
		ParachainSystem: cumulus_pallet_parachain_system,
		NFT:pallet_nfts::{Pallet, Call, Storage, Event<T>},
	}
);

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("parachain"),
	impl_name: create_runtime_str!("parachain"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type BlockHashCount = BlockHashCount;
	type Version = Version;
	type OnSetCode = ParachainSetCode<Self>;
}

pub struct SaveIntoThreadLocal;

impl cumulus_pallet_parachain_system::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId =  parachain_info::Pallet<Test>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = SaveIntoThreadLocal;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = AnyRelayNumber;
	type WeightInfo = ();
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
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
	pub const ReservedDmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl parachain_info::Config for Test {}

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: Option<NetworkId> = None;
	pub const AnyNetwork: Option<NetworkId> = None;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
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
	pub const Roc: MultiAssetFilter = Wild(AllOf { fun: WildFungible, id: Concrete(RelayLocation::get()) });
	pub const Para: MultiLocation = Parachain(2000).into_location();
	pub const Para2: MultiLocation = Parachain(2001).into_location();
	pub const OurchainPara: (MultiAssetFilter, MultiLocation) = (Roc::get(), Para::get());
	pub const OurchainPara2: (MultiAssetFilter, MultiLocation) = (Roc::get(), Para2::get());

}

pub type TrustedTeleporters = (xcm_builder::Case<OurchainPara>, xcm_builder::Case<OurchainPara2>);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = ();
	type OriginConverter = ();
	type IsReserve = NativeAsset;
	type IsTeleporter = TrustedTeleporters;
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
}

pub type PriceForSiblingParachainDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
	FeeAssetId,
	BaseDeliveryFee,
	TransactionByteFee,
	XcmpQueue,
>;

pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

impl cumulus_pallet_xcmp_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type PriceForSiblingDelivery = PriceForSiblingParachainDelivery;
}

impl cumulus_pallet_dmp_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
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

parameter_types! {
	pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::get().into())));
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
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		storage.into()
	}
}
