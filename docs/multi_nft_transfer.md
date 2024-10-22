# Multi NFT Transfer
Function calls to transfer multiple NFTs

## Prerequisites
1. Create a collection using the NFT pallet.
2. In the collection mint NFTs that you want to transfer.
3. Open bidirectinal HRMP channel between source and target chains.
4. Sibling account(as per the ParaID) must have enough funds for paying fees and to lock some funds until NFT is transfered.

## Functions (in specific order)
### 1. Collection Transfer
- To transfer multiple NFT, first we need their parent collection on the target chain. 
![collection_transfer](https://github.com/w3f/Grant-Milestone-Delivery/assets/60818312/78111f68-1f38-4e75-bfe5-bd37b3643fbd)
- To do so, we call the collection_transfer function in the xNFT pallet with the following parameters:

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `sibling_account` | `AccountId` | the sibling account id of the parachain |
| `collection_id` | `CollectionId` | the ID of the collection being sent |
| `destCollectionId` | `CollectionId` | the ID of the collection to which the nft is being sent |
| `dest` | `MultiLocation` | a multilocation to define the destination address for the tokens being sent via XCM. It supports different address formats, such as 20 or 32-byte addresses (Ethereum or Substrate) |

### 2. Transfer Multi NFTs
- Secondly, we need to create NFT on the target chain. 
![multi_nft_transfer](https://github.com/w3f/Grant-Milestone-Delivery/assets/60818312/b1206435-f980-4acf-a7f6-dfe28498de5a)
- To do this, call the multi_nft_transfer function in the xNFT Pallet with the following parameters:

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `collection_id` | `CollectionId` | the ID of the collection from which the nft is being sent |
| `item_id` | `Vec<ItemId>` | the ID of the NFT being sent |
| `dest_collection_id` | `CollectionId` | the ID of the collection to which the nft is being sent |
| `dest_item_id` | `Vec<ItemId>` | the ID of the NFT being created on target chain |
| `mint_to_sibling` | `AccountId` | ID of the sibling account of the parachain |
| `dest` | `MultiLocation` | a multilocation to define the destination address for the tokens being sent via XCM. It supports different address formats, such as 20 or 32-byte addresses (Ethereum or Substrate) |

### 3. Transfer NFT Metadata 
**NOTE:-** This is optional and will be used if the transferred NFT has metadata. Otherwise skip to point 4.
- Third, we transfer the metadata of the transferred NFT to the target chain.
![transfer_nft_metadata](https://github.com/w3f/Grant-Milestone-Delivery/assets/60818312/cf24f742-6c88-4da2-be59-87d6e7c46a8c)
- We call the transfer_nft_metadata function in the xNFT pallet with the following parameters:

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `destCollection_id` | `CollectionId` | the ID of the collection to which the nft is being sent |
| `destItemId` | `Vec<ItemId>` | the ID of the NFT being created on target chain |
| `dest` | `Multilocation` | a multilocation to define the destination address for the tokens being sent via XCM. It supports different address formats, such as 20 or 32-byte addresses (Ethereum or Substrate) |

### 4. Transfer NFTs Ownership
- Finally, we have the NFTs on target chain with their metadata set. However these NFT as still in the sibling account's wallet. We will need to transfer these NFT to the new owner's account
![transfer_nft_ownership](https://github.com/w3f/Grant-Milestone-Delivery/assets/60818312/6aa24416-c632-439a-a096-c22ec108ab99)
- Call the transfer_nft_ownership function function in the xNFT pallet with the following parameters:

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `new_owner` | `AccountId` | ID of the target account for transfer of the collection |
| `dest_collection_id` | `CollectionId` | the ID of the collection to which the nft is being sent on target chain |
| `dest_item_id` | `Vec<ItemId>` | the ID of the ITEM to which the nft is being sent on target chain |
| `dest` | `MultiLocation` | a multilocation to define the destination address for the tokens being sent via XCM. It supports different address formats, such as 20 or 32-byte  addresses (Ethereum or Substrate) |

#### Note: While transferring multiple NFTs make sure all the NFT belong to the same collection. Transfer the collection to target chain before transferring NFTs.