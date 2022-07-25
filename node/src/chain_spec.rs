use hex_literal::hex;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::{ChainSpecExtension, Properties};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use solar_node_runtime::{
	currency,
	opaque::SessionKeys,
	wasm_binary_unwrap,
	AccountId,
	BabeConfig,
	BalancesConfig,
	BaseFeeConfig,
	Block,
	CouncilConfig,
	Days,
	EVMConfig,
	ElectionsConfig,
	EpochDurationInBlocks,
	EpochDurationInSlots,
	EthereumConfig,
	GenesisConfig,
	GrandpaConfig,
	Hours,
	IndicesConfig,
	MillisecsPerBlock,
	Minutes,
	NodeAuthorizationConfig,
	NominationPoolsConfig,
	Permill,
	SecsPerBlock,
	SessionConfig,
	Signature,
	SlotDuration,
	StakerStatus,
	StakingConfig,
	SudoConfig,
	SystemConfig,
	TechnicalMembershipConfig,
	//TechnicalCommitteeConfig,
	MILLISECS_PER_BLOCK,
};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::crypto::UncheckedInto;
use sp_core::{crypto::AccountId32, sr25519, OpaquePeerId, Pair, Public, H160, U256}; /* A struct wraps Vec<u8>, represents as our `PeerId`. */
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
				EpochDurationInBlocks::set(&hours);

				EpochDurationInSlots::set(&(hours as u64));
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
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
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
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller session; do ./phala-node key inspect
	// "$secret"//phat//$j//$i; done; done and
	// for i in 1 2 3 4 ; do for j in session; do ./phala-node key inspect --scheme ed25519
	// "$secret"//phat//$j//$i; done; done
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
			// 5DXHAev6Kht2Q4Sjk4sFqNuRh4nAUa7bNk53scQJ2WDCq8x6
			hex!["4080bee9f2a76a4e0e0d7d27d1e91a4793ad8162b231e56f7797e8abfec7a735"].into(),
			// Controller
			// 5ECZDrEERBNPKtkrPJ9C3R9DgtBzWrDwbHauzUs28UroBTw2
			hex!["5e755cde5d8a46543a3a3a44cb887f82ffa83f6ae2f23a03ea7577e59e1f2976"].into(),
			// Session key ed25519
			// 5HfQhACBgihgDAKzdpTTJHuheRtGMjgBDf5MxUQ26VwTHDik
			hex!["f7a4fa3658cc703ae9049b524dc3925e48d45e2e9fa941d500a28f2d247763b3"]
				.unchecked_into(),
			// Session key sr25519
			// 5EyYhJTkXyRdij7DtogyrZkPKSYjxNXX86RfifrapECcFqzp
			hex!["80c5a20b5e848c3f0409190860215c96e10195d6ed53b1f4029939d86bccb56e"]
				.unchecked_into(),
			hex!["80c5a20b5e848c3f0409190860215c96e10195d6ed53b1f4029939d86bccb56e"]
				.unchecked_into(),
			hex!["80c5a20b5e848c3f0409190860215c96e10195d6ed53b1f4029939d86bccb56e"]
				.unchecked_into(),
		),
		(
			// Stash
			// 5E4k6rGMfVWoK56KFxHyMgFLWKTnjsjrEbAo6o7S1yC4uEFP
			hex!["588005c8a8175f09f2bd51a4d37fcee4325b596316482d45a95981f71f7c9d5e"].into(),
			// Controller
			// 5GhLmXeL5tG5ZPZ5sU1o5fZ7ZnXUcq6mwRAaJr8kJbFXXQks
			hex!["cce205ee8d7f5d9a735bdd96dae735398e6ffe084cf63d74fd442425c4595f57"].into(),
			// Session key ed25519
			// 5CpyfUqT6DcBvVok1vFNhNUvgnbvhJdmTsNtqAo8NnNsfTLz
			hex!["21c4025dd9d433e3792d245b0b5f92509badbe22b0d0d8e188557262f1182c56"]
				.unchecked_into(),
			// Session key sr25519
			// 5GBdn89iNNYxzHWgymoAdY8Wex17R29XMWDansq6yWnkhLNM
			hex!["b639ec343a1aa1d24e90e9ef8e6bde89f1b534eb42b97d4a2817499368cc7f7b"]
				.unchecked_into(),
			hex!["b639ec343a1aa1d24e90e9ef8e6bde89f1b534eb42b97d4a2817499368cc7f7b"]
				.unchecked_into(),
			hex!["b639ec343a1aa1d24e90e9ef8e6bde89f1b534eb42b97d4a2817499368cc7f7b"]
				.unchecked_into(),
		),
		(
			// Stash
			// 5C7o1GJZRDPmTSNsghdze4Txr1nFWAMKWxHUUQ84ZUbFu5Pn
			hex!["025b110dcd50fd36de3596c260192985f8646705683e765c91627eb28a74e770"].into(),
			// Controller
			// 5EqsSN4Zxsin5xBucWdP9vQDXoagfvY4sCU9LTh4kHkqWW2t
			hex!["7aeac3309b6d7d4dd394d330eba2ade536d623a8ab1610c7f1ad41dc74eafe16"].into(),
			// Session key ed25519
			// 5CsefRq3LDSThqRjWzVAPoLy9E3Q9vtRHDdYUXC7EXkyM7eE
			hex!["23cdc8621cfad1645ad1323ee25c8e4efb7f8baafdc57de78041eb8426b77396"]
				.unchecked_into(),
			// Session key sr25519
			// 5FnH8ti3mP3wNTZgn9A7hSvq5PJ4xBpaCgnRmh2uB7jf6nSn
			hex!["a469ca9c8a2ab060584028762e465ca6f509e33d83b6e91055da5e4020692133"]
				.unchecked_into(),
			hex!["a469ca9c8a2ab060584028762e465ca6f509e33d83b6e91055da5e4020692133"]
				.unchecked_into(),
			hex!["a469ca9c8a2ab060584028762e465ca6f509e33d83b6e91055da5e4020692133"]
				.unchecked_into(),
		),
		(
			// Stash
			// 5D85jACGvGCtAm5jWBTC7Rp4kF2UUax2AXowdr922EuXcgpQ
			hex!["2ecf97a91ab4985999ee0a25ef9ecc0e18f42b9cee34e47771d5a08e6eb5ea39"].into(),
			// Controller
			// 5GZVcXsUY3uqF567quAMycDn3as5fMqq8K4rTF6KLJvrbQvM
			hex!["c6e5d5a23f730d922762a46372144a1771ee1ed4f29e55e73c3f82fbbf3a1a4e"].into(),
			// Session key ed25519
			// 5ETyA4Kz4cGXWAi72aRn2NLEzBwQAaDgsqBfYon8TNV6gArL
			hex!["6a369d6f98d4cbda264eb2fa4506d381a28c545e2065413b9119767d8e6a779a"]
				.unchecked_into(),
			// Session key sr25519
			// 5CMQbZeiDgFA1rdwxPYBsZ9YdxWFthcsp6S5gdv7EixptPbx
			hex!["0cbd116df0cea2d32d769560a15cc416dcad87ae4852b1eee6720a1c704d2876"]
				.unchecked_into(),
			hex!["0cbd116df0cea2d32d769560a15cc416dcad87ae4852b1eee6720a1c704d2876"]
				.unchecked_into(),
			hex!["0cbd116df0cea2d32d769560a15cc416dcad87ae4852b1eee6720a1c704d2876"]
				.unchecked_into(),
		),
	];

	// generated with secret: ./phala-node key inspect -n phala --scheme Sr25519 "$secret"//phat
	// 46Ndhnw1q15CaTAf1Lu63wpba7hGkMYYTFXUrmndy1tpD2h9
	let root_key: AccountId =
		hex!["fe9d7ef50b53c1362253b398407e6130449d8d69f8434f93d4615bfa5ad21628"].into();

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
		node_authorization: NodeAuthorizationConfig {
			nodes: vec![
				(
					OpaquePeerId(
						bs58::decode("12D3KooWNv7yRKnUzXQGWhFRBhTh24v6LnB6CnZASApHPMn8YoCQ")
							.into_vec()
							.unwrap(),
					),
					AccountId32::from_str("5CyaJGqqD82o6ZQS96dEEWNVhH9t558JobxF1xtxNe37eUJK")
						.unwrap(),
				),
				(
					OpaquePeerId(
						bs58::decode("12D3KooWEQhdRc7Yk4x5p4vwgZmwv64qksfQTPxH6fzMCjhLQNMU")
							.into_vec()
							.unwrap(),
					),
					AccountId32::from_str("5E565XNkfB8vFEKHjqeeQcmcwXbUTBjEi29CvVwXsdMxCUqq")
						.unwrap(),
				),
				(
					OpaquePeerId(
						bs58::decode("12D3KooWHCqfSii1VynLWhN8LfBZvdZqgff97N1iuvQaTZB7dco9")
							.into_vec()
							.unwrap(),
					),
					AccountId32::from_str("5DAbCKqhhZuZXJ7TDnVdn7EMVqdYXvNeppMdLd4YPoFz4U1k")
						.unwrap(),
				),
				(
					OpaquePeerId(
						bs58::decode("12D3KooWSVegbF3mHM13EFQ5WD8rr8YgUFLRtVoxy9inNiVWKtbJ")
							.into_vec()
							.unwrap(),
					),
					AccountId32::from_str("5G3hw8pZFd4gAEssiHxjQBJ4hwefJdgJiDja7fegFeMF1BRw")
						.unwrap(),
				),
				(
					OpaquePeerId(
						bs58::decode("12D3KooWJgdR2tkmk1XtitQ9xiSrtR7kDKzA9miCtKypWNkQvbjH")
							.into_vec()
							.unwrap(),
					),
					AccountId32::from_str("5FyBJGEKY3LHGDHXsqrY7ei7x8GBMXG6pLZoBm2eeQ9cPS4h")
						.unwrap(),
				),
			],
		},
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
