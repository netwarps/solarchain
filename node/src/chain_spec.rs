//! Substrate chain configurations.

use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use node_runtime::{
	constants::currency::*, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
	BalancesConfig, Block, GrandpaConfig,
	ImOnlineConfig, MaxNominations, SessionConfig,
	SessionKeys, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	#[rustfmt::skip]
	// for i in 1 2 3 4 5 ; do for j in stash controller session; do ./target/release/solar-node key inspect
	// "$secret"//solarnetwork//$j//$i; done; done;
	// for i in 1 2 3 4 5 ; do for j in session; do ./target/release/solar-node key inspect --scheme ed25519
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
			hex!["c6b1ceff9077c01c9b8fc774ddf74a34cd7633aa54f53a8e94e23cad888d393c"].into(),
			// Controller
			hex!["6e5706195437b16976aba00a0cb6d068a31cc13b2ad3bd128c73e9cf80c7db5e"].into(),
			// Session key ed25519
			hex!["caa257552e9617fc74d602544e970813f949d1c581ff4fbabd163c24db971666"]
				.unchecked_into(),
			// Session key sr25519
			hex!["026b461233c20fe00024eabc8ebfffbe032b37293bcd1fd5968326e29fe46851"]
				.unchecked_into(),
			hex!["026b461233c20fe00024eabc8ebfffbe032b37293bcd1fd5968326e29fe46851"]
				.unchecked_into(),
			hex!["026b461233c20fe00024eabc8ebfffbe032b37293bcd1fd5968326e29fe46851"]
				.unchecked_into(),
		),
		(
			// Stash
			hex!["6ac457ec91cec8123061c828f8e50c9d37a3668e846770bb75402cf9d4f92d5c"].into(),
			// Controller
			hex!["a2a2e6faf40d2032ee8e4120a79eaa94cff8c476d8fd260ccf8a509bc142f91d"].into(),
			// Session key ed25519
			hex!["5ac8a916f0ec42f0ba9584b9308230c1a32e4bba1b290cd89e923100cad2a46a"]
				.unchecked_into(),
			// Session key sr25519
			hex!["3c8151c1ce0d2870604e67c8c77a8bd287d5def0a6c0f14cac356c4b4636352a"]
				.unchecked_into(),
			hex!["3c8151c1ce0d2870604e67c8c77a8bd287d5def0a6c0f14cac356c4b4636352a"]
				.unchecked_into(),
			hex!["3c8151c1ce0d2870604e67c8c77a8bd287d5def0a6c0f14cac356c4b4636352a"]
				.unchecked_into(),
		),
		(
			// Stash
			hex!["16feb9085b9116ae0606ced37da23540ca8d4f690be4aab46baf29765e023910"].into(),
			// Controller
			hex!["ac3994665383e2d9762a0372622b982f3b3b0322ddc8076323b4c716d3178679"].into(),
			// Session key ed25519
			hex!["b2e9b163dc4c1e1aa9cd27252b961ab679e2c9c837ce28dc2973e286bcd4c290"]
				.unchecked_into(),
			// Session key sr25519
			hex!["b2d342e3aab5e44b8bb84eeaf2be94bc3b6312a8a71273612f7049b4fd063b1f"]
				.unchecked_into(),
			hex!["b2d342e3aab5e44b8bb84eeaf2be94bc3b6312a8a71273612f7049b4fd063b1f"]
				.unchecked_into(),
			hex!["b2d342e3aab5e44b8bb84eeaf2be94bc3b6312a8a71273612f7049b4fd063b1f"]
				.unchecked_into(),
		),
		(
			// Stash
			hex!["2eaa3dc4e7c989870fa8f093bffedb9f2bc290ef8d943ed11741d35af2d50030"].into(),
			// Controller
			hex!["0cb6210c04efaa80a1b28935fc530a2617096d60f5a2f19922de6160f2a4b414"].into(),
			// Session key ed25519
			hex!["8ec8c442566d261226fc0a5d1ebe7557b26f14cac4956c1782491ed40e7d2eb8"]
				.unchecked_into(),
			// Session key sr25519
			hex!["20cc827c11c6546462a33cfb91c18c3cb3db9cdbb6a6af5afb9aa9956f35d668"]
				.unchecked_into(),
			hex!["20cc827c11c6546462a33cfb91c18c3cb3db9cdbb6a6af5afb9aa9956f35d668"]
				.unchecked_into(),
			hex!["20cc827c11c6546462a33cfb91c18c3cb3db9cdbb6a6af5afb9aa9956f35d668"]
				.unchecked_into(),
		),
		(
			// Stash
			hex!["fc8d05dc35bd49de8da6f1404ad50129eef40db2da6eedea526728212c456f2a"].into(),
			// Controller
			hex!["00b7f3b9060e6bf86d1db03511224e9bc83c00340689143b820dc4ed23982177"].into(),
			// Session key ed25519
			hex!["d8937ef85da06cea831ee0002d0d4d98412a5a23dd811c8b944ac3ffd3dd388f"]
				.unchecked_into(),
			// Session key sr25519
			hex!["0890d651314fb612b7a09cabfd579d3857c75c2a7128d0204858be2d7fa8a058"]
				.unchecked_into(),
			hex!["0890d651314fb612b7a09cabfd579d3857c75c2a7128d0204858be2d7fa8a058"]
				.unchecked_into(),
			hex!["0890d651314fb612b7a09cabfd579d3857c75c2a7128d0204858be2d7fa8a058"]
				.unchecked_into(),
		)
	];

	// generated with secret: ./target/release/solar-node key inspect -n substrate --scheme Sr25519 "$secret"//solarnetwork
	let root_key: AccountId =
		hex!["2e948eeada90ef72c244e0f1310308855ee1c2834227e3f1844d20ebf513ea44"].into();

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts))
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Solarnetwork Testnet",
		"solarnetwork_testnet",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Staging telemetry url is valid; qed"),
		),
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"SOLR\", \"SS58Prefix\": 1024}",
			)
				.expect("Provided valid json map"),
		),
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
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

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
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
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	const ENDOWMENT: Balance = 20_000_000 * SOLR;
	const STASH: Balance = ENDOWMENT / 2;

	GenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
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
			stakers,
			..Default::default()
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(node_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		treasury: Default::default(),
		transaction_payment: Default::default(),
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		development_config_genesis,
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"SOLR\", \"SS58Prefix\": 1024}",
			)
				.expect("Provided valid json map"),
		),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		local_testnet_genesis,
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"SOLR\", \"SS58Prefix\": 1024}",
			)
				.expect("Provided valid json map"),
		),
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sp_tracing::try_init_simple();

		sc_service_test::connectivity(integration_test_config_with_two_authorities(), |config| {
			let NewFullBase { task_manager, client, network, transaction_pool, .. } =
				new_full_base(config, false, |_, _| ())?;
			Ok(sc_service_test::TestNetComponents::new(
				task_manager,
				client,
				network,
				transaction_pool,
			))
		});
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}
