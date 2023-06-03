use criterion::{criterion_group, criterion_main, Criterion};
#[cfg(feature = "bench")]
mod run {
    use criterion::{criterion_group, criterion_main, Criterion};
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};
    use serde_columnar::columnar;
    lazy_static! {
        static ref STORE: VecStore = {
            let mut _data = Vec::new();
            for i in 0..10000 {
                _data.push(Data {
                    id: i / 50,
                    name: format!("name{}", i),
                });
            }
            VecStore { data: _data }
        };
        static ref NORMAL_STORE: NormalStore = {
            let mut _data = Vec::new();
            for i in 0..10000 {
                _data.push(NormalData {
                    id: i / 50,
                    name: format!("name{}", i),
                });
            }
            NormalStore { data: _data }
        };
    }

    #[columnar(vec, ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Data {
        #[columnar(strategy = "DeltaRle")]
        id: u64,
        #[columnar(strategy = "Rle")]
        name: String,
    }

    #[columnar(ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct VecStore {
        #[columnar(type = "vec")]
        pub data: Vec<Data>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NormalData {
        pub id: u64,
        pub name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NormalStore {
        pub data: Vec<NormalData>,
    }

    pub fn columnar_ende(c: &mut Criterion) {
        c.bench_function("columnar_encode", |b| {
            b.iter(|| {
                let _bytes = serde_columnar::to_vec(&*STORE).unwrap();
            })
        });
        c.bench_function("columnar_decode", |b| {
            let bytes = serde_columnar::to_vec(&*STORE).unwrap();
            b.iter(|| {
                let _store = serde_columnar::from_bytes::<VecStore>(&bytes).unwrap();
            })
        });
    }

    pub fn postcard_ende(c: &mut Criterion) {
        c.bench_function("postcard_encode", |b| {
            b.iter(|| {
                let _bytes = postcard::to_allocvec(&*NORMAL_STORE).unwrap();
            })
        });
        c.bench_function("postcard_decode", |b| {
            let bytes = postcard::to_allocvec(&*NORMAL_STORE).unwrap();
            b.iter(|| {
                let _store = postcard::from_bytes::<NormalStore>(&bytes).unwrap();
            })
        });
    }

    pub fn bincode_ende(c: &mut Criterion) {
        c.bench_function("bincode_encode", |b| {
            b.iter(|| {
                let _bytes = bincode::serialize(&*NORMAL_STORE).unwrap();
            })
        });
        c.bench_function("bincode_decode", |b| {
            let bytes = bincode::serialize(&*NORMAL_STORE).unwrap();
            b.iter(|| {
                let _store = bincode::deserialize::<NormalStore>(&bytes).unwrap();
            })
        });
    }
}
pub fn dumb(_c: &mut Criterion) {}
#[cfg(feature = "bench")]
criterion_group!(
    benches,
    run::columnar_ende,
    run::postcard_ende,
    run::bincode_ende
);
#[cfg(not(feature = "bench"))]
criterion_group!(benches, dumb);
criterion_main!(benches);
