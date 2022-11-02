#![no_main]
use columnar::fuzz::sample::Store;
use columnar::{ColumnarDecoder, ColumnarEncoder};
use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

fuzz_target!(|store: Store| {
    // fuzzed code goes here
    let mut encoder = ColumnarEncoder::new();
    store.serialize(encoder.deref_mut()).unwrap();
    let buf = encoder.into_bytes();
    let mut decoder = ColumnarDecoder::new(&buf);
    let store2 = Store::deserialize(decoder.deref_mut()).unwrap();
    assert_eq!(store, store2);
});
