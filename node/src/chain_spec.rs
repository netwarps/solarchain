use sc_service::ChainType;
use solar_node_runtime::{
	currency::UNIT, AccountId, AuraConfig, BalancesConfig, EVMConfig, EthereumConfig,
	GenesisConfig, GrandpaConfig, NodeAuthorizationConfig, Signature, SudoConfig, SystemConfig,
	WASM_BINARY,Permill
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::AccountId32, sr25519, OpaquePeerId, Pair, Public, H160, U256}; /* A struct wraps Vec<u8>, represents as our `PeerId`. */
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::{collections::BTreeMap, str::FromStr};
use solar_node_runtime::BaseFeeConfig;
/* The genesis config that serves for our pallet. */

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
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
				],
				true,
			)
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
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"SOLA\", \"SS58Prefix\": 42}",
			)
			.expect("Provided valid json map"),
		),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
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
				],
				true,
			)
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
		Some(
			serde_json::from_str(
				"{\"tokenDecimals\": 18, \"tokenSymbol\": \"SOLA\", \"SS58Prefix\": 42}",
			)
			.expect("Provided valid json map"),
		),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	//let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 100000000 * UNIT)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
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
						balance: (10000000 * UNIT).into(),
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
	}
}
