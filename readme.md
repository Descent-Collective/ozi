POC for using libp2p gossipsub protocol to broadcast signed price messages for relayer clients to pick it up and send to the oracle smart contract.

**TODO:**
- [x] use kademlia for peer discovery. right now we do it by pinging a peer. see https://github.com/libp2p/rust-libp2p/discussions/2447
- [x] Get price for a particular collateral pair(e.g USDC/NGN) from listed exchanges(e.g. cryptocompare)
- [x] Sign price data using nodes private keys
- [x] Use variables from config to make API calls to exchanges.
- [ ] implement price fetching for more exchanges.

## usage

In one terminal:

`$ cargo run -p relayer -- --port <PORT>`

The following will be printed:

Listening on *< address >*

------

Update node_config.toml with the correct relayer address. For local testing, use localhost:*< PORT >*

Then in another terminal:

`$ cargo run -p node`



