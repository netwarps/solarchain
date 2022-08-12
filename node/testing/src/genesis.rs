// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Genesis Configuration.

use crate::keyring::*;
use node_runtime::{
	constants::currency::*, wasm_binary_unwrap, AccountId, BabeConfig, BalancesConfig,
	GenesisConfig, GrandpaConfig, SessionConfig, StakerStatus,
	StakingConfig, SystemConfig, BABE_GENESIS_EPOCH_CONFIG,
};
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};
use sp_runtime::Perbill;

/// Create genesis runtime configuration for tests.
pub fn config(code: Option<&[u8]>) -> GenesisConfig {
	config_endowed(code, Default::default())
}

/// Create genesis runtime configuration for tests with some extra
/// endowed accounts.
pub fn config_endowed(code: Option<&[u8]>, extra_endowed: Vec<AccountId>) -> GenesisConfig {
	let mut endowed = vec![
		(alice(), 111 * SOLR),
		(bob(), 100 * SOLR),
		(charlie(), 100_000_000 * SOLR),
		(dave(), 111 * SOLR),
		(eve(), 101 * SOLR),
		(ferdie(), 100 * SOLR),
	];

	endowed.extend(extra_endowed.into_iter().map(|endowed| (endowed, 100 * SOLR)));

	GenesisConfig {
		system: SystemConfig {
			code: code.map(|x| x.to_vec()).unwrap_or_else(|| wasm_binary_unwrap().to_vec()),
		},
		balances: BalancesConfig { balances: endowed },
		session: SessionConfig {
			keys: vec![
				(alice(), dave(), to_session_keys(&Ed25519Keyring::Alice, &Sr25519Keyring::Alice)),
				(bob(), eve(), to_session_keys(&Ed25519Keyring::Bob, &Sr25519Keyring::Bob)),
				(
					charlie(),
					ferdie(),
					to_session_keys(&Ed25519Keyring::Charlie, &Sr25519Keyring::Charlie),
				),
			],
		},
		staking: StakingConfig {
			stakers: vec![
				(dave(), alice(), 111 * SOLR, StakerStatus::Validator),
				(eve(), bob(), 100 * SOLR, StakerStatus::Validator),
				(ferdie(), charlie(), 100 * SOLR, StakerStatus::Validator),
			],
			validator_count: 3,
			minimum_validator_count: 0,
			slash_reward_fraction: Perbill::from_percent(10),
			invulnerables: vec![alice(), bob(), charlie()],
			..Default::default()
		},
		babe: BabeConfig { authorities: vec![], epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG) },
		grandpa: GrandpaConfig { authorities: vec![] },
		im_online: Default::default(),
		authority_discovery: Default::default(),
		sudo: Default::default(),
		treasury: Default::default(),
        transaction_payment: Default::default(),
	}
}
