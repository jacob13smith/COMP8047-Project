# Blockchain Functionality
This document is where I keep rough notes about the design of each component of the blockchain.

## Creating a new chain
This requires at least creating a 'genesis' block for a user.  This is simple to implement with the following steps:
1. Generate a new "shared key" for the encryption of users data. The key will be saved to the database but not distributed unless granting access to a specific user.

2. Create a new block with the following custom properties
    - block_id = 0
    - previous_hash_field = 0
    - data = set user fields action with input

    The other fields are set as normal blocks would be
    - provider_public_key = user's public key
    - data hash = checksum of unencrypted data

3. Save new block to local database. New user and chain created.

## Granting access to new user
This is a core functionality of the system.  If the current user knows the IP address of the healthcare provider that they want to give record access, they can authorize the system running at that IP address.

Adding and revoking a provider will count as a block each on the chain.  Here are the steps for granting access:
1. The data for the block will be an 'add provider' action with the provider's name and IP address in the data field.
2. Save the block to the local database.
3. Distribute the block to all nodes authorized by chain (except the new provider).
4. Connect with provider system.
5. Send shared group key to the provider, then transmit the entire chain.

## Revoking access from a user
A user with access to a blockchain can revoke access to other users on the blockchain. This is important to discontinue access for stale or compromised providers.

1. Generate a new shared key.
2. Starting at the genesis block, decrpyt the data field with the old shared key, and re-encrypt it with the new key.
3. Save re-encrypted blockchain to local database.
4. The data for the next block will be a 'remove provider' action with provider's public key in the field (using the new shared key).
5. Save the block to the local database.
6. Connect with each authorized peer for the chain and distribute the new shared key.
7. Each node will do an identical process of re-encryption and save the chain.
8. Once a node is done re-encrypting and saving the data, it will ask the network for an updated chain. This should be readily provided by at least the current node.
9. Connect with revoked system and send message that access has been revoked for blockchain of given id.

## Adding medical record/updating user
This process is to add a medical record to a user's chain, or update their core info.

1. Create a block with new data and all valid header fields.
2. Connect with all authorized peers to indicate that their chains are stale and need updating.
3. Each peer will (after a random delay) request the updated chain from the network.
4. A node with the updated chain will respond with latest block.

## Dealing with concurrent updates
The system has to deal with the problem of 2 nodes with divergent chains (ie, different blocks).  For simplicity, the nodes will follow the Longest Chain rule, where the longest valid chain is accepted.  If two chains are the same length, then the first one received is accepted.