POC for using libp2p gossipsub protocol to broadcast signed price messages for relayer clients to pick it up and send to the oracle smart contract.

**TODO:**
[x] use kademlia for peer discovery. right now we do it by pinging a peer. see https://github.com/libp2p/rust-libp2p/discussions/2447
[] Get price for a particular collateral pair(e.g USDC/NGN) from listed exchanges(e.g. cryptocompare)
[] Sign price data using nodes private keys
[] Use variables from config to make API calls to exchanges.

## usage

In one terminal:

`$ cd ozi && cargo run -- m "hello"`

The following will be printed:

Listening on *< address >*

------

In another terminal:

`$ cd ozi && cargo run --m "hello from 2" -p < address >`



