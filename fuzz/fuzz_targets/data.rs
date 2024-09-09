#![no_main]
use columnar_fuzz::Data;
use libfuzzer_sys::fuzz_target;
use serde_columnar::{from_bytes, to_vec};
fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    if let Ok(data) = from_bytes::<Data>(data) {
        // println!("data: {:?}", data);
        let buf = to_vec(&data).unwrap();
        let data2 = from_bytes(&buf).unwrap();
        assert_eq!(data, data2);
    }
});
