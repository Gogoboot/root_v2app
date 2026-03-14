// ============================================================
// ROOT v2.0 — network/mod.rs
//
// Подмодули:
//   behaviour — RootBehaviour (Gossipsub + mDNS)
//   node      — start_node (интерактивный режим)
//   channels  — start_node_channels (Flutter FFI режим)
// ============================================================

pub mod behaviour;
pub mod node;
pub mod channels;

pub use behaviour::{RootBehaviour, RootBehaviourEvent};
pub use channels::{start_node_channels, P2pMessage};
