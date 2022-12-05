#![no_main]
use columnar_fuzz::NestedStore;
use serde_columnar::{from_bytes, to_vec};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|store: NestedStore| {
    // fuzzed code goes here
    let buf = to_vec(&store).unwrap();
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
});
