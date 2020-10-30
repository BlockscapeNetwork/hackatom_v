# Marketplace Smart Contract

This smart contract provides a marketplace for selling and buying CW721 tokens with CW20 tokens.

## Requirements

* [Go `v1.14+`](https://golang.org/)
* [Rust `v1.44.1+`](https://rustup.rs/)
* [Wasmd v0.11.1](https://github.com/CosmWasm/wasmd/tree/v0.11.1)
* [cosmwasm-plus v0.3.2](https://github.com/CosmWasm/cosmwasm-plus)
  * [cw20-base](https://github.com/CosmWasm/cosmwasm-plus/tree/master/contracts/cw20-base)
  * [cw721-base](https://github.com/CosmWasm/cosmwasm-plus/tree/master/contracts/cw721-base)

## Setup Environment

1) Follow [the CosmWasm docs](https://docs.cosmwasm.com/getting-started/installation.html) to install `go v1.14+`, `rust v1.44.1+` and `wasmd v0.11.1`
2) Once you've built `wasmd`, use the `wasmcli` to join the `hackatom-wasm` chain.

```shell
wasmcli config chain-id hackatom-wasm
wasmcli config indent true
wasmcli config keyring-backend test
wasmcli config node https://rpc.cosmwasm.hub.hackatom.dev:443
wasmcli config output json
wasmcli config trust-node true
```

3) You will need two accounts and get some tokens from the faucet. **If you already have accounts with funds, you can skip this step.**

```shell
# Create accounts and save the mnemonics
wasmcli keys add client
wasmcli keys add partner
```

Next, get funds from the [faucet](https://five.hackatom.org/resources). Otherwise, you won't be able to deploy the smart contracts on the blockchain.

### Building the Contracts

We need to build three smart contracts in total:

* `cw20-base` for buying tokens,
* `cw721-base` for selling tokens and withdrawing offerings
* and `marketplace`.

```shell
# Cargo build
# - cw20-base from cosmwasm-plus/contracts/cw20-base
# - cw721-base from from cosmwasm-plus/contracts/cw721-base
cargo wasm

# Docker build from
# - cosmwasm-plus/ directory for cw20-base and cw721-base
# - hackatom_v/ directory for marketplace
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.4
```

### Uploading the Contracts

Now that we've built our contracts, we need to upload them to the blockchain.

> :information_source: In order to avoid confusion, run `wasmcli query wasm list-code` after each individual upload to get the contract ID. You will be needing the IDs in the next step.

```shell
# Upload both cw20-base and cw721-base from cosmwasm-plus/artifacts/ directory
wasmcli tx wasm store cw20-base.wasm --from client --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y
wasmcli tx wasm store cw721-base.wasm --from client --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y

# Upload marketplace from hackatom_v/artifacts/ directory
wasmcli tx wasm store marketplace.wasm --from client --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y
```

### Instantiating the Contracts

Now that we've uploaded our contracts to the blockchain, we need to instantiate them individually using the IDs we got from uploading.

```shell
# cw20-base initialization
wasmcli tx wasm instantiate <CW20-BASE_CONTRACT_ID> '{
  "name": "<INSERT_NAME>",
  "symbol": "<INSERT_SYMBOL>",
  "decimals": "<INSERT_DECIMALS>",
  "initial_balances": [
    {
      "address": "<INSERT_ADDR>",
      "amount": "<INSERT_AMOUNT>"
    }
  ],
  "mint": {
    "minter": "<INSERT_MINTER_ADDR>"
  }
}' --label "cw20-base" --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client

# cw721-base initialization
wasmcli tx wasm instantiate <CW721-BASE_CONTRACT_ID> '{
  "name": "<INSERT_NAME>",
  "symbol": "<INSERT_SYMBOL>",
  "minter": "<INSERT_MINTER_ADDR>"
}' --label "cw721-base" --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client

# marketplace initialization
wasmcli tx wasm instantiate <MARKETPLACE_CONTRACT_ID> '{
  "marketplace_name": "<INSERT_NAME>"
}' --label "marketplace" --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client
```

Once instantiated, you can use `wasmcli query wasm list-contract-by-code <CONTRACT_ID>` to query contract info.

### Executing a Contract Method

```shell
wasmcli tx wasm execute <CONTRACT_ADDR> '{ "method_name": { <json encoded method params> } }' --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client
```

#### Selling an NFT Token

Puts an NFT token up for sale.

```shell
# Mint NFT token
wasmcli tx wasm execute <CW721-BASE_CONTRACT_ADDR> '{ "mint": { "token_id": "<TOKEN_ID>", "owner": "OWNER_ADDR", "name": "TOKEN_NAME", "level": "TOKEN_LEVEL" } }'

# Execute send_nft action to put token up for sale for specified list_price on the marketplace
wasmcli tx wasm execute <CW721-BASE_CONTRACT_ADDR> '{
  "send_nft": {
    "contract": "<MARKETPLACE_CONTRACT_ADDR>",
    "token_id": "<TOKEN_ID>",
    "msg": {
      "receive": {
        "cw721_rcv": {
          "sender": "<INSERT_SENDER_ADDR>",
          "token_id": "<INSERT_TOKEN_ID>",
          "msg": {
            "sell_nft": {
              "list_price": {
                "address": "<INSERT_ADDR>",
                "amount": "<INSERT_AMOUNT>"
              }
            }
          }
        }
      }
    }
  }
}'
```

#### Withdrawing an NFT Token Offering

Withdraws an NFT token offering from the global offerings list and returns the NFT token back to its owner.

> :warning: This will only work after having used `SellNft` on a token.

```shell
# Execute withdraw_nft action to withdraw the token with the specified offering_id from the marketplace
wasmcli tx wasm execute <MARKETPLACE_CONTRACT_ADDR> '{
  "withdraw_nft": {
    "offering_id": "<INSERT_OFFERING_ID>"
  }
}'
```

#### Buying an NFT Token

Buys an NFT token, transferring funds to the seller and the token to the buyer.

> :warning: This will only work after having used `SellNft` on a token.

```shell
# Execute send action to buy token with the specified offering_id from the marketplace
wasmcli tx wasm execute <CW20-BASE_CONTRACT_ADDR> '{
  "send": {
    "contract": "<MARKETPLACE_CONTRACT_ADDR>",
    "amount": "<INSERT_AMOUNT>",
    "msg": {
      "receive": {
        "cw20_rcv": {
          "sender": "<INSERT_SENDER_ADDR>",
          "amount": "<INSERT_AMOUNT>",
          "msg": {
            "buy_nft": {
              "offering_id": "<INSERT_OFFERING_ID>"
            }
          }
        }
      }
    }
  }
}'
```

#### Querying List of Offerings

Queries a list of all currently listed offerings.

```shell
wasmcli query wasm contract_state smart <MARKETPLACE_CONTRACT_ADDR> '{
  "get_offerings": {}
}'
```
