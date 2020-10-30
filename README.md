# Cosmons - An NFT Example for Managing Digital Collectibles

## How to Build

In order to optimize your smart contracts, you have to use:

```shell
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.4
```

## How to Work with REPL

`npx @cosmjs/cli@^0.23 --init contracts/cosmons/helpers.ts`

:warning: Please, keep in mind that helper.ts uses a local wasmd 0.11.1 instance (localnetOptions). Please, update your parameters accordingly.

For hackatom net, you need to use `defaultOptions` in useOptions.

### Using a contract

```typescript
// Hackatom_V Net
const client = await useOptions(defaultOptions).setup(<YOUR PASSWORD>);
const partner = await useOptions(defaultOptions).setup(<YOUR PASSWORD>, "/Users/user/.hackatom2.key");

// localnet
const client = await useOptions(localnetOptions).setup(<YOUR PASSWORD>, "/Users/user/localnet.key");
const partner = await useOptions(localnetOptions).setup(<YOUR PASSWORD>, "/Users/user/localnet2.key");

const address = client.senderAddress;
const partnerAddr = partner.senderAddress;
```

### Get the Factory

```typescript
const cw721 = CW721(client);
```

### Verify Funds in the Account

```typescript
client.getAccount()
partner.getAccount()
```

### Use Existing Accounts

You can skip this section if followed this transcript up until here.

```typescript
const fred = "cosmos1rgd5jtgp22vq44xz4c69x5z9mu0q92ujcnqdgw";
const bob = "cosmos1exmd9ml0adgkuggd6knqcjgw4e3x84r4hhfr07";
```

Query accounts:
`wasmcli query account $(wasmcli keys show -a fred)`
`wasmcli query account $(wasmcli keys show -a vhx)`

### Instantiate the Contract

```typescript
const codeId = <your CodeID>; // wasmcli q wasm list-code & find your contract ID
const initMsg = { name: "Cosmons", symbol: "mons",  minter: address };
const contract = await client.instantiate(codeId, initMsg, "Virtual Cosmons 1");
```

**OR**

```typescript
const contract = client.getContracts(<your codeID>); // And check for your contractAddress
```

### Use Contract

```typescript
const mine = cw721.use(contract.contractAddress);
```

### Mint a Cosmon NFT

```typescript
mine.mint("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb", address, "Cosmos", "Minted Cosmon!");
```

### Approve Token Transfer

> :warning: Needs to be called before `transferNft`.

```typescript
mine.approve(address, "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb");
```

### Revoke Token Transfer

> :warning: `transferNft` will not work after using `revoke`.

```typescript
mine.revoke(address, "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb");
```

### Transfer Token to Partner

> :warning: Needs to be called after `approve`.

```typescript
mine.transferNft(partnerAddr, "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb");
```

#### Queries

```typescript
mine.nftInfo("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb")
mine.ownerOf("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb")
mine.numTokens()
mine.tokens(address, "", 10)
mine.allNftInfo("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb")
mine.allTokens("", 10)
```

### Errata

Faucet is not supported.

## Licenses

This repo contains two license, [Apache 2.0](./LICENSE-APACHE) and
[AGPL 3.0](./LICENSE-AGPL.md). All crates in this repo may be licensed
as one or the other. Please check the `NOTICE` in each crate or the 
relevant `Cargo.toml` file for clarity.

All *specifications* will always be Apache-2.0. All contracts that are
meant to be *building blocks* will also be Apache-2.0. This is along
the lines of Open Zepellin or other public references.

Contracts that are "ready to deploy" may be licensed under AGPL 3.0 to 
encourage anyone using them to contribute back any improvements they
make. This is common practice for actual projects running on Ethereum,
like Uniswap or Maker DAO.

