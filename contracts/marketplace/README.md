# Marketplace Smart Contract

This smart contract enables trading of cw721 NFT tokens. To accomplish this the contract implements a *receiveNft* function which acts as a counterpart to the cw721 *sendNft* function. It also provides further functions for trading.

## Use Cases

### Buying/Selling an NFT

* Putting NFT/s up for sale for a fixed token price
* Buying NFT/s put up for sale for the specified token price

### NFT Auction

* Putting NFT/s up for auction with a time limit
* Bidding on NFT/s put up for auction

### Trade NFT/s for NFT/s

* Trade NFT/s with NFT/s from another person

## Messages

| Message                             | Description                                                                           | Status |
|:------------------------------------|:--------------------------------------------------------------------------------------|:------:|
| `receiveNft(sender, token_id, msg)` | Counter-part to `sendNft`, handling the receival of a token in the receiving contract | :x:    |
| `sellNft(token_id, list_price)`     | Sells a token for a specified price                                                   | :x:    |
| `buyNft(token_id)`                  | Buys a token for the price it has been put up for sale                                | :x:    |
| `withdrawNft(token_id)`             | Withdraws a token offering                                                            | :x:    |

## Queries

| Query            | Description                                                                                                               | Status |
|:-----------------|:--------------------------------------------------------------------------------------------------------------------------|:------:|
| `getOfferings()` | Retrieves a list of all current offerings (seller address, token information, contract address the token originated from) | :x:    |
