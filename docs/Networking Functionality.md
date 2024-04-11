# Networking Functionality
The document outlines the approach to networking segment of this project. This includes how peers connect and communicate updates to eachother to fascilitate the shared blockchains.

## LibP2P
We are using the libp2p library for networking functionality between peers. Using the Swarm and SwarmEvent objects provided, it is straightforward to implement specific actions and listeners for those actions on each node.

Documentation for libp2p can be found [here](https://docs.rs/libp2p/latest/libp2p/index.html).

## Swarm Events
These are events that peers can send to eachother for signals and updates.  Data for each signal/update will be serialized in JSON format for easy transport.

### Chain Length Signal
Signal sent to network to indicate the current length of a local chain.
- action: **ChainLength**
- data: 
  ```
  {
    id: integer,
    length: integer
  }
  ```

### Update Chain
Event to send chain update(s) to specific remote peer.
- action: **ChainUpdate**
- data: 
  ```
  {
    id: integer,
    blocks: [
        {
            id: integer,
            timestamp: integer,
            data: string,
            previous_hash: string,
            hash: string,
            provider_key: string,
            data_hash: string
        }
    ]
  }
  ```

### Request Chain Update Signal
Signal to network that this peer needs the most recent chain.  Signal provides the current chain length for remote peer to determine if they have a longer chain to provide.
- action: **RequestChainUpdate**
- data: 
  ```
  {
    id: integer,
    length: integer
  }
  ```

### Share Group Key
Event to share a group key with a peer for a specific chain.  This can either be targeted or sent out to entire network of authorized peers.
- action: **GroupKey**
- data: 
  ```
  {
    id: integer,
    key: string
  }
  ```

### Access Revoked Signal
Signal to specific peer that access has been revoked for a certain chain.
- action: **AccessRevoked**
- data: 
  ```
  {
    id: integer
  }
  ```