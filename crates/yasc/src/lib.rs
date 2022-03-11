pub mod collection;
pub mod util;
pub mod codegen;

pub type HashMap<K, V> = ahash::AHashMap<K, V>;
pub type HashSet<V> = ahash::AHashSet<V>;
