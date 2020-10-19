# Cosmos - A Nft example to manage pokemons or any other digital assets 


## How to build

To optimize your smart contracts you have to use:

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.4

## How to work with REPL 

npx @cosmjs/cli@^0.22 --init contracts/cosmons/helpers.ts 

Please consider that helper.ts is using local wasmd instance


### Using a contract

Option 1:

```
const client = await useOptions(defaultOptions).setup(<YOUR PASSWORD>;
const partner = await useOptions(defaultOptions).setup(<YOUR PASSWORD>, "</home/user>.localnet2.key");
const address = client.senderAddress;
const partnerAddr = partner.senderAddress;
```

### Get the factory

```
const cw721 = CW721(client);
```

### Verify Amount

```
client.getAccount()
```

### Use existing Accounts

```
const fred = "cosmos1rgd5jtgp22vq44xz4c69x5z9mu0q92ujcnqdgw";
const bob = "cosmos1exmd9ml0adgkuggd6knqcjgw4e3x84r4hhfr07";
```

wasmcli query account $(wasmcli keys show -a fred) 
wasmcli query account $(wasmcli keys show -a vhx) 

### Initiate Contract

```
const codeId = 12; // wasmcli q wasm list-code & find your contract
const initMsg = { name: "Cosmons", symbol: "mons",  minter: address };
const contract = await client.instantiate(codeId, initMsg, "Virtual Cosmons 3");
```
or
```
const contract1 = client.getContracts(12);
```

### Use our contract

```
const mine = cw721.use(contract.contractAddress);
```

### Let us mint an asset

```
const mintMsg = { token_id: "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsy", owner: address, name: "Cosmos", description: "Some Text" };

mine.mint("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsy", address, "Cosmos", "Some Text");
```

// Alternativ const exec = await client.execute(contract.contractAddress, { mint: { token_id: "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsy", owner: address, name: "Cosmons", description: "Some Text" } });
// 
```
mine.nftInfo("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsy")
mine.ownerOf("monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsy")
```

### Transfer Token to Partner
mine.transferNft(partnerAddr, "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsy");


#### Advanced tips


##### Second wallet

const { setup } = useOptions(defaultOptions);
const minter = await setup("mway12345", "/Users/vhahn/.localnet2.key")

const minter = await useOptions(defaultOptions).setup("mway12345", "/Users/vhahn/.localnet2.key");

Sometimes helper doesn't support any interfaces a contract provides, especially at the beginning, when you try to implement the helper.ts. Therefore you can access the wasm/smart contranct interface, like here:

const execMsg2 = ExecMsg { mint: { token_id: token_id, owner: address, name: "Cosmos 3" } };
const result = await client.execute(contractAddress, {mint: {mintMsg}});
const exec02 = await client.execute(address, { mint: { token_id: "coral1hzllnaf9tezv578p3ysmml6j00ysdac4xwly9w", owner: address, name: "Cosmos 3", description: "abc" } });
exec.logs[0].events[0];


const exec3 = await client.execute(address, { mint: { token_id: "cosmos1rgd5jtgp22vq44xz4c69x5z9mu0q92ujcnqdgw", owner: partnerAddr, name: "Cosmons", description: "Some Text" } });

const exec8 = await client.execute(contract1.contractAddress, { mint: { token_id: "monster112a9lf95atqvyejqe22xnna8x4mfqd75tkq2kvwcjyysarcsy", owner: address, name: "Cosmons", description: "Some Text" } });

minterClient1.sendTokens(address, [ { denom: 'ucosm', amount: '200000' }])


#### Check contracts
```
client.getContracts(codeId)
```

// Check raw data
```
const key = new Uint8Array([0, 6, ...toAscii("config")]);
const raw = await client.queryContractRaw(contract.contractAddress, key)
```

#### Queries
GetID() string
GetOwner() sdk.AccAddress
SetOwner(address sdk.AccAddress) -> CW1
GetTokenURI() string
EditMetadata(tokenURI string)
String() string

type Collection struct {
    Denom string
    NFTs  NFTs
}
#### Actions 

type MsgMintNFT struct {
    Sender    sdk.AccAddress
    Recipient sdk.AccAddress
    ID        string
    Denom     string
    TokenURI  string
}

type MsgTransferNFT struct {
    Sender    sdk.AccAddress
    Recipient sdk.AccAddress
    Denom     string
    ID        string
}

type MsgEditNFTMetadata struct {
    Sender   sdk.AccAddress
    ID       string
    Denom    string
    TokenURI string
}
### Errata

Faucet is not supported 

partner.sendTokens(client.senderAddress, [ { denom: 'ucosm', amount: '200000' }])