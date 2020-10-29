# Marketplace Smart Contract

This smart contract enables trading of cw721 NFT tokens. To accomplish this the contract implements a *receiveNft* function which acts as a counterpart to the cw721 *sendNft* function. It also provides further functions for trading.

## Messages

| Message                             | Description                                                                           | Status             |
|:------------------------------------|:--------------------------------------------------------------------------------------|:------------------:|
| `receiveNft(sender, token_id, msg)` | Counter-part to `sendNft`, handling the receival of a token in the receiving contract | :white_check_mark: |
| `sellNft(list_price)`               | Sells a token for a specified price                                                   | :white_check_mark: |
| `buyNft(token_id)`                  | Buys a token for the price it has been put up for sale                                | :x:                |
| `withdrawNft(token_id)`             | Withdraws a token offering                                                            | :white_check_mark: |

## Queries

| Query            | Description                                                                                                               | Status             |
|:-----------------|:--------------------------------------------------------------------------------------------------------------------------|:------------------:|
| `getOfferings()` | Retrieves a list of all current offerings (seller address, token information, contract address the token originated from) | :x:                |

## CLI Workflow

### Create Account(s)

**If you already have account(s) with funds, you can skip this step.**

```shell
wasmcli keys add client --keyring-backend test
wasmcli keys add partner --keyring-backend test
```

Make sure to get funds from the [faucet](https://five.hackatom.org/resources).

### Build and Upload the Contract

Before you build, make sure the schemas in the `schema/` directory are there. To generate them, use `cargo schema`.

```shell
// Cargo build from hackatom_v/contracts/marketplace
cargo wasm

// Docker build from hackatom_v/
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.4

// Upload from hackatom_v/artifacts
wasmcli tx wasm store marketplace.wasm --from client --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y
```

Use `wasmcli query wasm list-code` to find out your contract ID (`id`) and address (`creator`).

Example output:

```json
{
    "id": 64,
    "creator": "cosmos1nprc69rwkusxj46zcayvwsy9zckkasu3fpf5kf",
    "data_hash": "BDEE625C5C826819008CC80F5E83276F234370C6232A4471F8C92CDF39D44FE6",
    "source": "",
    "builder": ""
}
```

### Instantiate the Contract

```shell
wasmcli tx wasm instantiate 64 '{ "marketplace_name": "Test Marketplace" }' --label "marketplace" --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client
```

Once instantiated, you can use `wasmcli query wasm list-contract-by-code <ID>` to query contract info.

### Execute Contract Method

```shell
wasmcli tx wasm execute <CONTRACT_ADDR> '{<json encoded params>}' --gas-prices="0.025ucosm" --gas="auto" --gas-adjustment="1.2" -y --from client
```
