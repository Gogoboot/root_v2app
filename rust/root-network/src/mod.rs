// ============================================================
// ROOT v2.0 — network/mod.rs
//
// Подмодули:
//   behaviour — RootBehaviour (Gossipsub + mDNS)
//   node      — start_node (интерактивный режим)
//   channels  — start_node_channels (Flutter FFI режим)
// ============================================================

pub mod behaviour;
pub mod channels;
pub mod node;

pub use behaviour::{RootBehaviour, RootBehaviourEvent};
pub use channels::{P2pMessage, start_node_channels};
