use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, time::Duration};
use libp2p::{
    futures::StreamExt, gossipsub, identify, identity::Keypair, noise, swarm::{NetworkBehaviour, SwarmEvent}, tcp::Config, yamux, Multiaddr, SwarmBuilder
};
use serde_json::{from_str, to_string, to_value, Map, Value};
use serde::{Deserialize, Serialize};
use tokio::{io, select};
use tokio::sync::mpsc::{Receiver, Sender};
use crate::database::get_key_pair;

const DEFAULT_PORT: i32 = 24195;

#[derive(NetworkBehaviour)]
struct P2PBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour
}
#[derive(Debug, Serialize, Deserialize)]
pub struct P2PRequest {
    pub action: String,
    pub parameters: Map<String, Value>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2PResponse {
    pub ok: bool,
    pub data: Value,
}

pub async fn initialize_p2p_thread(mut receiver_from_blockchain: Receiver<String>, mut sender_to_blockchain: Sender<String>) {
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
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .message_id_fn(message_id_fn) 
                    .build()
                    .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;
    
                // build a gossipsub network behaviour
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let identify = identify::Behaviour::new(identify::Config::new(
                    "/ipfs/id/1.0.0".to_string(),
                    key.public(),
                ));
    
                Ok(P2PBehaviour { gossipsub, identify })
            }).unwrap()
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();
        
        let topic = gossipsub::IdentTopic::new("authorize_chain");
        
        swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", DEFAULT_PORT).parse().unwrap()).unwrap();
        
        // Kick it off
        loop {
            select! {
                Some(msg) = receiver_from_blockchain.recv() => {
                    let request = from_str::<P2PRequest>(&msg).unwrap();
                    match request.action.as_str() {
                        "add-provider" => {
                            let ip_address = request.parameters.get("ip").unwrap().as_str().unwrap().to_string();
                            println!("{}", ip_address);

                            let multiaddr_str = format!("/ip4/{}/tcp/{}", ip_address, DEFAULT_PORT);
                            let remote = multiaddr_str.parse::<Multiaddr>().unwrap();
                            swarm.dial(remote).unwrap();
                            
                            let chain_id = to_string(request.parameters.get("chain_id").unwrap()).unwrap();
                            let shared_key = to_string(request.parameters.get("shared_key").unwrap()).unwrap();
                            
                            // Send message to dialed peer
                        },
                        _ => {}
                    }
                }
                event = swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {address:?}"),
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => println!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    ),
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Identify(identify::Event::Sent { peer_id })) => {
                        println!("Sent identify info to {peer_id:?}")
                    },
                    // Prints out the info received via the identify event
                    SwarmEvent::Behaviour(P2PBehaviourEvent::Identify(identify::Event::Received { peer_id, .. })) => {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        println!("Connected to peer with id {peer_id}");
                    }
                    _ => {}
                }
            }
        }
    }

}

