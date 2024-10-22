# xNFT pallet
Pallet for cross chain NFT transfer.

version = polkadot-v0.9.43
## Overview
The pallet contains the following features:-
1. Transfers collection from parachain to parachian 
2. Transfers NFTs from parachain to parachain 
#### **Please note:** In the latest version of the XCM protocol (v3), there's a limitation to the number of NFTs that can be transferred in a single transaction. Users are currently restricted to transferring a maximum of three NFTs at a time.
## Installation
#### Prerequisites:
- Rust. [Installation Guide](https://docs.substrate.io/install/). (Recommended version: rustc 1.68.0-nightly (574b64a97 2022-12-31))
- Substrate version polkadot-v0.43 [Installation Guide](https://github.com/paritytech/substrate/tree/polkadot-v0.9.43). 
- pallet_nft and pallet_xnft
- Relay-para environment
- Channel between the para chains
## For Building Binaries:-
#### For Relay-Chain Binary:-

Simply  clone the repo provided in the link https://github.com/paritytech/polkadot/tree/release-v0.9.43 and build it by running the command as
```
cargo build --release
```
#### For the parachain-template-node binary:-

-Simply clone the repo from link https://github.com/substrate-developer-hub/substrate-parachain-template/tree/polkadot-v0.9.40

-You need to add pallets Frame nft and pallet xnft to your parachain node template. For integration of pallets in your chain refer to Step 1 and Step 2 below.

-After adding the pallets to your parachain node template, follow Step 3 provided below

#### For zombinet-linux binary or zombinet-macos binary:-

You can directly download it from the link https://github.com/paritytech/zombienet/releases/

In this way you can build all the three binaries required for the testing enivironment.

After you get all the binaries, Follow step 4 provided below to setup Relay-Para environment.
#### Steps:-
1. Add the pallets Frame nft and pallet-xnft to your parachain-node-template. Refer this [guide](https://docs.substrate.io/tutorials/build-application-logic/add-a-pallet/) for help with the integration. 
2. Add the following snippet to your runtime:This is the runtime for pallet-xnft
```rust
impl pallet_xnft::Config for Runtime{
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmSender = XcmRouter;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}
```
3. Build the parachain-node-template repo using the following command:
```
cargo build --release
```
4. Set your relay-para connection. For help refer to the following links:-
- [Connect to a Relay Chain](https://docs.substrate.io/reference/how-to-guides/parachains/connect-to-a-relay-chain/)
- [Connect to a Local Parachain](https://docs.substrate.io/tutorials/build-a-parachain/connect-a-local-parachain/)
- [Open HRMP Channel between parachains](https://docs.substrate.io/tutorials/build-a-parachain/open-message-passing-channels/)
## Testing Guide
- Run the following command to execute the test cases:
```
cargo test --package pallet-xnft --lib -- test --nocapture 
```
- For ecosystem testing using trappist:
1. checkout to the [trappist](https://github.com/antiers-solutions/xnft/tree/trappist) branch.
2. follow the readme in that branch for help with the setup.
#### Licence: Apache-2.0
## Notes
- We have used [trappist](https://github.com/paritytech/trappist) for help with ecosystem testing. A few changes have been made to the readme and the repo for integrating our pallet effectively.
- For multi NFT transfer, there is a limitation of maximum 3 transaction in XCM queue while using transact function. Due to this the maximum number of nft that can be transferred at one time is 3.
- The sibling account must have enough funds to perform the transactions 
