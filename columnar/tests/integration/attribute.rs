use serde::{Deserialize, Serialize};
use serde_columnar::{columnar, from_bytes, to_vec};

#[test]
fn derive_serialize() {
    #[columnar(ser)]
    struct A {}
    let bytes = to_vec(&A {}).unwrap();
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn derive_deserialize() {
    #[columnar(ser, de)]
    struct B {}
    let table: B = from_bytes(&[0]).unwrap();
    insta::assert_yaml_snapshot!(table);
}

#[test]
fn derive_serialize_generic() {
    #[columnar(ser, de)]
    struct A<P, Q>
    where
        P: Serialize + for<'a> Deserialize<'a>,
        Q: Serialize + for<'a> Deserialize<'a>,
    {
        p: P,
        q: Q,
    }

    let bytes = to_vec(&A { p: 1u64, q: 2u32 }).unwrap();
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn derive_deserialize_generic() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B<P, Q>
    where
        P: Serialize + for<'a> Deserialize<'a>,
        Q: Serialize + for<'a> Deserialize<'a>,
    {
        p: P,
        q: Q,
    }
    let table: B<u64, u32> = from_bytes(&[2, 1, 2]).unwrap();
    assert_eq!(table, B { p: 1u64, q: 2u32 });
    insta::assert_yaml_snapshot!(table);
}

#[test]
fn table_optional_serialize() {
    #[columnar(ser)]
    struct A {}
}
