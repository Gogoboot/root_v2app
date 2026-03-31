// root-network/src/topic.rs
use sha2::{Sha256, Digest};

pub fn generate_topic_id(peer_a: &str, peer_b: &str) -> String {
    let (first, second) = if peer_a < peer_b { (peer_a, peer_b) } else { (peer_b, peer_a) };
    let mut hasher = Sha256::new();
    hasher.update(first.as_bytes());
    hasher.update(second.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..])[..32].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_order_independence() {
        let a = "ed25519:abc123";
        let b = "ed25519:def456";
        assert_eq!(generate_topic_id(a, b), generate_topic_id(b, a));
    }
}
