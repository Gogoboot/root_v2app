// ============================================================
// root-network — домен P2P сети
//
// libp2p, Gossipsub, mDNS
// ============================================================

pub mod behaviour;
pub mod channels;
pub mod node;

pub use behaviour::{RootBehaviour, RootBehaviourEvent};
pub use channels::{P2pMessage, start_node_channels};
