use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::{ChainSpecExtension, Properties};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use solar_node_runtime::{
	currency, opaque::SessionKeys, wasm_binary_unwrap, AccountId, BabeConfig, BalancesConfig,
	BaseFeeConfig, Block, CouncilConfig, Days, EVMConfig, ElectionsConfig, EpochDurationInBlocks,
	EpochDurationInSlots, EthereumConfig, GenesisConfig, GrandpaConfig, Hours, IndicesConfig,
	MillisecsPerBlock, Minutes, NominationPoolsConfig, Permill, SecsPerBlock, SessionConfig,
	Signature, SlotDuration, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
	TechnicalMembershipConfig, EPOCH_DURATION_IN_BLOCKS, MILLISECS_PER_BLOCK,
};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public, H160, U256}; /* A struct wraps Vec<u8>, represents as our `PeerId`. */
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};
use std::{collections::BTreeMap, str::FromStr};

/* The genesis config that serves for our pallet. */

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
//pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisExt, Extensions>;

/// Extension for the Phala devnet genesis config to support a custom changes to the genesis state.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct GenesisExt {
	/// The runtime genesis config.
	runtime_genesis_config: GenesisConfig,
	/// The block duration in milliseconds.
	/// If `None` is supplied, the default value is used.
	block_milliseconds: Option<u64>,
}

impl sp_runtime::BuildStorage for GenesisExt {
	fn assimilate_storage(&self, storage: &mut sp_core::storage::Storage) -> Result<(), String> {
		sp_state_machine::BasicExternalities::execute_with_storage(storage, || {
			if let Some(bm) = self.block_milliseconds.as_ref() {
				// change runtime parameter
				MillisecsPerBlock::set(bm);
				let bm_f = *bm as f64;
				let secs_per_block: f64 = bm_f / 1000.0;
				SecsPerBlock::set(&(secs_per_block as u64));

				let minutes = (60.0 / secs_per_block) as u32;
				let hours = minutes * 60;
				let days = hours * 24;

				Minutes::set(&minutes);
				Hours::set(&hours);
				Days::set(&days);
				SlotDuration::set(bm);
				EpochDurationInBlocks::set(&EPOCH_DURATION_IN_BLOCKS);
				EpochDurationInSlots::set(&(EPOCH_DURATION_IN_BLOCKS as u64));
			}
		});
		self.runtime_genesis_config.assimilate_storage(storage)
	}
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	let properties = {
		let mut p = Properties::new();
		p.insert("tokenSymbol".into(), "SOLA".into());
		p.insert("tokenDecimals".into(), 18u32.into());
		p.insert("ss58Format".into(), 42u32.into());
		p
	};
	ChainSpec::from_genesis(
		"SolarNetwork Development",
		"SonarNetwork_development",
		ChainType::Development,
		move || GenesisExt {
			runtime_genesis_config: development_config_genesis(),
			block_milliseconds: Some(MILLISECS_PER_BLOCK),
		},
		// Bootnodes
		vec![],
		// Telemetry
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Default::default(),
	)
}

/// Development config (single validator Alice, custom block duration)
pub fn development_config_custom_block_duration(bd: u64) -> ChainSpec {
	let properties = {
		let mut p = Properties::new();
		p.insert("tokenSymbol".into(), "SOLA".into());
		p.insert("tokenDecimals".into(), 18u32.into());
		p.insert("ss58Format".into(), 42u32.into());
		p
	};
	ChainSpec::from_genesis(
		"SolarNetwork Development",
		"SonarNetwork_development",
		ChainType::Development,
		move || GenesisExt {
			runtime_genesis_config: development_config_genesis(),
			block_milliseconds: Some(bd),
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Default::default(),
	)
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		true,
	)
}

pub fn local_config() -> ChainSpec {
	let properties = {
		let mut p = Properties::new();
		p.insert("tokenSymbol".into(), "SOLA".into());
		p.insert("tokenDecimals".into(), 18u32.into());
		p.insert("ss58Format".into(), 42u32.into());
		p
	};

	ChainSpec::from_genesis(
		"SolarNetwork Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || GenesisExt {
			runtime_genesis_config: local_genesis(),
			block_milliseconds: Some(MILLISECS_PER_BLOCK),
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(properties),
		Default::default(),
	)
}

fn local_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob"), authority_keys_from_seed("Charlie")],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		false,
	)
}
pub fn devnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../spec/dev_network/customSpecRaw.json")[..])
}

pub fn testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../spec/test_network/customSpecRaw.json")[..])
}

pub fn local_testnet_config() -> ChainSpec {
	let properties = {
		let mut p = Properties::new();
		p.insert("tokenSymbol".into(), "SOLA".into());
		p.insert("tokenDecimals".into(), 18u32.into());
		p.insert("ss58Format".into(), 42u32.into());
		p
	};

	ChainSpec::from_genesis(
		"SolarNetwork PoC-5",
		"SolarNetwork_poc_5",
		ChainType::Local,
		move || GenesisExt {
			runtime_genesis_config: testnet_local_config_genesis(),
			block_milliseconds: Some(MILLISECS_PER_BLOCK),
		},
		// Bootnodes
		vec![],
		// Telemetry
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Default::default(),
	)
}

fn testnet_local_config_genesis() -> GenesisConfig {
	// secret='deal juice like gown parade pyramid armed pact risk south bubble type'
	// for i in 1 2 3 4 ; do for j in stash controller session; do ./target/release/solar-node key inspect
	// "$secret"//solarnetwork//$j//$i; done; done;
	// for i in 1 2 3 4 ; do for j in session; do ./target/release/solar-node key inspect --scheme ed25519
	// "$secret"//solarnetwork//$j//$i; done; done;
	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)> = vec![
		(
			// Stash
			// 5CZifRVA2UsHiwp3yxF8mCHPDaqwhmUwmNvriD1GJwcQnfxN
			// seed:0x31d4cdedf5683cc1668df28a00aa90c730477053b6559edb26d4469b8976e756
			hex!["1620d4903b682ba160049765471471328fa1db784a3f899724872c9606a79319"].into(),
			// Controller
			// 5EjhNWr5J3WA5jkSQWhA4BEF4yMK6gWD68G4S91rSXrnpcAs
			// seed:0xc762b0bad00042ef354b5b4db94e99858d4724664d1fd8ef5dce718f0ca97c1b
			hex!["763567569ede84779c1922759b14451046f8a4ae925bcec10077061a8ca72407"].into(),
			// Session key ed25519
			// 5DBAdtip6jTLcX5D9QkGUTz1GtEyxyweHi1ae8HV1bk2S3iW
			// seed:0xf8c5f2eecc1c0b6f52f55bcb1263770067489b403c3d56aab48ee06c9f513e44
			hex!["3129dabd9339c78294b83e2584be8d88e8560f9f1b048ce1e61956c55147ddd8"]
				.unchecked_into(),
			// Session key sr25519
			// 5GWm4ZjR7xWc8zb7jVNyNx7vtzqVkyFr3wgU2JiLYuBGZ9hm
			// seed:0x84ab20f0b7d449be9df715653ffbf0c239a22eafc43486fe6f2a6d492a0c06f3
			hex!["c4d01a8b5290a111cf758cf90bb6bd944c22f774010d63413445e5f56cf68b42"]
				.unchecked_into(),
			hex!["c4d01a8b5290a111cf758cf90bb6bd944c22f774010d63413445e5f56cf68b42"]
				.unchecked_into(),
			hex!["c4d01a8b5290a111cf758cf90bb6bd944c22f774010d63413445e5f56cf68b42"]
				.unchecked_into(),
		),
		(
			// Stash
			// 5D7HdwqNzMn83hiFX2qQ3uNmYB1ufuu3FTTr8vBWBbQTGgmo
			// seed:0xe83241ced79fde2ead6e84c0cb25367c4715336eb3659b4ef48c95373be5a6f2
			hex!["2e3470a75aa259bba2f50d6682b13cbff9ef374cd4641106b2052e6bcfdfc44d"].into(),
			// Controller
			// 5EXDNNneMX34weBpJCfinvmQkvfBb2iqK7jQHSKRctR4HLGC
			// seed:0xec36ab85f450eb6af76eec936cbab41192300af78de0c6614c16f03f552f1a41
			hex!["6cb031c0af353d847c64152ed07c33f6a51604ba399ba22aa9abba4f83059b20"].into(),
			// Session key ed25519
			// 5E15bSPS8F5horeaN9bzYFZ57dbPk3GYwwYqnw6sYJsqCiXf
			// seed:0x678ea74352c4517f07b7b484dee092af0f407fa7ec7945fe0d4c1b82b2534860
			hex!["55b4a8f15a0f0cc2b88d5fd6b32c1980d78f0234cd0ed86ac2ba9ea2418676e4"]
				.unchecked_into(),
			// Session key sr25519
			// 5F7N4mWqxfg7EfBPqscjWpq4od1LYMD7MCAPHdBPuWQRP1uT
			// seed:0xa2b73edd73d3b95342f6e9b2bf81fe1c97a9fc5705d0ec118bd7cb00f8cd1f96
			hex!["86bbd01547021f5cee14c90baefe0c7c9caf83b51bbfff4cd4f7d384e334b443"]
				.unchecked_into(),
			hex!["86bbd01547021f5cee14c90baefe0c7c9caf83b51bbfff4cd4f7d384e334b443"]
				.unchecked_into(),
			hex!["86bbd01547021f5cee14c90baefe0c7c9caf83b51bbfff4cd4f7d384e334b443"]
				.unchecked_into(),
		),
		(
			// Stash
			// 5D2oTrNLjKCk6kcRBMPFKmRWDMddP4jN5SY21mscG9E3M5u1
			// seed:0x2de2944e812b2d67cdbbb601c8cd054bade8ced5831aa5033a59b6cc46c2ab14
			hex!["2ac89dbd2885e7b41c2bcf80314de6cc0bc8fb4e9984f7b357b5fbe4f9018326"].into(),
			// Controller
			// 5DnuMdVhksw4JtWHUAtc71MCYpwHh1r93jMEfLeDh28ZKyWE
			// seed:0x2b4c0301bebbb78f8c86473e067aced9b9486d72cfbed7eb6cf82e37162e3b4a
			hex!["4c6b4029f3d0cc3fe1991e6e9a298b58084ea150c2b17da5e33c75c5da649045"].into(),
			// Session key ed25519
			// 5FYKgcrVkd7Ln7JWTVQ8fTAuSTcMuA3YvxdPHG9FTX6GmFJP
			// seed:0xc7f1c2fe41e678a5221ea15f50a7c4ce859d6df1cb9aa0e23f90368df84dbb14
			hex!["99c4ee06fb09d3142e0ace10998dbfd54282ea1ce03547ef440abc16bc2d0b02"]
				.unchecked_into(),
			// Session key sr25519
			// 5DSVW3z3fj9x8jMyZSiq1dScBkyAgoB4jY3jNnUGvKiYCVto
			// seed:0xde1b20b5a2f4af226a6e756b7105b4c2b3a29041d82abdea0f96d9b1eacae706
			hex!["3cda0a48d4211d2670f21471f47b880d58155a3feb564fab77528d20e0d5a87c"]
				.unchecked_into(),
			hex!["3cda0a48d4211d2670f21471f47b880d58155a3feb564fab77528d20e0d5a87c"]
				.unchecked_into(),
			hex!["3cda0a48d4211d2670f21471f47b880d58155a3feb564fab77528d20e0d5a87c"]
				.unchecked_into(),
		),
		(
			// Stash
			// 5CrB9AD2WfDwXxgqfCgiNEGXYH8WfKYfnqapxVE3L41aFBs1
			// seed:0x881fb6694ac516451eb708ac051aaee45894b7e321bf487ed5cd4f9d2bdb35bd
			hex!["22ade3f286694c41e3d46a9c6e079fd0ba166dd5e2d650f6cab16d06c828a27c"].into(),
			// Controller
			// 5DXQhJxFfw2xonhdpDNHmVwDeh6Dk1bESUj9GxBNbt5KDrXR
			// seed:0x2bfeec12fb6242471578a5203e8133d51ab45161598508a3b9ff5c5a653944da
			hex!["409a16cff6503509ac9f7738ba572bccb649eb8b1c4aecde36c0a6df39e39338"].into(),
			// Session key ed25519
			// 5EiCXsd3JWLcijeN9qEmarbnexgGCd1dffWCxfMiR1Fd51px
			// seed:0x08598a3dac249a70ebf14f042a3b7b3604f467a6f79d6a9d219877c3ea0b338d
			hex!["751114260814036c3edba6654007cb50a549c7f894e0ae256520bc4060b098bf"]
				.unchecked_into(),
			// Session key sr25519
			// 5G8jrXUCawH8dMdGhwdWR1AtwmX4wh5WyFfcvCuTzX6cKtCg
			// seed:0x70a0ece4231112a4d73f328df6ccd189122a97e8835466907f738b0b5dc5968b
			hex!["b404a3c6d35dd2164be6aa4a1074449d253eeb25faa29d9d02e6502c66b3fe0c"]
				.unchecked_into(),
			hex!["b404a3c6d35dd2164be6aa4a1074449d253eeb25faa29d9d02e6502c66b3fe0c"]
				.unchecked_into(),
			hex!["b404a3c6d35dd2164be6aa4a1074449d253eeb25faa29d9d02e6502c66b3fe0c"]
				.unchecked_into(),
		),
	];

	// generated with secret: ./target/release/solar-node key inspect -n substrate --scheme Sr25519 "$secret"//solarnetwork
	// 5HphK5aUnkYH1L68DNKZJ8Zgw2fSc7tKkZtSjNrrzbwEvzdR
	// seed: 0x2370746c4694bebd5651ae9db351925619c961e526bbfe36e6615862f802eac6
	let root_key: AccountId =
		hex!["feba21b33b224cc703ad74e3f19410d9794b46ddb71ee6b69daebb08b0eff02a"].into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, root_key, Some(endowed_accounts), false)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	_enable_println: bool,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});

	initial_authorities.iter().for_each(|x| {
		if !endowed_accounts.contains(&x.0) {
			endowed_accounts.push(x.0.clone())
		}
	});
	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: u128 = 100000000 * currency::UNIT;
	const STASH: u128 = ENDOWMENT / 1000;

	GenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: IndicesConfig { indices: vec![] },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(solar_node_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: GrandpaConfig { authorities: vec![] },
		sudo: SudoConfig { key: Some(root_key) },
		transaction_payment: Default::default(),
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					H160::from_str("0xF833A151AA2623122625C65834Da5216fa28b927")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: (10000000 * currency::UNIT).into(),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
		},
		ethereum: EthereumConfig {},
		dynamic_fee: Default::default(),
		base_fee: BaseFeeConfig::new(
			U256::from(1_000_000_000u64),
			false,
			Permill::from_parts(125_000),
		),
		treasury: Default::default(),
		vesting: Default::default(),
		democracy: Default::default(),
		council: CouncilConfig::default(),
		technical_committee: Default::default(),
		technical_membership: TechnicalMembershipConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		authority_discovery: Default::default(),
		im_online: Default::default(),
		nomination_pools: NominationPoolsConfig {
			min_create_bond: 10 * currency::UNIT,
			min_join_bond: 1 * currency::UNIT,
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			..Default::default()
		},
	}
}
