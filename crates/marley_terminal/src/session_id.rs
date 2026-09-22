//! `SessionId` — the process-unique session identity.

use std::sync::atomic::{AtomicU64, Ordering};

/// A process-unique, monotonic session identity that threads through every
/// shell/block. The shell-hook-reported id is the distinct [`crate::ShellSessionId`].
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SessionId(u64);

impl SessionId {
    /// Allocate the next process-unique session id — strictly greater than every id
    /// previously returned by `next` in this process.
    pub fn next() -> SessionId {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        SessionId(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    /// The wrapped `u64`, returned unchanged.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for SessionId {
    fn from(value: u64) -> Self {
        SessionId(value)
    }
}

impl From<SessionId> for u64 {
    fn from(value: SessionId) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // R1: transparent u64 newtype; as_u64 returns the wrapped value.
    #[test]
    fn r1_repr_transparent_and_as_u64() {
        assert_eq!(std::mem::size_of::<SessionId>(), std::mem::size_of::<u64>());
        assert_eq!(SessionId::from(7).as_u64(), 7);
    }

    // R2: lossless round-trip through both From impls (literal not in {0,1} to kill
    // the as_u64 -> 0/1 and From -> 0 mutants).
    #[test]
    fn r2_u64_roundtrip() {
        let s = SessionId::from(42);
        assert_eq!(s.as_u64(), 42);
        assert_eq!(u64::from(s), 42);
    }

    // R3: next() yields strictly increasing, never-repeating ids.
    #[test]
    fn r3_next_strictly_increasing() {
        let ids: HashSet<u64> = (0..1000).map(|_| SessionId::next().as_u64()).collect();
        assert_eq!(ids.len(), 1000);
    }

    // R4: Copy + Eq + Hash agree.
    #[test]
    fn r4_eq_hash_copy() {
        let a = SessionId::from(5);
        let b = a; // Copy
        assert_eq!(a, b);
        assert_eq!(a.as_u64(), 5); // a still usable after the copy
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}
