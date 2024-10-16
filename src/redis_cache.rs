use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// Function to generate a hash from function inputs
pub fn hash_inputs<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
