# Marketplace Smart Contract

This smart contract enables trading of cw721 NFT tokens. To accomplish this the contract implements a *receiveNft* function which acts as a counterpart to the cw721 *sendNft* function. It also provides further functions for trading.

## Messages

| Message                             | Description                                                                           | Status             |
|:------------------------------------|:--------------------------------------------------------------------------------------|:------------------:|
| `receiveNft(sender, token_id, msg)` | Counter-part to `sendNft`, handling the receival of a token in the receiving contract | :white_check_mark: |
| `sellNft(list_price)`               | Sells a token for a specified price                                                   | :white_check_mark: |
| `buyNft(token_id)`                  | Buys a token for the price it has been put up for sale                                | :white_check_mark: |
| `withdrawNft(token_id)`             | Withdraws a token offering                                                            | :white_check_mark: |

## Queries

| Query            | Description                                                                                                               | Status             |
|:-----------------|:--------------------------------------------------------------------------------------------------------------------------|:------------------:|
| `getOfferings()` | Retrieves a list of all current offerings (seller address, token information, contract address the token originated from) | :x:                |

## CLI Workflow

In total, we will be needing three contracts: `cw20-base` for buying tokens, `cw721-base` for selling tokens and withdrawing offerings and `marketplace`.

### Create Account(s)

**If you already have account(s) with funds, you can skip this step.**

```shell
wasmcli keys add client --keyring-backend test
wasmcli keys add partner --keyring-backend test
```

Make sure to get funds from the [faucet](https://five.hackatom.org/resources).

### Building and Uploading a Contract

The following instructions apply to all contracts.

```shell
# If necessary, create schema from hackatom_v/contracts/<CONTRACT>
cargo schema

# Cargo build from hackatom_v/contracts/<CONTRACT>
cargo wasm

# Docker build from hackatom_v/
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.4

# Upload from hackatom_v/artifacts
wasmcli tx wasm store <CONTRACT>.wasm --from client --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y
```

Then, use `wasmcli query wasm list-code` to find out your contract ID.

### Instantiating a Contract

```shell
wasmcli tx wasm instantiate <CONTRACT_ID> '{ json encoded InitMsg params }' --label "<CONTRACT>" --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client
```

* The contract's `InitMsg` params can be found inside the `msg.rs` file in the `pub struct InitMsg {}`
  * Example: For `marketplace`, it's `{ "marketplace_name": "<SOME_NAME_HERE>" }`
* Once instantiated, you can use `wasmcli query wasm list-contract-by-code <ID>` to query contract info

### Executing a Contract Method

```shell
wasmcli tx wasm execute <CONTRACT_ADDR> '{<json encoded method params>}' --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client
```

* The contract's method params can be found inside the `msg.rs` file
  * Example: For `cw721-base`'s method `sendNft`, the params are `{ "contract": "<HumanAddr>", "token_id": "<String>", "msg": "<Option<Binary>>" }`

#### Selling an NFT Token

> :information_source: Needs `cw721-base` and `marketplace` deployed on the blockchain.

Puts an NFT token up for sale.

```shell
# Mint NFT token
wasmcli tx wasm execute <CW721_CONTRACT_ADDR> '{ "mint": { "token_id": "<TOKEN_ID>", "owner": "OWNER_ADDR", "name": "TOKEN_NAME", "level": "TOKEN_LEVEL" } }'

# Execute send_nft action to send token to marketplace
wasmcli tx wasm execute <CW721_CONTRACT_ADDR> '{ "send_nft": { "contract": "<MARKETPLACE_CONTRACT_ADDR>", "token_id": "<TOKEN_ID>", "msg": <BINARY_ENCODED_SELL_NFT_MSG> } }' # TODO: Unclear how msg binary is encoded
```

#### Withdrawing an NFT Token Offering

> :information_source: Needs `cw721-base` and `marketplace` deployed on the blockchain.

Withdraws an NFT token offering from the global offerings list.

:warning: Only works after having used `SellNft` on a token.

```shell
# Execute send_nft action to send token to marketplace
wasmcli tx wasm execute <MARKETPLACE_CONTRACT_ADDR> '{ "withdraw_nft": { "token_id": "<TOKEN_ID>" } }'
```

#### Buying an NFT Token

> :information_source: Needs `cw20-base`, `cw721-base` and `marketplace` deployed on the blockchain.

Buys an NFT token, transferring funds to the seller and the token to the buyer.

:warning: Only works after having used `SellNft` on a token.

```shell
# Execute send_nft action to send token to marketplace
wasmcli tx wasm execute <MARKETPLACE_CONTRACT_ADDR> '{ "buy_nft": { "token_id": "<TOKEN_ID>" } }'
```

#### Querying List of Offerings

TODO
