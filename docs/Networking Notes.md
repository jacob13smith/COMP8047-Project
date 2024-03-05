# Networking Notes
The document outlines the approach to networking segment of this project. This includes how peers connect and communicate updates to eachother to fascilitate the shared blockchains.

## LibP2P
We are using the libp2p library for networking functionality between peers. Using the Swarm and SwarmEvent objects provided, it is straightforward to implement specific actions and listeners for those actions on each node.

Documentation for libp2p can be found [here](https://docs.rs/libp2p/latest/libp2p/index.html).

## Swarm Event Actions
These are actions that peers can send to eachother to signal and update.