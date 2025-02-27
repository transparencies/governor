#![cfg(feature = "std")]

use assertables::*;
use futures_executor::block_on;
use futures_util::sink::SinkExt;
use governor::{prelude::*, Quota, RateLimiter};
use nonzero_ext::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(feature = "jitter")]
use governor::Jitter;

#[test]
#[allow(clippy::unit_cmp)]
fn sink() {
    let lim = Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(10u32))));
    let mut sink = Vec::new().ratelimit_sink(&lim);
    let i = Instant::now();

    for _ in 0..10 {
        block_on(sink.send(())).unwrap();
    }
    assert_lt!(i.elapsed(), Duration::from_millis(100));

    block_on(sink.send(())).unwrap();
    assert_in_range!(i.elapsed().as_millis(), 100..201);

    block_on(sink.send(())).unwrap();
    assert_in_range!(i.elapsed().as_millis(), 200..301);

    let result = sink.get_ref();
    assert_eq!(result.len(), 12);
    assert!(result.iter().all(|&elt| elt == ()));
}

#[test]
fn auxilliary_sink_methods() {
    let lim = Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(10u32))));
    // TODO: use futures_ringbuf to build a Sink-that-is-a-Stream and
    // use it as both
    // (https://github.com/najamelan/futures_ringbuf/issues/5)
    let mut sink = Vec::<u8>::new().ratelimit_sink(&lim);

    // Closes the underlying sink:
    assert!(block_on(sink.close()).is_ok());
}

#[cfg(all(feature = "jitter", test))]
#[cfg_attr(feature = "jitter", test)]
#[allow(clippy::unit_cmp)]
fn sink_with_jitter() {
    let lim = Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(10u32))));
    let mut sink =
        Vec::new().ratelimit_sink_with_jitter(&lim, Jitter::up_to(Duration::from_nanos(1)));
    let i = Instant::now();

    for _ in 0..10 {
        block_on(sink.send(())).unwrap();
    }
    assert_le!(i.elapsed(), Duration::from_millis(100),);

    block_on(sink.send(())).unwrap();
    assert_in_range!(i.elapsed().as_millis(), 100..201);

    block_on(sink.send(())).unwrap();
    assert_in_range!(i.elapsed().as_millis(), 200..301);

    let result = sink.into_inner();
    assert_eq!(result.len(), 12);
    assert!(result.into_iter().all(|elt| elt == ()));
}
