POC for using libp2p gossipsub protocol to broadcast signed price messages for relayer clients to pick it up and send to the oracle smart contract.

**TODO:**
- use kademlia for peer discovery. right now we do it by pinging a peer. see https://github.com/libp2p/rust-libp2p/discussions/2447
- Get price for a particular collateral pair(e.g USDC/NGN) from listed exchanges(e.g. cryptocompare)
- Sign price data using nodes private keys
- Setup config for variables nodes will have to set when running Ozi:( PRIVATE_KEY, EXCHANGE_NAME, EXCHANGE_API_KEY, PRICE_PAIR)

## usage

In one terminal:

`$ cd ozi && cargo run -- m "hello"`

The following will be printed:

Listening on *< address >*

------

In another terminal:

`$ cd ozi && cargo run --m "hello from 2" -p < address >`



