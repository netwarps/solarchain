#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![allow(clippy::identity_op)]

use codec::{Decode, Encode};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_contracts::{migration, weights::WeightInfo, DefaultContractAccessWeight};
use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
	crypto::{ByteArray, KeyTypeId},
	OpaqueMetadata, H160, H256, U256,
};
use sp_runtime::{
	create_runtime_str,
	curve::PiecewiseLinear,
	generic, impl_opaque_keys,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, DispatchInfoOf, Dispatchable,
		IdentifyAccount, NumberFor, OpaqueKeys, PostDispatchInfoOf, Verify,
	},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
		TransactionValidityError,
	},
	ApplyExtrinsicResult, MultiSignature, Percent, SaturatedConversion,
};
use sp_std::{marker::PhantomData, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;

use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
use pallet_evm::{
	Account as EVMAccount, EVMCurrencyAdapter, EnsureAddressTruncated, FeeCalculator,
	GasWeightMapping, HashedAddressMapping, OnChargeEVMTransaction as OnChargeEVMTransactionT,
	Runner,
};

// A few exports that help ease life for downstream crates.
use fp_rpc::TransactionStatus;
use frame_election_provider_support::{
	onchain, SequentialPhragmen,
	VoteWeight,
};
pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, Contains,
		Currency as CurrencyT, EnsureOneOf, EqualPrivilegeOnly, FindAuthor, Imbalance,
		InstanceFilter, KeyOwnerProofSystem, LockIdentifier, OffchainWorker, OnFinalize, OnIdle,
		OnInitialize, OnRuntimeUpgrade, OnUnbalanced, Randomness, U128CurrencyToVote,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		ConstantMultiplier, DispatchClass, GetDispatchInfo, IdentityFee, Weight,
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
	ConsensusEngineId, PalletId, StorageValue,
};

pub use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_election_provider_multi_phase::Call as EPMCall;
#[cfg(feature = "std")]
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::CurrencyAdapter;
use smallvec::smallvec;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};
mod impl_on_charge_evm_transaction;
mod impl_runtime_apis_plus_common;
mod impl_self_contained_call;
mod precompiles;
pub use precompiles::FrontierPrecompiles;

// Make the WASM binary available.
#[cfg(all(feature = "std", feature = "include-wasm"))]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Wasm binary unwrapped. If built with `SKIP_WASM_BUILD`, the function panics.
#[cfg(all(feature = "std", feature = "include-wasm"))]
pub fn wasm_binary_unwrap() -> &'static [u8] {
	WASM_BINARY.expect(
		"Development wasm binary is not available. This means the client is built with \
		 `SKIP_WASM_BUILD` flag and it is only usable for production chains. Please rebuild with \
		 the flag disabled.",
	)
}

/// SOLAR, the native token, uses 18 decimals of precision.
pub mod currency {
	use super::Balance;

	// Provide a common factor between runtimes based on a supply of 10_000_000 tokens.
	pub const SUPPLY_FACTOR: Balance = 1;

	pub const WEI: Balance = 1;
	pub const KILOWEI: Balance = 1_000;
	pub const MEGAWEI: Balance = 1_000_000;
	pub const GIGAWEI: Balance = 1_000_000_000;
	pub const MICROSOLAR: Balance = 1_000_000_000_000;
	pub const MILLISOLAR: Balance = 1_000_000_000_000_000;
	pub const SOLAR: Balance = 1_000_000_000_000_000_000;
	pub const KILOSOLAR: Balance = 1_000_000_000_000_000_000_000;
	// Unit = the base number of indivisible units for balances
	pub const UNIT: Balance = 1_000_000_000_000_000_000;
	pub const TRANSACTION_BYTE_FEE: Balance = 10 * MICROSOLAR * SUPPLY_FACTOR;
	pub const STORAGE_BYTE_FEE: Balance = 100 * MICROSOLAR * SUPPLY_FACTOR;
	pub const WEIGHT_FEE: Balance = 100 * KILOWEI * SUPPLY_FACTOR;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 100 * MILLISOLAR * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
	}
}

/// An index to a block.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
			pub im_online: ImOnline,
			pub authority_discovery: AuthorityDiscovery,
		}
	}
}

parameter_types! {
	pub const Period: BlockNumber = 5;
	pub const Offset: BlockNumber = 0;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: sp_staking::EraIndex = 24 * 28;
	pub const SlashDeferDuration: sp_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub OffchainRepeat: BlockNumber = 5;
}

pub struct StakingBenchmarkingConfig;

impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxNominators = ConstU32<1000>;
	type MaxValidators = ConstU32<1000>;
}

frame_election_provider_support::generate_solution_type!(
	#[compact]
	pub struct NposSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
		MaxVoters = MaxElectingVoters,
	>(16)
);

parameter_types! {
	pub MaxNominations: u32 = <NposSolution16 as frame_election_provider_support::NposSolution>::LIMIT as u32;
	pub MaxElectingVoters: u32 = 10_000;
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<
		AccountId,
		pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
	>;
	type DataProvider = <Runtime as pallet_election_provider_multi_phase::Config>::DataProvider;
	type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
}

impl onchain::BoundedConfig for OnChainSeqPhragmen {
	type VotersBound = MaxElectingVoters;
	type TargetsBound = ConstU32<2_000>;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = Call;
}

type SlashCancelOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, Council, 1, 2>,
>;

pub struct ElectionProviderBenchmarkConfig;
impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionProviderBenchmarkConfig {
	const VOTERS: [u32; 2] = [1000, 2000];
	const TARGETS: [u32; 2] = [500, 1000];
	const ACTIVE_VOTERS: [u32; 2] = [500, 800];
	const DESIRED_TARGETS: [u32; 2] = [200, 400];
	const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
	const MINER_MAXIMUM_VOTERS: u32 = 1000;
	const MAXIMUM_TARGETS: u32 = 300;
}

parameter_types! {
	// phase durations. 1/4 of the last session for each.
	pub const SignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;
	pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;

	// signed config
	pub const SignedRewardBase: Balance = 1 * currency::SOLAR;
	pub const SignedDepositBase: Balance = 1 * currency::SOLAR;
	pub const SignedDepositByte: Balance = 1 * currency::SOLAR/100;

	pub BetterUnsignedThreshold: Perbill = Perbill::from_rational(1u32, 10_000);

	// miner configs
	pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
	pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
		.saturating_sub(BlockExecutionWeight::get());
	// Solution can occupy 90% of normal block size
	pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
		*RuntimeBlockLength::get()
		.max
		.get(DispatchClass::Normal);
}

impl pallet_election_provider_multi_phase::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type EstimateCallFee = TransactionPayment;
	type UnsignedPhase = UnsignedPhase;
	type SignedMaxSubmissions = ConstU32<10>;
	type SignedMaxRefunds = ConstU32<3>;
	type SignedRewardBase = SignedRewardBase;
	type SignedDepositBase = SignedDepositBase;
	type SignedDepositByte = SignedDepositByte;
	type SignedDepositWeight = ();
	type SignedMaxWeight = Self::MinerMaxWeight;
	type SlashHandler = (); // burn slashes
	type RewardHandler = (); // nothing to do upon rewards
	type SignedPhase = SignedPhase;
	type BetterUnsignedThreshold = BetterUnsignedThreshold;
	type BetterSignedThreshold = ();
	type MinerMaxWeight = MinerMaxWeight; // For now use the one from staking.
	type MinerMaxLength = MinerMaxLength;
	type OffchainRepeat = OffchainRepeat;
	type MinerTxPriority = MultiPhaseUnsignedPriority;
	type DataProvider = Staking;
	type Solution = NposSolution16;
	type Fallback = pallet_election_provider_multi_phase::NoFallback<Self>;
	type GovernanceFallback = onchain::UnboundedExecution<OnChainSeqPhragmen>;
	type Solver = SequentialPhragmen<
		AccountId,
		pallet_election_provider_multi_phase::SolutionAccuracyOf<Self>,
		(),
	>;
	type BenchmarkingConfig = ElectionProviderBenchmarkConfig;
	// type ForceOrigin = EnsureOneOf<
	// 	EnsureRoot<AccountId>,
	// 	pallet_collective::EnsureProportionAtLeast<AccountId, Council, 2, 3>,
	// >;//todo
	type ForceOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Self>;
	type MaxElectingVoters = MaxElectingVoters;
	type MaxElectableTargets = ConstU16<{ u16::MAX }>;
}

impl pallet_staking::Config for Runtime {
	type MaxNominations = ConstU32<1000>;
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnixTime = Timestamp;
	type CurrencyToVote = U128CurrencyToVote;
	type ElectionProvider = ElectionProviderMultiPhase;
	type GenesisElectionProvider = onchain::UnboundedExecution<OnChainSeqPhragmen>;
	type RewardRemainder = Treasury;
	type Event = Event;
	type Slash = Treasury;
	type Reward = ();
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	// A majority of the council or root can cancel the slash.
	//type SlashCancelOrigin = SlashCancelOrigin; //todo
	type SlashCancelOrigin = EnsureRoot<AccountId>;
	type SessionInterface = Self;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type NextNewSession = Session;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
	type VoterList = BagsList;
	type MaxUnlockingChunks = frame_support::traits::ConstU32<32>;
	type OnStakerSlash = NominationPools;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	type BenchmarkingConfig = StakingBenchmarkingConfig;
}

parameter_types! {
	pub const PostUnbondPoolsWindow: u32 = 4;
	pub const NominationPoolsPalletId: PalletId = PalletId(*b"py/nopls");
	pub const MinPointsToBalance: u32 = 10;
}

use sp_runtime::traits::Convert;
pub struct BalanceToU256;
impl Convert<Balance, sp_core::U256> for BalanceToU256 {
	fn convert(balance: Balance) -> sp_core::U256 {
		sp_core::U256::from(balance)
	}
}
pub struct U256ToBalance;
impl Convert<sp_core::U256, Balance> for U256ToBalance {
	fn convert(n: sp_core::U256) -> Balance {
		n.try_into().unwrap_or(Balance::max_value())
	}
}

impl pallet_nomination_pools::Config for Runtime {
	type WeightInfo = ();
	type Event = Event;
	type Currency = Balances;
	type BalanceToU256 = BalanceToU256;
	type U256ToBalance = U256ToBalance;
	type StakingInterface = pallet_staking::Pallet<Self>;
	type PostUnbondingPoolsWindow = PostUnbondPoolsWindow;
	type MaxMetadataLen = ConstU32<256>;
	type MaxUnbonding = ConstU32<8>;
	type PalletId = NominationPoolsPalletId;
}

// To learn more about runtime versioning and what each of the following value means:
//   https://docs.substrate.io/v3/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("solar-node"),
	impl_name: create_runtime_str!("solar-node"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 112,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

pub const EPOCH_DURATION_IN_BLOCKS: u32 = 10 * MINUTES;

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Storage
frame_support::parameter_types! {
	pub storage MillisecsPerBlock: Moment = MILLISECS_PER_BLOCK;
	pub storage SecsPerBlock: Moment = MILLISECS_PER_BLOCK / 1000;

	pub storage SlotDuration: Moment = MILLISECS_PER_BLOCK;
	pub storage EpochDurationInBlocks: BlockNumber = 1 * HOURS;
	pub storage EpochDurationInSlots: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};

	// These time units are defined in number of blocks.
	pub storage Minutes: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	pub storage Hours: BlockNumber = MINUTES * 60;
	pub storage Days: BlockNumber = HOURS * 24;
}

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);

/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

// Prints debug output of the `contracts` pallet to stdout if the node is
// started with `-lruntime::contracts=debug`.
const CONTRACTS_DEBUG_OUTPUT: bool = true;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
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

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount; //ConstU32<256>;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = ConstU16<42>;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = ();

	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type HandleEquivocation = ();

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	//type FindAuthor=();
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	//type EventHandler=();
	type EventHandler = (Staking, ImOnline);
}

impl pallet_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<0>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;

	/// Return a vec of coefficients. Here we just use one coefficient and reduce it to a constant
	/// modifier in order to closely match Ethereum-based fees.
	///
	/// Calculation, per the documentation in `frame_support`:
	///
	/// ```ignore
	/// coeff_integer * x^(degree) + coeff_frac * x^(degree)
	/// ```
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			coeff_frac: Perbill::zero(),
			coeff_integer: currency::WEIGHT_FEE,
			negative: false,
		}]
	}
}

pub struct LengthToFee;
impl WeightToFeePolynomial for LengthToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		smallvec![
			WeightToFeeCoefficient {
				degree: 1,
				coeff_frac: Perbill::zero(),
				coeff_integer: currency::TRANSACTION_BYTE_FEE,
				negative: false,
			},
			WeightToFeeCoefficient {
				degree: 3,
				coeff_frac: Perbill::zero(),
				coeff_integer: 1,
				negative: false,
			},
		]
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = WeightToFee;
	type LengthToFee = LengthToFee;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}
use crate::currency::deposit;
parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	// The lazy deletion runs inside on_initialize.
	pub DeletionWeightLimit: Weight = RuntimeBlockWeights::get()
		.per_class
		.get(DispatchClass::Normal)
		.max_total
		.unwrap_or(RuntimeBlockWeights::get().max_block);
	// The weight needed for decoding the queue should be less or equal than a fifth
	// of the overall weight dedicated to the lazy deletion.
	pub DeletionQueueDepth: u32 = ((DeletionWeightLimit::get() / (
			<Runtime as pallet_contracts::Config>::WeightInfo::on_initialize_per_queue_item(1) -
			<Runtime as pallet_contracts::Config>::WeightInfo::on_initialize_per_queue_item(0)
		)) / 5) as u32;
	pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = RandomnessCollectiveFlip;
	type Currency = Balances;
	type Event = Event;
	type Call = Call;
	/// The safest default is to allow no calls at all.
	///
	/// Runtimes should whitelist dispatchables that are allowed to be called from contracts
	/// and make sure they are stable. Dispatchables exposed to contracts are not allowed to
	/// change because that would break already deployed contracts. The `Call` structure itself
	/// is not allowed to change the indices of existing pallets, too.
	type CallFilter = frame_support::traits::Nothing;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type CallStack = [pallet_contracts::Frame<Self>; 31];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = ();
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type Schedule = Schedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type ContractAccessWeight = DefaultContractAccessWeight<RuntimeBlockWeights>;
	// This node is geared towards development and testing of contracts.
	// We decided to increase the default allowed contract size for this
	// reason (the default is `128 * 1024`).
	//
	// Our reasoning is that the error code `CodeTooLarge` is thrown
	// if a too-large contract is uploaded. We noticed that it poses
	// less friction during development when the requirement here is
	// just more lax.
	// type MaxCodeLen = ConstU32<{ 256 * 1024 }>;
	// type RelaxedMaxCodeLen = ConstU32<{ 512 * 1024 }>;
}

pub struct Migrations;
impl OnRuntimeUpgrade for Migrations {
	fn on_runtime_upgrade() -> Weight {
		migration::migrate::<Runtime>()
	}
}

// Define the types required by the Scheduler pallet.
parameter_types! {
	//pub MaximumSchedulerWeight: Weight = 10_000_000;
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

// Configure the runtime's implementation of the Scheduler pallet.
impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = Preimage;
	type NoPreimagePostponement = NoPreimagePostponement;
}

parameter_types! {
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = 1 * currency::UNIT;
	pub const PreimageByteDeposit: Balance = 1 * currency::UNIT/100;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type Event = Event;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type MaxSize = PreimageMaxSize;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

parameter_types! {
	pub const MaxWellKnownNodes: u32 = 8;
	pub const MaxPeerIdLength: u32 = 128;
}

impl pallet_node_authorization::Config for Runtime {
	type Event = Event;
	type MaxWellKnownNodes = MaxWellKnownNodes;
	type MaxPeerIdLength = MaxPeerIdLength;
	type AddOrigin = EnsureRoot<AccountId>;
	type RemoveOrigin = EnsureRoot<AccountId>;
	type SwapOrigin = EnsureRoot<AccountId>;
	type ResetOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id = Aura::authorities()[author_index as usize].clone();
			return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]))
		}
		None
	}
}

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

pub struct FixedGasWeightMapping;

impl pallet_evm::GasWeightMapping for FixedGasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight {
		gas.saturating_mul(WEIGHT_PER_GAS)
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		u64::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(u32::MAX as u64)
	}
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		((1 * currency::GIGAWEI * currency::SUPPLY_FACTOR).into(), 0u64)
	}
}

parameter_types! {
	pub const ChainId: u64 = 1668;
	pub BlockGasLimit: U256 = U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
	pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::new();
}

impl_on_charge_evm_transaction!();

impl pallet_evm::Config for Runtime {
	type FeeCalculator = BaseFee;
	type GasWeightMapping = FixedGasWeightMapping;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = FrontierPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = ();
	type FindAuthor = FindAuthorTruncated<Aura>;
}

impl pallet_ethereum::Config for Runtime {
	type Event = Event;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

frame_support::parameter_types! {
	pub BoundDivision: U256 = U256::from(1024);
}

impl pallet_dynamic_fee::Config for Runtime {
	type MinGasPriceBoundDivisor = BoundDivision;
}

frame_support::parameter_types! {
	pub DefaultBaseFeePerGas: U256 = (1 * currency::GIGAWEI * currency::SUPPLY_FACTOR).into();
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type Event = Event;
	type Threshold = BaseFeeThreshold;
	type IsActive = ConstBool<true>;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
}

impl pallet_hotfix_sufficients::Config for Runtime {
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type WeightInfo = pallet_hotfix_sufficients::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilInstance = pallet_collective::Instance1;
type TechCommitteeInstance = pallet_collective::Instance2;

impl pallet_collective::Config<CouncilInstance> for Runtime {
	type Origin = Origin;
	type Event = Event;
	type Proposal = Call;
	/// The maximum amount of time (in blocks) for council members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = CouncilMotionDuration;
	/// The maximum number of Proposlas that can be open in the council at once.
	type MaxProposals = CouncilMaxProposals;
	/// The maximum number of council members.
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub TechnicalMotionDuration: BlockNumber = 3 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

impl pallet_collective::Config<TechCommitteeInstance> for Runtime {
	type Origin = Origin;
	type Event = Event;
	type Proposal = Call;
	/// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = TechnicalMotionDuration;
	/// The maximum number of Proposlas that can be open in the technical committee at once.
	type MaxProposals = TechnicalMaxProposals;
	/// The maximum number of technical committee members.
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}


// type EnsureRootOrHalfCouncil = EnsureOneOf<
// 	EnsureRoot<AccountId>,
// 	pallet_collective::EnsureProportionMoreThan<AccountId, Council, 1, 2>,
// >;

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRoot<AccountId>;
	type RemoveOrigin = EnsureRoot<AccountId>;
	type SwapOrigin = EnsureRoot<AccountId>;
	type ResetOrigin = EnsureRoot<AccountId>;
	type PrimeOrigin = EnsureRoot<AccountId>;
	type MembershipInitialized = TechCommittee;
	type MembershipChanged = TechCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}


parameter_types! {
	pub const CandidacyBond: Balance = 10 * currency::SOLAR;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	pub TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type Event = Event;
	type PalletId = ElectionsPhragmenPalletId;
	type Currency = Balances;
	type ChangeMembers = Council;
	// NOTE: this implies that council's genesis members cannot be set directly and must come from
	// this module.
	type InitializeMembers = Council;
	type CurrencyToVote = U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = ();
	type KickedMember = ();
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = ConstU32<{ 2 * DAYS }>;
	type LaunchPeriod = ConstU32<{ 7 * DAYS }>;
	type VotingPeriod = ConstU32<{ 14 * DAYS }>;
	type VoteLockingPeriod = ConstU32<{ 7 * DAYS }>;
	type FastTrackVotingPeriod = ConstU32<{ 1 * DAYS }>;
	type MinimumDeposit = ConstU128<{ 4 * currency::SOLAR * currency::SUPPLY_FACTOR }>;
	/// To decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 1, 2>;
	/// To have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>;
	/// To have the next scheduled referendum be a straight default-carries (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>;
	/// To allow a shorter voting/enactment period for external proposals.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 1, 2>;
	/// To instant fast track.
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 3, 5>;
	// To cancel a proposal which has been passed.
	type CancellationOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>,
	>;
	// To cancel a proposal before it has been passed.
	type CancelProposalOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 3, 5>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechCommitteeInstance>;
	type CooloffPeriod = ConstU32<{ 7 * DAYS }>;
	type PreimageByteDeposit = ConstU128<{ currency::STORAGE_BYTE_FEE }>;
	type Slash = ();
	type InstantAllowed = ConstBool<true>;
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilInstance>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = ConstU32<100>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const TreasuryId: PalletId = PalletId(*b"py/trsry");

}

type TreasuryApproveOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>,
>;

type TreasuryRejectOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryId;
	type Currency = Balances;
	// At least three-fifths majority of the council is required (or root) to approve a proposal
	type ApproveOrigin = TreasuryApproveOrigin;
	// More than half of the council is required (or root) to reject a proposal
	type RejectOrigin = TreasuryRejectOrigin;
	type Event = Event;
	// If spending proposal rejected, transfer proposer bond to treasury
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ConstU128<{ 1 * currency::SOLAR * currency::SUPPLY_FACTOR }>;
	type SpendPeriod = ConstU32<{ 6 * DAYS }>;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = ConstU32<100>;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type SpendFunds = ();
	type ProposalBondMaximum = ();
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 10 * currency::SOLAR;
}

impl pallet_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BlockNumberToBalance = ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = ();
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

parameter_types! {
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 5 * currency::SOLAR;
	pub const BountyDepositBase: Balance = 1 * currency::SOLAR;
	pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);
	pub const CuratorDepositMin: Balance = 1 * currency::SOLAR;
	pub const CuratorDepositMax: Balance = 100 * currency::SOLAR;
	pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const DataDepositPerByte: Balance = 1 * currency::SOLAR/100;
	pub const MaximumReasonLength: u32 = 300;
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * currency::SOLAR;
}

impl pallet_bounties::Config for Runtime {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type CuratorDepositMultiplier = CuratorDepositMultiplier;
	type CuratorDepositMin = CuratorDepositMin;
	type CuratorDepositMax = CuratorDepositMax;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
	type ChildBountyManager = ChildBounties;
}

parameter_types! {
	pub const ChildBountyValueMinimum: Balance = 1 * currency::SOLAR;
}

impl pallet_child_bounties::Config for Runtime {
	type Event = Event;
	type MaxActiveChildBountyCount = ConstU32<5>;
	type ChildBountyValueMinimum = ChildBountyValueMinimum;
	type WeightInfo = pallet_child_bounties::weights::SubstrateWeight<Runtime>;
}

impl pallet_tips::Config for Runtime {
	type Event = Event;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type Tippers = Elections;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type WeightInfo = pallet_tips::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
	pub const MaxAuthorities: u32 = 100;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type Event = Event;
	type ValidatorSet = Historical;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
	type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
}

impl pallet_offences::Config for Runtime {
	type Event = Event;
	//type IdentificationTuple =();
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = ();
}

impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}

mod voter_bags;

parameter_types! {
	pub const BagThresholds: &'static [u64] = &voter_bags::THRESHOLDS;
}

impl pallet_bags_list::Config for Runtime {
	type Event = Event;
	type ScoreProvider = Staking;
	type WeightInfo = pallet_bags_list::weights::SubstrateWeight<Runtime>;
	type BagThresholds = BagThresholds;
	type Score = VoteWeight;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system =0,
		Aura: pallet_aura=1,
		Timestamp: pallet_timestamp=2,
		Balances: pallet_balances=3,
		Authorship: pallet_authorship=4,
		Staking: pallet_staking=5,
		Offences: pallet_offences=6,
		Historical: pallet_session_historical::{Pallet}=7,
		Session: pallet_session=8,
		Grandpa: pallet_grandpa=9,
		ImOnline: pallet_im_online=10,
		AuthorityDiscovery: pallet_authority_discovery=11,
		// Governance stuff.
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>}=12,
		//Council stuff.
		Council:
			pallet_collective::<Instance1>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>}=13,
		TechCommittee:
			pallet_collective::<Instance2>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>}=14,
		Elections: pallet_elections_phragmen=15,
		TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} =16 ,
		//Treasury stuff.
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} =17,
		Utility: pallet_utility=18,
		// Vesting stuff.
		Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>}=19 ,
		Scheduler: pallet_scheduler=20,
		Multisig: pallet_multisig=21,
		Preimage: pallet_preimage=22,
		Bounties: pallet_bounties=23,
		ChildBounties: pallet_child_bounties=24,
		Tips: pallet_tips=25,
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase=26,
		BagsList: pallet_bags_list=27,
		NominationPools: pallet_nomination_pools=28,

		NodeAuthorization: pallet_node_authorization=29,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip =30,
		Sudo: pallet_sudo=31,
		Contracts: pallet_contracts=32,
		TransactionPayment: pallet_transaction_payment=33,

		// Ethereum stuff.
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, Origin}=34,
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>}=35,
		DynamicFee: pallet_dynamic_fee::{Pallet, Call, Storage, Config, Inherent}=36,
		BaseFee: pallet_base_fee::{Pallet, Call, Storage, Config<T>, Event}=37,
		HotfixSufficients: pallet_hotfix_sufficients::{Pallet, Call}=38,
	}
);

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
// /// The payload being signed in transactions.
// pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
// /// Unchecked extrinsic type as expected by this runtime.
// pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = fp_self_contained::CheckedExtrinsic<AccountId, Call, SignedExtra, H160>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

impl_runtime_apis_plus_common! {
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			xt: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			// Filtered calls should not enter the tx pool as they'll fail if inserted.
			// If this call is not allowed, we return early.
			if !<Runtime as frame_system::Config>::BaseCallFilter::contains(&xt.0.function) {
				return InvalidTransaction::Call.into();
			}

			// This runtime uses Substrate's pallet transaction payment. This
			// makes the chain feel like a standard Substrate chain when submitting
			// frame transactions and using Substrate ecosystem tools. It has the downside that
			// transaction are not prioritized by gas_price. The following code reprioritizes
			// transactions to overcome this.
			//
			// A more elegant, ethereum-first solution is
			// a pallet that replaces pallet transaction payment, and allows users
			// to directly specify a gas price rather than computing an effective one.
			// #HopefullySomeday

			// First we pass the transactions to the standard FRAME executive. This calculates all the
			// necessary tags, longevity and other properties that we will leave unchanged.
			// This also assigns some priority that we don't care about and will overwrite next.
			let mut intermediate_valid = Executive::validate_transaction(source, xt.clone(), block_hash)?;

			let dispatch_info = xt.get_dispatch_info();

			// If this is a pallet ethereum transaction, then its priority is already set
			// according to gas price from pallet ethereum. If it is any other kind of transaction,
			// we modify its priority.
			Ok(match &xt.0.function {
				Call::Ethereum(transact { .. }) => intermediate_valid,
				_ if dispatch_info.class != DispatchClass::Normal => intermediate_valid,
				_ => {
					let tip = match xt.0.signature {
						None => 0,
						Some((_, _, ref signed_extra)) => {
							// Yuck, this depends on the index of charge transaction in Signed Extra
							let charge_transaction = &signed_extra.7;
							charge_transaction.tip()
						}
					};

					// Calculate the fee that will be taken by pallet transaction payment
					let fee: u64 = TransactionPayment::compute_fee(
						xt.encode().len() as u32,
						&dispatch_info,
						tip,
					).saturated_into();

					// Calculate how much gas this effectively uses according to the existing mapping
					let effective_gas =
						<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
							dispatch_info.weight
						);

					// Here we calculate an ethereum-style effective gas price using the
					// current fee of the transaction. Because the weight -> gas conversion is
					// lossy, we have to handle the case where a very low weight maps to zero gas.
					let effective_gas_price = if effective_gas > 0 {
						fee / effective_gas
					} else {
						// If the effective gas was zero, we just act like it was 1.
						fee
					};

					// Overwrite the original prioritization with this ethereum one
					intermediate_valid.priority = effective_gas_price;
					intermediate_valid
				}
			})
		}
	}
}

impl_self_contained_call!();
