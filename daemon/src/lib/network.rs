use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, time::Duration};

use libp2p::{
    futures::StreamExt, gossipsub, identity::Keypair, mdns, noise, swarm::{NetworkBehaviour, SwarmEvent}, tcp::Config, yamux, SwarmBuilder
};

use tokio::{io, io::AsyncBufReadExt, select};
use crate::{blockchain::Block, database::get_key_pair};

#[derive(NetworkBehaviour)]
struct P2PBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}


pub async fn initialize_p2p() {
    let keys = get_key_pair().unwrap();

    if let Some(key_pair) = keys {
        let rsa_pkey = key_pair.private_key.clone();
        let mut rsa_pkey_bytes = rsa_pkey;
        let libp2p_key_pair = Keypair::rsa_from_pkcs8(&mut rsa_pkey_bytes).unwrap();
        let mut swarm = SwarmBuilder::with_existing_identity(libp2p_key_pair)
            .with_tokio()
            .with_tcp(Config::default(), noise::Config::new, yamux::Config::default).unwrap()
            .with_quic()
            .with_behaviour(|key| {
                // To content-address message, we can take the hash of message and use it as an ID.
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };
    
                // Set a custom gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                    .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                    .build()
                    .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.
    
                // build a gossipsub network behaviour
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;
    
                let mdns =
                    mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
                Ok(P2PBehaviour { gossipsub, mdns })
            }).unwrap()
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        
        let topic = gossipsub::IdentTopic::new("test-net");
        
        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap()).unwrap();
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
        // Create a Gossipsub topic

        println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

        // Kick it off
        loop {
            select! {
                Ok(Some(line)) = stdin.next_line() => {
                    if let Err(e) = swarm
                        .behaviour_mut().gossipsub
                        .publish(topic.clone(), line.as_bytes()) {
                        println!("Publish error: {e:?}");
                    }
                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            println!("mDNS discovered a new peer: {peer_id}");
                        }
                    },
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discover peer has expired: {peer_id}");
                            swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => println!(
                            "Got message: '{}' with id: {id} from peer: {peer_id}",
                            String::from_utf8_lossy(&message.data),
                        ),
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Local node is listening on {address}");
                    }
                    _ => {}
                }
            }
        }
    }

}

