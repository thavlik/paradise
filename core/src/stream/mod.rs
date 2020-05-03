pub mod rx;
pub mod tx;

fn cycle(parity: &std::sync::atomic::AtomicUsize) -> usize {
    let original: usize = parity.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let wrapped = original % 2;
    if original > 100_000_000 {
        // Wrap parity back to [0, 1] so there's no risk of overflow.
        // fetch_add returns the old value, so the current value will
        // (functionally) be the complement. This is *only* okay
        // because we know we're the only thread that is writing to
        // parity. Note that the write is non-transactional and could
        // otherwise introduce a race condition.
        parity.store(1 - wrapped, std::sync::atomic::Ordering::SeqCst);
    }
    wrapped
}
