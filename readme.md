poc for using libp2p gossipsub protocol 

todo:

- use kademlia for peer discovery. right now we do it by pinging a peer. see https://github.com/libp2p/rust-libp2p/discussions/2447

## usage

In one terminal:

`$ cd ozi && cargo run -- m "hello"`

The following will be printed:

Listening on *< address >*

------

In another terminal:

`$ cd ozi && cargo run --m "hello from 2" -p < address >`



