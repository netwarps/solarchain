// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

#[macro_export]
macro_rules! impl_runtime_apis_plus_common {
	{$($custom:tt)*} => {
		impl_runtime_apis! {
			$($custom)*

			impl sp_api::Core<Block> for Runtime {
				fn version() -> RuntimeVersion {
					VERSION
				}

				fn execute_block(block: Block) {
					Executive::execute_block(block);
				}

				fn initialize_block(header: &<Block as BlockT>::Header) {
					Executive::initialize_block(header)
				}
			}

			impl sp_api::Metadata<Block> for Runtime {
				fn metadata() -> OpaqueMetadata {
					OpaqueMetadata::new(Runtime::metadata().into())
				}
			}

			impl sp_block_builder::BlockBuilder<Block> for Runtime {
				fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
					Executive::apply_extrinsic(extrinsic)
				}

				fn finalize_block() -> <Block as BlockT>::Header {
					Executive::finalize_block()
				}

				fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
					data.create_extrinsics()
				}

				fn check_inherents(
					block: Block,
					data: sp_inherents::InherentData,
				) -> sp_inherents::CheckInherentsResult {
					data.check_extrinsics(&block)
				}
			}

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

			impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
				fn offchain_worker(header: &<Block as BlockT>::Header) {
					Executive::offchain_worker(header)
				}
			}

			impl sp_consensus_babe::BabeApi<Block> for Runtime {
				fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
					// The choice of `c` parameter (where `1 - c` represents the
					// probability of a slot being empty), is done in accordance to the
					// slot duration and expected target block time, for safely
					// resisting network delays of maximum two seconds.
					// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
					sp_consensus_babe::BabeGenesisConfiguration {
						slot_duration: Babe::slot_duration(),
						epoch_length: EpochDuration::get(),
						c: BABE_GENESIS_EPOCH_CONFIG.c,
						genesis_authorities: Babe::authorities().to_vec(),
						randomness: Babe::randomness(),
						allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
					}
				}

				fn current_epoch_start() -> sp_consensus_babe::Slot {
					Babe::current_epoch_start()
				}

				fn current_epoch() -> sp_consensus_babe::Epoch {
					Babe::current_epoch()
				}

				fn next_epoch() -> sp_consensus_babe::Epoch {
					Babe::next_epoch()
				}

				fn generate_key_ownership_proof(
					_slot: sp_consensus_babe::Slot,
					authority_id: sp_consensus_babe::AuthorityId,
				) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
					use codec::Encode;

					Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
						.map(|p| p.encode())
						.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
				}

				fn submit_report_equivocation_unsigned_extrinsic(
					equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
					key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
				) -> Option<()> {
					let key_owner_proof = key_owner_proof.decode()?;

					Babe::submit_unsigned_equivocation_report(
						equivocation_proof,
						key_owner_proof,
					)
				}
			}


			impl sp_session::SessionKeys<Block> for Runtime {
				fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
					opaque::SessionKeys::generate(seed)
				}

				fn decode_session_keys(
					encoded: Vec<u8>,
				) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
					opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
				}
			}


			impl fg_primitives::GrandpaApi<Block> for Runtime {
				fn grandpa_authorities() -> GrandpaAuthorityList {
					Grandpa::grandpa_authorities()
				}

				fn current_set_id() -> fg_primitives::SetId {
					Grandpa::current_set_id()
				}

				fn submit_report_equivocation_unsigned_extrinsic(
					_equivocation_proof: fg_primitives::EquivocationProof<
						<Block as BlockT>::Hash,
						NumberFor<Block>,
					>,
					_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
				) -> Option<()> {
					None
				}

				fn generate_key_ownership_proof(
					_set_id: fg_primitives::SetId,
					_authority_id: GrandpaId,
				) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
					// NOTE: this is the only implementation possible since we've
					// defined our key owner proof type as a bottom type (i.e. a type
					// with no values).
					None
				}
			}


			// impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
			// 	fn authorities() -> Vec<AuthorityDiscoveryId> {
			// 		AuthorityDiscovery::authorities()
			// 	}
			// }

			impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
				fn account_nonce(account: AccountId) -> Index {
					System::account_nonce(account)
				}
			}

			impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
				fn chain_id() -> u64 {
					<Runtime as pallet_evm::Config>::ChainId::get()
				}

				fn account_basic(address: H160) -> EVMAccount {
					let (account, _) = EVM::account_basic(&address);
					account
				}

				fn gas_price() -> U256 {
					let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
					gas_price
				}

				fn account_code_at(address: H160) -> Vec<u8> {
					EVM::account_codes(address)
				}

				fn author() -> H160 {
					<pallet_evm::Pallet<Runtime>>::find_author()
				}

				fn storage_at(address: H160, index: U256) -> H256 {
					let mut tmp = [0u8; 32];
					index.to_big_endian(&mut tmp);
					EVM::account_storages(address, H256::from_slice(&tmp[..]))
				}

				fn call(
					from: H160,
					to: H160,
					data: Vec<u8>,
					value: U256,
					gas_limit: U256,
					max_fee_per_gas: Option<U256>,
					max_priority_fee_per_gas: Option<U256>,
					nonce: Option<U256>,
					estimate: bool,
					access_list: Option<Vec<(H160, Vec<H256>)>>,
				) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
					let config = if estimate {
						let mut config = <Runtime as pallet_evm::Config>::config().clone();
						config.estimate = true;
						Some(config)
					} else {
						None
					};

					let is_transactional = false;
					<Runtime as pallet_evm::Config>::Runner::call(
						from,
						to,
						data,
						value,
						gas_limit.low_u64(),
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
					).map_err(|err| err.error.into())
				}

				fn create(
					from: H160,
					data: Vec<u8>,
					value: U256,
					gas_limit: U256,
					max_fee_per_gas: Option<U256>,
					max_priority_fee_per_gas: Option<U256>,
					nonce: Option<U256>,
					estimate: bool,
					access_list: Option<Vec<(H160, Vec<H256>)>>,
				) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
					let config = if estimate {
						let mut config = <Runtime as pallet_evm::Config>::config().clone();
						config.estimate = true;
						Some(config)
					} else {
						None
					};

					let is_transactional = false;
					<Runtime as pallet_evm::Config>::Runner::create(
						from,
						data,
						value,
						gas_limit.low_u64(),
						max_fee_per_gas,
						max_priority_fee_per_gas,
						nonce,
						access_list.unwrap_or_default(),
						is_transactional,
						config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
					).map_err(|err| err.error.into())
				}

				fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
					Ethereum::current_transaction_statuses()
				}

				fn current_block() -> Option<pallet_ethereum::Block> {
					Ethereum::current_block()
				}

				fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
					Ethereum::current_receipts()
				}

				fn current_all() -> (
					Option<pallet_ethereum::Block>,
					Option<Vec<pallet_ethereum::Receipt>>,
					Option<Vec<TransactionStatus>>
				) {
					(
						Ethereum::current_block(),
						Ethereum::current_receipts(),
						Ethereum::current_transaction_statuses()
					)
				}

				fn extrinsic_filter(
					xts: Vec<<Block as BlockT>::Extrinsic>,
				) -> Vec<EthereumTransaction> {
					xts.into_iter().filter_map(|xt| match xt.0.function {
						Call::Ethereum(transact { transaction }) => Some(transaction),
						_ => None
					}).collect::<Vec<EthereumTransaction>>()
				}

				fn elasticity() -> Option<Permill> {
					Some(BaseFee::elasticity())
				}
			}

			impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
				fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
					UncheckedExtrinsic::new_unsigned(
						pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
					)
				}
			}

			impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
				fn query_info(
					uxt: <Block as BlockT>::Extrinsic,
					len: u32,
				) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
					TransactionPayment::query_info(uxt, len)
				}
				fn query_fee_details(
					uxt: <Block as BlockT>::Extrinsic,
					len: u32,
				) -> pallet_transaction_payment::FeeDetails<Balance> {
					TransactionPayment::query_fee_details(uxt, len)
				}
			}

			#[cfg(feature = "runtime-benchmarks")]
			impl frame_benchmarking::Benchmark<Block> for Runtime {
				fn benchmark_metadata(extra: bool) -> (
					Vec<frame_benchmarking::BenchmarkList>,
					Vec<frame_support::traits::StorageInfo>,
				) {
					use frame_benchmarking::{list_benchmark, baseline, Benchmarking, BenchmarkList};
					use frame_support::traits::StorageInfoTrait;
					use frame_system_benchmarking::Pallet as SystemBench;
					use baseline::Pallet as BaselineBench;

					let mut list = Vec::<BenchmarkList>::new();

					list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
					list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
					list_benchmark!(list, extra, pallet_balances, Balances);
					list_benchmark!(list, extra, pallet_timestamp, Timestamp);

					let storage_info = AllPalletsWithSystem::storage_info();

					(list, storage_info)
				}

				fn dispatch_benchmark(
					config: frame_benchmarking::BenchmarkConfig
				) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
					use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

					use frame_system_benchmarking::Pallet as SystemBench;
					use baseline::Pallet as BaselineBench;

					impl frame_system_benchmarking::Config for Runtime {}
					impl baseline::Config for Runtime {}

					let whitelist: Vec<TrackedStorageKey> = vec![
						// Block Number
						hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
						// Total Issuance
						hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
						// Execution Phase
						hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
						// Event Count
						hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
						// System Events
						hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
					];

					let mut batches = Vec::<BenchmarkBatch>::new();
					let params = (&config, &whitelist);

					add_benchmark!(params, batches, frame_benchmarking, BaselineBench::<Runtime>);
					add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
					add_benchmark!(params, batches, pallet_balances, Balances);
					add_benchmark!(params, batches, pallet_timestamp, Timestamp);

					Ok(batches)
				}
			}

			impl pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash>
				for Runtime
			{
				fn call(
					origin: AccountId,
					dest: AccountId,
					value: Balance,
					gas_limit: u64,
					storage_deposit_limit: Option<Balance>,
					input_data: Vec<u8>,
				) -> pallet_contracts_primitives::ContractExecResult<Balance> {
					Contracts::bare_call(origin, dest, value, gas_limit, storage_deposit_limit, input_data, CONTRACTS_DEBUG_OUTPUT)
				}

				fn instantiate(
					origin: AccountId,
					value: Balance,
					gas_limit: u64,
					storage_deposit_limit: Option<Balance>,
					code: pallet_contracts_primitives::Code<Hash>,
					data: Vec<u8>,
					salt: Vec<u8>,
				) -> pallet_contracts_primitives::ContractInstantiateResult<AccountId, Balance>
				{
					Contracts::bare_instantiate(origin, value, gas_limit, storage_deposit_limit, code, data, salt, CONTRACTS_DEBUG_OUTPUT)
				}

				fn upload_code(
					origin: AccountId,
					code: Vec<u8>,
					storage_deposit_limit: Option<Balance>,
				) -> pallet_contracts_primitives::CodeUploadResult<Hash, Balance>
				{
					Contracts::bare_upload_code(origin, code, storage_deposit_limit)
				}

				fn get_storage(
					address: AccountId,
					key: [u8; 32],
				) -> pallet_contracts_primitives::GetStorageResult {
					Contracts::get_storage(address, key)
				}
			}

			#[cfg(feature = "try-runtime")]
			impl frame_try_runtime::TryRuntime<Block> for Runtime {
				fn on_runtime_upgrade() -> (Weight, Weight) {
					// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
					// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
					// right here and right now.
					let weight = Executive::try_runtime_upgrade().unwrap();
					(weight, BlockWeights::get().max_block)
				}

				fn execute_block_no_check(block: Block) -> Weight {
					Executive::execute_block_no_check(block)
				}
			}
		}
	};
}
