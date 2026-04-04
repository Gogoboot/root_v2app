// ============================================================
// root-network — домен P2P сети
//
// libp2p, Gossipsub, mDNS
// ============================================================

pub mod behaviour;
pub mod channels;
pub mod node;
pub mod topic;
pub mod error; 


pub use behaviour::{RootBehaviour, RootBehaviourEvent, build_gossipsub, private_topic, verify_message_sender};
pub use channels::{P2pMessage, P2pOutMessage, start_node_channels};
pub use topic::generate_topic_id;
pub use error::NetworkError;