# Single NFT Transfer
Function calls to transfer a single NFT

## Prerequisites
1. Create a collection using the NFT pallet.
2. In the collection mint an NFT that you want to transfer.
3. Open bidirectinal HRMP channel between source and target chains.
4. Sibling account(as per the ParaID) must have enough funds for paying fees and to lock some funds until NFT is transfered.

## Functions (in specific order)
### 1. Collection Transfer 
- To transfer an NFT, first we need it's parent collection on the target chain.
![collection_transfer](https://github.com/w3f/Grant-Milestone-Delivery/assets/60818312/78111f68-1f38-4e75-bfe5-bd37b3643fbd)
- Hence we call the collection_transfer function in the xNFT pallet with the following parameters:

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `sibling_account` | `AccountId` | the sibling account id of the parachain |
| `collection_id` | `CollectionId` | the ID of the collection being sent |
| `destCollectionId` | `CollectionId` | the ID of the collection to which the nft is being sent |
| `dest` | `MultiLocation` | a multilocation to define the destination address for the tokens being sent via XCM. It supports different address formats, such as 20 or 32-byte addresses (Ethereum or Substrate) |

### 2. NFT Transfer
- We simply transfer the NFT to the target chain
![nft_transfer](https://github.com/w3f/Grant-Milestone-Delivery/assets/60818312/6694e314-106f-45ef-9359-b898969c84b4)
- Call the nft_transfer function in the xNFT pallet with the following parameters:

| Parameter | Type     | Description                       |
| :-------- | :------- | :-------------------------------- |
| `collection_id`      | `CollectionId` | the ID of the collection of which the nft is being sent |
| `item_id`      | `Vec<ItemId>` | the ID of the NFT being sent |
| `dest_collection_id`      | `CollectionId` | the ID of the collection to which the nft is being sent |
| `dest_item_id`      | `Vec<ItemId>` | the ID of the NFT being created |
| `mint_to_sibling`      | `AccountId` | the sibling account of the parachain |
| `new_owner`      | `AccountId` | the new owner of the NFT's being sent |
| `dest`      | `MultiLocation` | a multilocation to define the destination address for the tokens being sent via XCM. It supports different address formats, such as 20 or 32-byte addresses (Ethereum or Substrate) |
