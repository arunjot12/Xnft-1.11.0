# USER GUIDE
This guide includes steps for a user to leverage the xNFT pallet to transfer NFT cross-chains

## Prerequisites
1. Establish bidirectional HRMP channel between parchains through relay chain. For help, use [this](https://docs.substrate.io/reference/how-to-guides/parachains/add-hrmp-channels/) guide.
2. Create a collection, and mint some NFTs using the [NFT Pallet](https://github.com/antiers-solutions/xNFT/tree/master/nfts).
3. Sibling account(as per the ParaID) must have enough funds for paying fees and to lock some funds until NFT is transfered.

## Following are the steps that you will need to follow in order to use the xNFT functionality:-
#### **COLLECTION TRANSFER**
Here we are transferring the collection with collection ID = 0 along with its associated metadata. 

1. Transfer the collection to destination chain through xNFT pallet. 
- Go to developers tab then go to extrinsics.
- Select **xNFT** pallet inside this select **collectionTransfer** function. Set the parameters as per your requirement.
![ss8](./assets/12.jpg)

**Events popped at source chain will be as follows:-**
- collectionSent
- collectionMetadataTransfer
- collectionTransferedSuccessfully
![ss9](./assets/13.jpg)

**Events popped at destination chain will be as follows:-**
- created
- success 
- collectionMetadataSet
![ss10](./assets/14.jpg)

#### **NFT Transfer**
Here we are transferring NFT with NFT ID = 0 present at collection with collection_id = 0. It transfers NFT along with its associated metadata and assign a new owner to the NFT. 
2. Transfer the NFT.
- Go to developers tab then go to extrinsics.
- Select **xNFT** pallet inside this select **nftTransfer** function. Set the parameters as per your requirement.
![ss11](./assets/15.jpg)

**Events popped at source chain will be as follows:-**
- NftSent
- itemMetadataTransfered
- nftOwnershipTransfered
- Burned

![ss12](./assets/16.jpg)
**Events popped at destination chain will be as follows:-**
- issued 
- success 
- itemMetadataSet
- transfered
![ss13](./assets/17.jpg)

#### **TRANSFER MULTIPLE NFTs**
Here we are transferring NFTs with NFT ID = 1,2,3 present in collection with collection ID = 0. It transfers just the NFTs at the destination chain.
3. Transfer the multi NFT. 
- Go to developers tab then go to extrinsics.
- Select **xNFT** pallet inside this select **transferMultiNfts** function. Set the parameters as per your requirement.
![ss14](./assets/18.jpg)


**Events popped at source chain will be as follows:-**
- NftSent
- Burned
![ss15](./assets/19.jpg)
**Events popped at source chain will be as follows:-**
- issued 
- success
![ss16](./assets/20.jpg)

#### **TRANSFER NFT METADATA**
Here we are transferring above sent NFTs metadatas. 
4. Transfer metadata of NFTs. 
- Go to developers tab then go to extrinsics.
- Select **xNFT** pallet inside this select **transferNftMetadata** function. Set the parameters as per your requirement.
![ss17](./assets/21.jpg)

**Events popped at source chain are as follows:-**
- itemMetadataTransfered
![ss18](./assets/22.jpg)

**Events triggered at destination chain are as follows:-** 
- itemMetadataSet
- Success
![ss19](./assets/23.jpg)


#### **TRANSFER NFT OWNERSHIP**
Here we're assigning the ownership of NFTs sent in step 10. 
5. Change the owner of the NFTs.
- Go to developers tab then go to extrinsics.
- Select **xNFT** pallet inside this select **tansferNftsOwnership** function. Set the parameters as per your requirement.
![ss20](./assets/24.jpg)

**Events popped at source chain are as follows:-**
- NftOwnershipTransfered
      
![ss21](./assets/25.jpg)

**Events triggered at destination chain are as follows:-** 
- transfered
- Success
![ss22](./assets/26.jpg)


#### **COLLECTION OWNER TRANSFER**
Here we change the owner of collection that is being sent from source chain to destination chain.For this we need to accept the ownership at destination chain for the collection of which we are changing the ownership.
6. Change the owner of the collection.
- Go to destination chain.
- Go to developers tab then go to extrinsics.
- Select **NFT** pallet inside this select **setAcceptOwnership** function. Set the parameters as per your requirement.
![ss23](./assets/27.jpg)

- Then go back to the source chain select developer tab.Next, select extrinsics. 
- Select **xNFT** pallet inside this select **transferCollectionOwnership** function. Set the parameters as per your requirement.
![ss24](./assets/28.jpg)

**Events popped at source chain are as follows:-**
- collectionOwnershipTransfer
      
![ss25](./assets/29.jpg)

**Events triggered at destination chain are as follows:-** 
- ownerChanged
- Success
![ss26](./assets/30.jpg)