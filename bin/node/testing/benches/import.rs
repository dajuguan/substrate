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
use sc_client_db::PruningMode;
use sc_executor::{NativeExecutor, RuntimeInfo, WasmExecutionMethod, Externalities};
use sp_consensus::block_import::{BlockImport, BlockImportParams};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

criterion_group!(benches, bench_block_import);
criterion_main!(benches);

fn genesis() -> node_runtime::GenesisConfig {
	node_testing::genesis::config(false, Some(node_runtime::WASM_BINARY))
}

// This should return client that is doing everything that full node
// is doing.
//
// - This client should not cache anything.
//     (TODO: configure zero rocksdb block cache)
//
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
	unimplemented!();
}

// Import generated block.
fn import_block(client: &mut Client) {
	let block = generate_block_import(client);
	client.import_block(block, Default::default())
		.expect("Failed to import block");
}

fn bench_block_import(c: &mut Criterion) {
	unimplemented!()
}