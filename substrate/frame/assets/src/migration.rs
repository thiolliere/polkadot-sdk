// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use frame_support::traits::OnRuntimeUpgrade;
use log;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

pub mod v1 {
	use frame_support::{pallet_prelude::*, weights::Weight};

	use super::*;

	#[derive(Decode)]
	pub struct OldAssetDetails<Balance, AccountId, DepositBalance> {
		pub owner: AccountId,
		pub issuer: AccountId,
		pub admin: AccountId,
		pub freezer: AccountId,
		pub supply: Balance,
		pub deposit: DepositBalance,
		pub min_balance: Balance,
		pub is_sufficient: bool,
		pub accounts: u32,
		pub sufficients: u32,
		pub approvals: u32,
		pub is_frozen: bool,
	}

	impl<Balance, AccountId, DepositBalance> OldAssetDetails<Balance, AccountId, DepositBalance> {
		fn migrate_to_v1(self) -> AssetDetails<Balance, AccountId, DepositBalance> {
			let status = if self.is_frozen { AssetStatus::Frozen } else { AssetStatus::Live };

			AssetDetails {
				owner: self.owner,
				issuer: self.issuer,
				admin: self.admin,
				freezer: self.freezer,
				supply: self.supply,
				deposit: self.deposit,
				min_balance: self.min_balance,
				is_sufficient: self.is_sufficient,
				accounts: self.accounts,
				sufficients: self.sufficients,
				approvals: self.approvals,
				status,
			}
		}
	}

	pub struct MigrateToV1<T>(core::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let in_code_version = Pallet::<T>::in_code_storage_version();
			let on_chain_version = Pallet::<T>::on_chain_storage_version();
			if on_chain_version == 0 && in_code_version == 1 {
				let mut translated = 0u64;
				Asset::<T>::translate::<
					OldAssetDetails<T::Balance, T::AccountId, DepositBalanceOf<T>>,
					_,
				>(|_key, old_value| {
					translated.saturating_inc();
					Some(old_value.migrate_to_v1())
				});
				in_code_version.put::<Pallet<T>>();
				log::info!(
					target: LOG_TARGET,
					"Upgraded {} pools, storage to version {:?}",
					translated,
					in_code_version
				);
				T::DbWeight::get().reads_writes(translated + 1, translated + 1)
			} else {
				log::info!(
					target: LOG_TARGET,
					"Migration did not execute. This probably should be removed"
				);
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			frame_support::ensure!(
				Pallet::<T>::on_chain_storage_version() == 0,
				"must upgrade linearly"
			);
			let prev_count = Asset::<T>::iter().count();
			Ok((prev_count as u32).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(prev_count: Vec<u8>) -> Result<(), TryRuntimeError> {
			let prev_count: u32 = Decode::decode(&mut prev_count.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);
			let post_count = Asset::<T>::iter().count() as u32;
			ensure!(
				prev_count == post_count,
				"the asset count before and after the migration should be the same"
			);

			let in_code_version = Pallet::<T>::in_code_storage_version();
			let on_chain_version = Pallet::<T>::on_chain_storage_version();

			frame_support::ensure!(in_code_version == 1, "must_upgrade");
			ensure!(
				in_code_version == on_chain_version,
				"after migration, the in_code_version and on_chain_version should be the same"
			);

			Asset::<T>::iter().try_for_each(|(_id, asset)| -> Result<(), TryRuntimeError> {
				ensure!(
					asset.status == AssetStatus::Live || asset.status == AssetStatus::Frozen,
					 "assets should only be live or frozen. None should be in destroying status, or undefined state"
				);
				Ok(())
			})?;
			Ok(())
		}
	}
}

pub mod v2 {
	use frame_support::{pallet_prelude::*, weights::Weight};

	use super::*;

	pub struct MigrateV1ToV2<T>(core::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateV1ToV2<T> {
		fn on_runtime_upgrade() -> Weight {
			let in_code_version = Pallet::<T>::in_code_storage_version();
			let on_chain_version = Pallet::<T>::on_chain_storage_version();
			if on_chain_version == 0 && in_code_version == 1 {
				in_code_version.put::<Pallet<T>>();
				log::info!(
					target: LOG_TARGET,
					"Upgraded storage to version {:?}",
					in_code_version
				);
				T::DbWeight::get().reads_writes(1, 1)
			} else {
				log::info!(
					target: LOG_TARGET,
					"Migration did not execute. This probably should be removed"
				);
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			frame_support::ensure!(
				Pallet::<T>::on_chain_storage_version() == 1,
				"must upgrade linearly"
			);
			let prev_count = Asset::<T>::iter().count();
			Ok((prev_count as u32).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(prev_count: Vec<u8>) -> Result<(), TryRuntimeError> {
			let prev_count: u32 = Decode::decode(&mut prev_count.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);
			let post_count = Asset::<T>::iter().count() as u32;
			ensure!(
				prev_count == post_count,
				"the asset count before and after the migration should be the same"
			);

			let in_code_version = Pallet::<T>::in_code_storage_version();
			let on_chain_version = Pallet::<T>::on_chain_storage_version();

			frame_support::ensure!(in_code_version == 2, "must_upgrade");
			ensure!(
				in_code_version == on_chain_version,
				"after migration, the in_code_version and on_chain_version should be the same"
			);

			Asset::<T>::iter().try_for_each(|(_id, asset)| -> Result<(), TryRuntimeError> {
				ensure!(
					asset.status == AssetStatus::Live || asset.status == AssetStatus::Frozen || asset.status == AssetStatus::Destroying,
					 "assets should only be live or frozen or destroying. None should be in LiveAndNoPrivileges status, or undefined state"
				);
				Ok(())
			})?;
			Ok(())
		}
	}
}
