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

    #[columnar(vec)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Data {
        #[columnar(strategy = "Rle")]
        id: u64,
        name: String,
    }

    #[columnar]
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
        c.bench_function("columnar_ende", |b| {
            b.iter(|| {
                let bytes = serde_columnar::to_vec(&*STORE).unwrap();
                let store = serde_columnar::from_bytes::<VecStore>(&bytes).unwrap();
                assert_eq!(store, *STORE);
            })
        });
    }

    pub fn postcard_ende(c: &mut Criterion) {
        c.bench_function("postcard_ende", |b| {
            b.iter(|| {
                let bytes = postcard::to_allocvec(&*NORMAL_STORE).unwrap();
                let store = postcard::from_bytes::<NormalStore>(&bytes).unwrap();
                assert_eq!(store, *NORMAL_STORE);
            })
        });
    }

    pub fn bincode_ende(c: &mut Criterion) {
        c.bench_function("bincode_ende", |b| {
            b.iter(|| {
                let bytes = bincode::serialize(&*NORMAL_STORE).unwrap();
                let store = bincode::deserialize::<NormalStore>(&bytes).unwrap();
                assert_eq!(store, *NORMAL_STORE);
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
