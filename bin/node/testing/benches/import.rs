// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Block import benchmark.
//!
//! This benchmark is expected to measure block import operation of
//! some full block.
//!
//! As we also want to protect against cold-cache attacks, this
//! benchmark should not rely on any caching - database or otherwise
//! (except those that DO NOT depend on user input).
//!
//! This is why we populate block with transactions to random accounts
//! and set state_cache_size to 0.
//!
//! This is supposed to be very simple benchmark and is not subject
//! to much configuring - just block full of randomized transactions.
//! It is not supposed to measure runtime modules weight correctness
//! (there is a dedicated benchmarking mode for this).

use node_primitives::Block;
use node_testing::client::{Client, Backend, Transaction};
use node_testing::keyring::*;
use sc_client_db::PruningMode;
use sc_executor::{NativeExecutor, RuntimeInfo, WasmExecutionMethod, Externalities};
use sp_consensus::{SelectChain, BlockOrigin, BlockImport, BlockImportParams, ForkChoiceStrategy};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use sp_runtime::{
	generic::BlockId,
	OpaqueExtrinsic,
	traits::{Block as BlockT, Zero},
};
use codec::{Decode, Encode};
use node_runtime::{
	Call,
	CheckedExtrinsic,
	constants::currency::DOLLARS,
	UncheckedExtrinsic,
	MinimumPeriod,
	BalancesCall,
};
use sp_core::ExecutionContext;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_inherents::InherentData;

criterion_group!(benches, bench_block_import);
criterion_main!(benches);

fn genesis() -> node_runtime::GenesisConfig {
	node_testing::genesis::config(false, Some(node_runtime::WASM_BINARY))
}

fn sign(xt: CheckedExtrinsic, genesis_hash: [u8; 32], version: u32) -> UncheckedExtrinsic {
	node_testing::keyring::sign(xt, version, genesis_hash)
}

// This should return client that is doing everything that full node
// is doing.
//
// - This client should not cache anything.
//     (TODO: configure zero rocksdb block cache)
// - This client should use best wasm execution method.
// - This client should work with real database only.
fn bench_client() -> (Client, std::sync::Arc<Backend>) {
	let path = std::path::PathBuf::from("/tmp/sub-bench");

	let db_config = sc_client_db::DatabaseSettings {
		state_cache_size: 0,
		state_cache_child_ratio: Some((0, 100)),
		pruning: PruningMode::ArchiveAll,
		source: sc_client_db::DatabaseSettingsSrc::Path {
			path: path.clone(),
			cache_size: None,
		},
	};

	sc_client_db::new_client(
		db_config,
		NativeExecutor::new(WasmExecutionMethod::Compiled, None),
		&genesis(),
		None,
		None,
		Default::default(),
	).expect("Should not fail")
}

// Full (almost) block generation. This is expected to be roughly
// equal to the block which is hitting weight limit.
fn generate_block_import(client: &Client) -> BlockImportParams<Block, Transaction> {
	let version = client.runtime_version_at(&BlockId::number(0))
		.expect("There should be runtime version at 0")
		.spec_version;
	let genesis_hash = client.block_hash(Zero::zero())
		.expect("Database error?")
		.expect("Genesis block always exists; qed")
		.into();

	let mut block = client
		.new_block(Default::default())
		.expect("Block creation failed");

	let timestamp = 1 * MinimumPeriod::get();

	let mut inherent_data = InherentData::new();
	inherent_data.put_data(sp_timestamp::INHERENT_IDENTIFIER, &timestamp)
		.expect("Put timestamb failed");
	inherent_data.put_data(sp_finality_tracker::INHERENT_IDENTIFIER, &0)
		.expect("Put finality tracker failed");

	for extrinsic in client.runtime_api()
		.inherent_extrinsics_with_context(
			&BlockId::number(0),
			ExecutionContext::BlockConstruction,
			inherent_data,
		).expect("Get inherents failed")
	{
		block.push(extrinsic).expect("Push inherent failed");
	}

	let nonce = 0u32;

	let signed = sign(
		CheckedExtrinsic {
			signed: Some((alice(), signed_extra(nonce, 1*DOLLARS))),
			function: Call::Balances(
				BalancesCall::transfer(
					pallet_indices::address::Address::Id(bob()),
					1*DOLLARS
				)
			),
		}, genesis_hash, version);
	let encoded = Encode::encode(&signed);

	let opaque = OpaqueExtrinsic::decode(&mut &encoded[..])
		.expect("Failed  to decode opaque");

	block.push(opaque).expect("Push transaction failed");

	let block = block.build().expect("Block build failed").block;

	BlockImportParams {
		origin: BlockOrigin::File,
		header: block.header().clone(),
		post_digests: Default::default(),
		body: Some(block.extrinsics().to_vec()),
		storage_changes: Default::default(),
		finalized: false,
		justification: Default::default(),
		auxiliary: Default::default(),
		intermediates: Default::default(),
		fork_choice: Some(ForkChoiceStrategy::LongestChain),
		allow_missing_state: false,
		import_existing: false,
	}
}

// Import generated block.
fn import_block(client: &mut Client) {
	let block = generate_block_import(client);
	client.import_block(block, Default::default())
		.expect("Failed to import block");
}

fn bench_block_import(c: &mut Criterion) {
	let (mut client, backend) = bench_client();
	import_block(&mut client);
}
