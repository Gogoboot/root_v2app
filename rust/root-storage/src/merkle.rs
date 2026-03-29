// ============================================================
// ROOT v2.0 — storage/merkle.rs
// MerkleTree — верификация целостности сообщений
// ============================================================

use sha2::{Digest, Sha256};

pub struct MerkleTree {
    /// Листья дерева — хеши сообщений
    pub leaves: Vec<[u8; 32]>,
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree { leaves: Vec::new() }
    }

    /// Добавить хеш нового сообщения
    pub fn add_leaf(&mut self, hash: [u8; 32]) {
        self.leaves.push(hash);
    }

    /// Вычислить корневой хеш всего дерева
    pub fn root(&self) -> Option<[u8; 32]> {
        if self.leaves.is_empty() {
            return None;
        }
        if self.leaves.len() == 1 {
            return Some(self.leaves[0]);
        }

        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            let mut i = 0;
            while i < current_level.len() {
                let left = current_level[i];
                let right = if i + 1 < current_level.len() {
                    current_level[i + 1]
                } else {
                    current_level[i] // дублируем последний если нечётное
                };
                let mut hasher = Sha256::new();
                hasher.update(left);
                hasher.update(right);
                next_level.push(hasher.finalize().into());
                i += 2;
            }
            current_level = next_level;
        }

        Some(current_level[0])
    }

    /// Получить proof путь для листа по индексу
    pub fn proof(&self, leaf_index: usize) -> Option<Vec<[u8; 32]>> {
        if leaf_index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_level = self.leaves.clone();
        let mut index = leaf_index;

        while current_level.len() > 1 {
            let sibling_index = if index.is_multiple_of(2) {
                (index + 1).min(current_level.len() - 1)
            } else {
                index - 1
            };
            proof.push(current_level[sibling_index]);

            let mut next_level = Vec::new();
            let mut i = 0;
            while i < current_level.len() {
                let left = current_level[i];
                let right = if i + 1 < current_level.len() {
                    current_level[i + 1]
                } else {
                    current_level[i]
                };
                let mut hasher = Sha256::new();
                hasher.update(left);
                hasher.update(right);
                next_level.push(hasher.finalize().into());
                i += 2;
            }
            current_level = next_level;
            index /= 2;
        }

        Some(proof)
    }

    /// Верифицировать лист по proof пути
    pub fn verify(
        leaf_hash: [u8; 32],
        leaf_index: usize,
        proof: &[[u8; 32]],
        root: [u8; 32],
    ) -> bool {
        let mut current = leaf_hash;
        let mut index = leaf_index;

        for sibling in proof {
            let mut hasher = Sha256::new();
            if index.is_multiple_of(2) {
                hasher.update(current);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(current);
            }
            current = hasher.finalize().into();
            index /= 2;
        }

        current == root
    }

    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}
