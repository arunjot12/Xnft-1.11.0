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

//! # xNft Module
//!
//! A simple, secure module for dealing with  cross chain transfer of non-fungible items
//!
//! ## Related Modules
//!
//! * [`System`](../frame_system/index.html)
//! * [`Support`](../frame_support/index.html)
//! * [`Nfts`](../pallets/nfts)
//! * [`XCM`](../pallets/xcm)
   
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod test;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use cumulus_pallet_xcm::Origin as CumulusOrigin;
pub use pallet::*;
pub use sp_runtime::traits::StaticLookup;
pub use frame_support::traits::Currency;
pub use pallet_nfts::{ Call::create, ItemDetails};
use scale_info::prelude::{vec, vec::Vec};
use frame_system::{
	ensure_signed,
	pallet_prelude::{BlockNumberFor, OriginFor},
};
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub use xcm::prelude::*;
pub type BalanceOf<T, I = ()> =
	<<T as pallet_nfts::Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CollectionConfigFor<T, I = ()> = pallet_nfts::CollectionConfig<
	BalanceOf<T, I>, BlockNumberFor<T>,
	<T as pallet_nfts::Config<I>>::CollectionId,
>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::tokens::nonfungibles_v2::Inspect};
	use frame_system:: Config as SystemConfig;
	use sp_runtime::DispatchResult;
	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	/// Map destination collection_id to src collection_id
	#[pallet::storage]
	pub type CollectionMap<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::CollectionId, T::CollectionId, OptionQuery>;

	/// Map destination collection_id and destination item_id to src item_id
	#[pallet::storage]
	pub type ItemIdMap<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(NMapKey<Blake2_128Concat, T::CollectionId>, NMapKey<Blake2_128Concat, T::ItemId>),
		T::ItemId,
		OptionQuery,
	>;

	/// Map destination collection_id , destination item_id , src collection_id and src item_id to NFT metadata
	#[pallet::storage]
	pub type ItemMetadataMap<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Blake2_128Concat, T::ItemId>,
			NMapKey<Blake2_128Concat, T::ItemId>,
		),
		BoundedVec<u8, <T as pallet_nfts::Config<I>>::StringLimit>,
		OptionQuery,
	>;

	/// Map destnation collection_id and item_id to source item owner accountId
	#[pallet::storage]
	pub type AccountIds<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::CollectionId>,
			NMapKey<Blake2_128Concat, T::ItemId>,
		),
		T::AccountId,
		OptionQuery,
	>;

	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_nfts::Config<I> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Required origin for executing XCM messages, including the teleport functionality. If successful,
		/// then it resolves to `Location` which exists as an interior location within this chain's XCM
		/// context.
		type ExecuteXcmOrigin: EnsureOrigin<
			<Self as SystemConfig>::RuntimeOrigin,
			Success = Location,
		>;
		/// Something to execute an XCM message.
		type XcmExecutor: ExecuteXcm<<Self as pallet::Config<I>>::RuntimeCall>;
		/// The runtime `Origin` type.
		type RuntimeOrigin: From<<Self as SystemConfig>::RuntimeOrigin>
			+ Into<Result<CumulusOrigin, <Self as Config<I>>::RuntimeOrigin>>;
		/// How to send an onward XCM message.
		type XcmSender: SendXcm;
		/// The runtime `Call` type.
		type RuntimeCall: From<pallet_nfts::Call<Self, I>> + Encode;
		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type Helper: BenchmarkHelper<Self::CollectionId, Self::ItemId>;
		
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Cross chain collection transfered
		/// to destination chain
		CollectionSent(AccountIdLookupOf<T>,CollectionConfigFor<T, I>),
		/// Collection not successfully transferred 
		/// to the destination chain
		CollectionSendingError(SendError),
		/// Cross chain NFT transfered
		/// to destination chain
		NftSent(T::CollectionId,T::ItemId,AccountIdLookupOf<T>),
		/// NFT not successfully transferred 
		/// to the destination chain
		NftSendingError(SendError),
		/// Cross chain NFT Ownership transfered
		/// to destination chain
		NftOwnershipTransferred(T::CollectionId,T::ItemId,AccountIdLookupOf<T>),
		/// Ownership of the NFT not successfully transferred 
		/// to the destination chain.
		NftOwnershipTransferringError(SendError),
		/// Cross chain Collection Ownership transfered
		/// to destination chain
		CollectionOwnershipTransferred(T::CollectionId,AccountIdLookupOf<T>),
		/// Collection ownership not successfully transfered 
		/// to destination chain
    	CollectionOwnershipTransferringError(SendError),
		/// Cross chain NFT metadata transfered
		/// to destination chain
		ItemMetadataTransferred(T::CollectionId,T::ItemId,BoundedVec<u8, <T as pallet_nfts::Config<I>>::StringLimit>),
		/// NFT metadata not successfully transfered 
		/// to destination chain
		ItemMetadataError(),
		/// Cross chain Collection metadata transfered
		CollectionMetadataTransferred(T::CollectionId, BoundedVec<u8, <T as pallet_nfts::Config<I>>::StringLimit>),
		/// Collection metadata not transfered 
		/// to destination chain
		CollectionMetadataError(SendError),
		/// Cross chain Collection transfered
		/// to destination chain
		CollectionTransferredSuccessfully,

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		// Not the legitimate owner of collection
		NoTCollectionOwner,
		/// Not the legitimate owner of NFT
		NoTNftOwner,
		/// Collection_Id doesn't exist on source chain
		NoSuchCollectionId,
		/// NFT_id doesn't exist on source chain
		NoSuchItemId,
		/// Maximum NFT transferring limit exceeded
		MaxItemCountExceeded,
        /// Unsuccessful execution of NFT transfer 
		NftNotTransferred,
		/// Maximum NFT receiving limit exceeded
		MaxDestItemCountExceeded,
		/// Metadata doesn't exist for collection or NFT
		NotHavingMetadata,
		/// Not the legitimate owner of collection or NFT
		NotTheOwner,
		/// Cross chain collection ownership not transfered
		CollectionOwnershipTransferringError,
		/// Cross chain Collection not transfered
		CollectionTransferringError,
		/// Cross chain Collection metadata not transfered
		CollectionMetadataError,
		/// Cross chain NFT not transfered
		NFTTransferringError,
		/// Cross chain NFT metadata not transfered
		MetadataError,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Transfer a Collection along with its associated metadata
		/// from the src chain to dest chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Collection`;
		///
		/// Arguments:
		/// - `sibling_account`: The sibling account of sender parachain.
		/// - `collection_id`: The collection_id of the collection to be transferred.
		/// - `dest_collection_id`: The collection_id of new collection at destination chain.
		/// - `dest`: The destination chain to which collection is transferred.
		///
		/// Emits `CollectionTransferredSuccessfully`.
		///
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn collection_transfer(
			origin: OriginFor<T>,
			sibling_account: AccountIdLookupOf<T>,
			collection_id: T::CollectionId,
			dest_collection_id: T::CollectionId,
			dest: Location,
			config: CollectionConfigFor<T,I>
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&collection_id),
				Error::<T, I>::NoSuchCollectionId
			);
			//let owner = pallet_nfts::Collection::<T, I>::get(collection_id).map(|a| a.owner);
			let owner = pallet_nfts::Pallet::<T,I>::collection_owner(collection_id);
			ensure!(owner == Some(from), Error::<T, I>::NoTCollectionOwner);
			CollectionMap::<T, I>::insert(dest_collection_id, collection_id);
			let result = Self::do_transfer_collection(dest.clone(), sibling_account, config)?;

			if result == true {
				if pallet_nfts::CollectionMetadataOf::<T, I>::contains_key(&collection_id) {
					let bounded =
				     pallet_nfts::Pallet::<T,I>::collection_attribute(&collection_id, &[])
						.ok_or(Error::<T, I>::NotHavingMetadata)?;
                    let collection_metadata = BoundedVec::truncate_from(bounded);

					let collection_result =
						Self::do_set_collection_metadata(dest.clone(), collection_id, collection_metadata)?;

					if collection_result == true {
						Self::deposit_event(Event::CollectionTransferredSuccessfully);
					} else {
						return Err(Error::<T, I>::CollectionMetadataError.into())
					}
				} else {
					return Ok(())
				}
			} else {
				// Return an error if do_transfer_collection failed
				return Err(Error::<T, I>::CollectionTransferringError.into())
			}

			Ok(())
		}

		/// Transfer an item, along with its associated metadata,
		/// and assign a new owner to the item being transferred 
		/// from src chain to dest chain.
		/// 
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Item`;
		///
		/// Arguments:
		/// - `collection_id`: The collection_id of the collection whose item is to be transferred.
		/// - `item_id`: The item_id of the item to be transferred.
		/// - `dest_collection_id`: The collection_id of the collection to which item is to be transferred.
		/// - `dest_item_id`: The item_id of new item at destination chain.
		/// - `mint_to_sibling`: The sibling account of sender parachain.
		/// - `new_owner`: The new owner of item being sent at destination chain.
		/// - `dest`: The destination chain to which item is transferred.
		///

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::zero())]
		pub fn nft_transfer(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			dest_collection_id: T::CollectionId,
			dest_item_id: T::ItemId,
			mint_to_sibling: AccountIdLookupOf<T>,
			new_owner: AccountIdLookupOf<T>,
			dest: Location,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&collection_id),
				Error::<T, I>::NoSuchCollectionId
			);

			ensure!(
				pallet_nfts::Item::<T, I>::contains_key(&collection_id, &item_id),
				Error::<T, I>::NoSuchItemId
			);
			
			let owner = pallet_nfts::Pallet::<T,I>::owner(collection_id, item_id);
			ensure!(owner == Some(from), Error::<T, I>::NoTNftOwner);

			let nft_result =
				Self::do_nft_transfer(dest.clone(), dest_collection_id, dest_item_id, mint_to_sibling)?;

			if !nft_result {
				return Err(Error::<T, I>::NFTTransferringError.into())
			}

			if pallet_nfts::ItemMetadataOf::<T, I>::contains_key(&collection_id, &item_id) {
				let bounded =
				pallet_nfts::Pallet::<T,I>::attribute(&collection_id, &item_id, &[])
						.ok_or(Error::<T, I>::NotHavingMetadata)?;
				let item_collection_metadata = BoundedVec::truncate_from(bounded);
				let metadata_result = Self::do_set_nft_metadata(
					dest.clone(),
					dest_collection_id,
					dest_item_id,
					item_collection_metadata,
				)?;

				if !metadata_result {
					return Err(Error::<T, I>::MetadataError.into())
				}
			}

			let transfer = Self::do_transfer_nft_ownership(
				dest.clone(),
				dest_collection_id,
				dest_item_id,
				new_owner.clone(),
			);

			if let Ok(true) = transfer {
				let _burn =
					pallet_nfts::Pallet::<T, I>::burn(origin.clone(), collection_id, item_id);
			} else {
				return Err(Error::<T, I>::NftNotTransferred.into())
			}

			Ok(())
		}

		/// Change the owner of collection that is being sent
		/// from src chain to dest chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Collection`;
		///
		/// Arguments:
		/// - `collection_id`: The collection_id of collection at destination chain whose ownership is to be transferred.
		/// - `new_owner`: The new owner of collection being sent at destination chain.
		/// - `dest`: The destination chain to which collection is transferred.
		///

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::zero())]
		pub fn transfer_collection_ownership(
			origin: OriginFor<T>,
			new_owner: AccountIdLookupOf<T>,
			collection_id: T::CollectionId,
			dest: Location,
		) -> DispatchResult {
			let collection = CollectionMap::<T, I>::get(collection_id)
				.ok_or(Error::<T, I>::NoSuchCollectionId)?;
			let owner = pallet_nfts::Pallet::<T,I>::collection_owner(collection).ok_or(Error::<T, I>::NotTheOwner)?;
			let orig = ensure_signed(origin.clone())?;
			ensure!(owner == orig, Error::<T, I>::NoTCollectionOwner);
			let result = Self::do_transfer_collection_ownership(dest, collection_id, new_owner);

			if let Ok(true) = result {
				return Ok(())
			} else {
				return Err(Error::<T, I>::CollectionOwnershipTransferringError.into())
			}
		}

		/// Transfer multiple items
		/// from src chain to dest chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Items`;
		///
		/// Arguments:
		/// - `collection_id`: The collection_id of the collection whose items are transferred.
		/// - `item_id`: The item_id of the items to be transferred.
		/// - `dest_collection_id`: The collection_id of the collection to which items are transferred.
		/// - `dest_item_id`: The item_id of new items at destination chain.
		/// - `mint_to_sibling`: The sibling account of sender parachain.
		/// - `dest`: The destination chain to which items are transferred.
		///

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::zero())]
		pub fn transfer_multi_nfts(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			item_id: Vec<T::ItemId>,
			dest_collection_id: T::CollectionId,
			dest_item_id: Vec<T::ItemId>,
			mint_to_sibling: AccountIdLookupOf<T>,
			dest: Location,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			ensure!(
				pallet_nfts::Collection::<T, I>::contains_key(&collection_id),
				Error::<T, I>::NoSuchCollectionId
			);
			ensure!(item_id.len() <= 3, Error::<T, I>::MaxItemCountExceeded);
			ensure!(dest_item_id.len() <= 3, Error::<T, I>::MaxDestItemCountExceeded);

			let _mint = for i in &item_id {
				ensure!(
					pallet_nfts::Item::<T, I>::contains_key(&collection_id, &i),
					Error::<T, I>::NoSuchItemId
				);
				let owner = pallet_nfts::Pallet::<T,I>::owner(collection_id, *i);
				ensure!(owner == Some(from.clone()), Error::<T, I>::NoTNftOwner);
			};

			for dest_i in &dest_item_id {
				let nft_transfer = Self::do_nft_transfer(
					dest.clone(),
					dest_collection_id,
					*dest_i,
					mint_to_sibling.clone(),
				)?;

				if !nft_transfer {
					return Err(Error::<T, I>::NFTTransferringError.into())
				}
			}

			for (i, dest_i) in item_id.iter().zip(dest_item_id.iter()) {
				AccountIds::<T, I>::insert((dest_collection_id,dest_i), from.clone());
				ItemIdMap::<T, I>::insert((dest_collection_id, dest_i), i);
				if pallet_nfts::ItemMetadataOf::<T, I>::contains_key(&collection_id, &i) {
					let bounded =
				     pallet_nfts::Pallet::<T,I>::attribute(&collection_id, &i, &[])
						.ok_or(Error::<T, I>::NotHavingMetadata)?;
                    let item_collection_metadata = BoundedVec::truncate_from(bounded);
					ItemMetadataMap::<T, I>::insert(
						(collection_id, dest_collection_id, i, dest_i),
						item_collection_metadata.clone(),
					);
				} else {
					continue
				}
			}

			for i in item_id.iter() {
				let _ = pallet_nfts::Pallet::<T, I>::burn(origin.clone(), collection_id, *i);
			}
			Ok(())
		}

		/// Change the owner of Item that is being sent
		/// from src chain to dest chain.
		///
		/// Origin must be Signed and the signing account must be :
		/// - the Owner of the `Item`;
		///
		/// Arguments:
		/// - `dest_collection_id`: The collection_id of collection at destination chain whose item's ownership is to be transferred.
		/// - `dest_item_id`: The item_id of item at destination chain whose ownership is to be transferred.
		/// - `new_owner`: The new owner of item being sent at destination chain.
		/// - `dest`: The destination chain to which item is transferred.
		///

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::zero())]
		pub fn transfer_nfts_ownership(
			origin: OriginFor<T>,
			new_owner: AccountIdLookupOf<T>,
			dest_collection_id: T::CollectionId,
			dest_item_id: Vec<T::ItemId>,
			dest: Location,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(dest_item_id.len() <= 3, Error::<T, I>::MaxItemCountExceeded);

			CollectionMap::<T, I>::get(dest_collection_id).ok_or(Error::<T, I>::NoSuchCollectionId)?;

			for dest_item_id in dest_item_id {
				let owner = AccountIds::<T, I>::get((dest_collection_id,dest_item_id));
				ensure!(owner == Some(who.clone()), Error::<T, I>::NotTheOwner);
				 ItemIdMap::<T, I>::get(&(dest_collection_id, dest_item_id))
					.ok_or(Error::<T, I>::NoSuchItemId)?;

				let ownership_result = Self::do_transfer_nft_ownership(
					dest.clone(),
					dest_collection_id,
					dest_item_id,
					new_owner.clone(),
				);

				if let Ok(true) = ownership_result {
					ItemIdMap::<T, I>::remove((dest_collection_id, dest_item_id));
					AccountIds::<T, I>::remove((dest_collection_id,dest_item_id));
				} else {
					return Err(Error::<T, I>::CollectionOwnershipTransferringError.into())
				}
			}

			Ok(())
		}
		/// Transfers the associated metadata of item that is being sent
		/// from src chain to dest chain.
		///
		/// Origin must be Signed
		///
		/// Arguments:
		/// - `dest_collection_id`: The collection_id of collection at destination chain whose item's metadata is to be set.
		/// - `dest_item_id`: The item_id of item at destination chain whose metadata is to be set.
		/// - `dest`: The destination chain at which item metadata is set.
		///

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::zero())]
		pub fn transfer_nft_metadata(
			origin: OriginFor<T>,
			dest_collection_id: T::CollectionId,
			dest_item_id: Vec<T::ItemId>,
			dest: Location,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			
			ensure!(dest_item_id.len() <= 3, Error::<T, I>::MaxDestItemCountExceeded);

			let collection = CollectionMap::<T, I>::get(dest_collection_id)
				.ok_or(Error::<T, I>::NoSuchCollectionId)?;

			for dest_item_id in dest_item_id {
				let owner = AccountIds::<T, I>::get((dest_collection_id,dest_item_id));
				ensure!(owner == Some(who.clone()), Error::<T, I>::NotTheOwner);
				let source_item_id = ItemIdMap::<T, I>::get(&(dest_collection_id, dest_item_id))
					.ok_or(Error::<T, I>::NoSuchItemId)?;

				let data = ItemMetadataMap::<T, I>::get((
					collection,
					dest_collection_id,
					source_item_id,
					dest_item_id,
				))
				.ok_or(Error::<T, I>::NotHavingMetadata)?;

				let result =
					Self::do_set_nft_metadata(dest.clone(), dest_collection_id, dest_item_id, data.clone());

				if result.unwrap_or(false) {
					Self::deposit_event(Event::ItemMetadataTransferred(dest_collection_id,dest_item_id,data));
					ItemMetadataMap::<T, I>::remove((
						collection,
						dest_collection_id,
						source_item_id,
						dest_item_id,
					));
				} else {
					Self::deposit_event(Event::ItemMetadataError());
				}
			}

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn do_set_nft_metadata(
			dest: Location,
			dest_collection_id: T::CollectionId,
			dest_item_id: T::ItemId,
			data: BoundedVec<u8, <T as pallet_nfts::Config<I>>::StringLimit>,
		) -> Result<bool, DispatchError> {
			match send_xcm::<T::XcmSender>(
				dest.clone(),
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_001_000, 66_536),
					call: <T as Config<I>>::RuntimeCall::from(pallet_nfts::Call::<
						T,
						I,
					>::set_metadata {
						collection: dest_collection_id,
						item: dest_item_id,
						data:data.clone(),
					})
					.encode()
					.into(),
				}]),
			) {
				Ok(_) => {
					Self::deposit_event(Event::ItemMetadataTransferred(dest_collection_id,dest_item_id,data.clone()));
					Ok(true)
				},

				Err(_) => {
					Self::deposit_event(Event::ItemMetadataError());
					Ok(false)
				},
			}
		}

		pub fn do_transfer_collection(
			dest: Location,
			sibling_account: AccountIdLookupOf<T>,
			config: CollectionConfigFor<T, I>,
		) -> Result<bool, DispatchError> {
			match send_xcm::<T::XcmSender>(
				dest,
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_001_000, 66_536),
					call: <T as Config<I>>::RuntimeCall::from(pallet_nfts::Call::<
						T,
						I,
					>::create {
						admin: sibling_account.clone(),
						config,
					})
					.encode()
					.into(),
				}]),
			) {
				Ok(_) => {
					Self::deposit_event(Event::CollectionSent(sibling_account.clone(), config));
					Ok(true)
				},

				Err(e) => {
					Self::deposit_event(Event::CollectionSendingError(e));
					Ok(false)
				},
			}
		}

		pub fn do_set_collection_metadata(
			dest: Location,
			collection_id: T::CollectionId,
			data: BoundedVec<u8, <T as pallet_nfts::Config<I>>::StringLimit>,
		) -> Result<bool, DispatchError> {
			match send_xcm::<T::XcmSender>(
				dest,
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_001_000, 66_536),
					call: <T as Config<I>>::RuntimeCall::from(
						pallet_nfts::Call::<T, I>::set_collection_metadata {
							collection: collection_id,
							data:data.clone(),
						},
					)
					.encode()
					.into(),
				}]),
			) {
				Ok(_) => {
					Self::deposit_event(Event::CollectionMetadataTransferred(collection_id, data.clone()));
					Ok(true)
				},

				Err(e) => {
					Self::deposit_event(Event::CollectionMetadataError(e));
					Ok(false)
				},
			}
		}

		pub fn do_nft_transfer(
			dest: Location,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			mint_to: AccountIdLookupOf<T>,
		) -> Result<bool, DispatchError> {
			match send_xcm::<T::XcmSender>(
				dest,
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_001_000, 66_536),
					call: <T as Config<I>>::RuntimeCall::from(pallet_nfts::Call::<
						T,
						I,
					>::mint {
						collection: collection_id,
						item: item_id,
						mint_to:mint_to.clone(),
						witness_data: None,
					})
					.encode()
					.into(),
				}]),
			) {
				Ok(_) => {
					Self::deposit_event(Event::NftSent(collection_id, item_id, mint_to.clone()));
					Ok(true)
				},

				Err(e) => {
					Self::deposit_event(Event::NftSendingError(e));
					Ok(false)
				}
			}
		}

		pub fn do_transfer_nft_ownership(
			dest: Location,
			collection_id: T::CollectionId,
			item_id: T::ItemId,
			new_owner: AccountIdLookupOf<T>,
		) -> Result<bool, DispatchError> {
			match send_xcm::<T::XcmSender>(
				dest,
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_001_000, 66_536),
					call: <T as Config<I>>::RuntimeCall::from(pallet_nfts::Call::<
						T,
						I,
					>::transfer {
						collection: collection_id,
						item: item_id,
						dest: new_owner.clone(),
					})
					.encode()
					.into(),
				}]),
			) {
				Ok(_) => {
					Self::deposit_event(Event::NftOwnershipTransferred(collection_id, item_id,new_owner.clone()));
					Ok(true)
				},

				Err(e) => {
					Self::deposit_event(Event::NftOwnershipTransferringError(e));
					Ok(false)
				},
			}
		}

		pub fn do_transfer_collection_ownership(
			dest: Location,
			collection_id: T::CollectionId,
			new_owner: AccountIdLookupOf<T>,
		) -> Result<bool, DispatchError> {
			match send_xcm::<T::XcmSender>(
				dest,
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_001_000, 66_536),
					call: <T as Config<I>>::RuntimeCall::from(pallet_nfts::Call::<
						T,
						I,
					>::transfer_ownership {
						collection: collection_id,
						new_owner: new_owner.clone(),
					})
					.encode()
					.into(),
				}]),
			) {
				Ok(_) => {
					Self::deposit_event(Event::CollectionOwnershipTransferred(collection_id, new_owner.clone()));
					Ok(true)
				},
				Err(e) => {
					Self::deposit_event(Event::CollectionOwnershipTransferringError(e));
					Ok(false)
				},
			}
		}
	}
}