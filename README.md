# Cosmons - A Nft example to manage digital collectibles


## How to build

To optimize your smart contracts you have to use:

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.10.4

## How to work with REPL 

npx @cosmjs/cli@^0.22 --init contracts/cosmons/helpers.ts 

Please consider that helper.ts is using local wasmd 0.11.1 instance



### Errata

Faucet is not supported 
