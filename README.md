# Cosmons - A Nft example to manage digital collectibles


## How to build

To optimize your smart contracts you have to use:

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.4

## How to work with REPL 

npx @cosmjs/cli@^0.23 --init contracts/cosmons/helpers.ts 

Please consider that helper.ts is using local wasmd 0.11.1 instance. Please update it to your parameters. 


You can use ```heldernetOptions``` in useOptions instead

### Using a contract

Option 1:

```
// Local 
const client = await useOptions(defaultOptions).setup(<YOUR PASSWORD>);
const partner = await useOptions(defaultOptions).setup(<YOUR PASSWORD>, "/Users/vhahn/.localnet2.key");

// or Heldernet
const client = await useOptions(heldernetOptions).setup(<YOUR PASSWORD>, "/Users/vhahn/.heldernet.key");
const partner = await useOptions(heldernetOptions).setup(<YOUR PASSWORD>, "/Users/vhahn/.heldernet2.key");

const address = client.senderAddress;
const partnerAddr = partner.senderAddress;
```

### Get the factory

```
const cw721 = CW721(client);
```

### Verify amount in the account

```
client.getAccount()
partner.getAccount()
```

### Use existing Accounts

You can skip this section if followed this transcript until here.
```
const fred = "cosmos1rgd5jtgp22vq44xz4c69x5z9mu0q92ujcnqdgw";
const bob = "cosmos1exmd9ml0adgkuggd6knqcjgw4e3x84r4hhfr07";
```
Query accounts:
wasmcli query account $(wasmcli keys show -a fred) 
wasmcli query account $(wasmcli keys show -a vhx) 

### Initiate Contract

```
const codeId = <your CodeID>; // wasmcli q wasm list-code & find your contract ID
const initMsg = { name: "Cosmons", symbol: "mons",  minter: address };
const contract = await client.instantiate(codeId, initMsg, "Virtual Cosmons 1");
```
or
```
const contract = client.getContracts(<your codeID>); // And check for your contractAddress
```

### Use our contract

```
const mine = cw721.use(contract.contractAddress);
```

### Let us mint an asset

```
mine.mint("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb", address, "Cosmos", "Some Text");
```

### Transfer Token to Partner
mine.transferNft(partnerAddr, "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb");

### Approve Contract
// Don't call transferNft before, otherwise it fails
mine.approve(address, "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb");

#### Queries

```
mine.nftInfo("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb")
mine.ownerOf("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb")
mine.numTokens()
mine.tokens(address, "", 10)
mine.allNftInfo("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsb")
mine.allTokens("", 10)
```

### Errata

Faucet is not supported

