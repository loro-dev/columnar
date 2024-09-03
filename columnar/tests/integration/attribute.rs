use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

use serde::{Deserialize, Serialize};
use serde_columnar::{columnar, from_bytes, to_vec, DeltaRleable, Rleable};

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
    let b: B = from_bytes(&[0]).unwrap();
    insta::assert_yaml_snapshot!(b);
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
    let b: B<u64, u32> = from_bytes(&[2, 1, 2]).unwrap();
    assert_eq!(b, B { p: 1u64, q: 2u32 });
    insta::assert_yaml_snapshot!(b);
}

#[test]
fn derive_serialize_lifetime() {
    #[columnar(ser)]
    struct A<'s, 'd, P, Q>
    where
        P: Serialize + for<'a> Deserialize<'a>,
        Q: Serialize + for<'a> Deserialize<'a>,
    {
        p: P,
        q: Q,
        s: &'s str,
        d: &'d str,
    }

    let bytes = to_vec(&A {
        p: 1u64,
        q: 2u32,
        s: "a",
        d: "b",
    })
    .unwrap();
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn derive_deserialize_lifetime() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B<P, Q, 's>
    where
        P: Serialize + for<'a> Deserialize<'a>,
        Q: Serialize + for<'a> Deserialize<'a>,
    {
        p: P,
        q: Q,
        s: &'s str,
    }
    let table: B<u64, u32> = from_bytes(&[3, 1, 2, 1, 97]).unwrap();
    assert_eq!(
        table,
        B {
            p: 1u64,
            q: 2u32,
            s: "a"
        }
    );
}

#[test]
fn derive_serialize_skip() {
    #[columnar(ser)]
    #[derive(Debug, PartialEq)]
    struct A {
        a: u64,
        #[columnar(skip)]
        b: u32,
    }
    let s = A { a: 1, b: 2 };
    let bytes = to_vec(&s).unwrap();
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn derive_deserialize_skip() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A {
        a: u64,
        #[columnar(skip)]
        b: u32,
    }
    let s = A { a: 1, b: 2 };
    let bytes = to_vec(&s).unwrap();
    let a: A = from_bytes(&bytes).unwrap();
    assert_eq!(a, A { a: 1, b: 0 });
}

#[test]
fn derive_deserialize_borrow_str() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A<'a> {
        a: u64,
        #[columnar(borrow)]
        b: Cow<'a, str>,
    }
    let s = A {
        a: 1,
        b: Cow::Borrowed("hello"),
    };
    let bytes = to_vec(&s).unwrap();
    let a: A = from_bytes(&bytes).unwrap();
    if let Cow::Owned(_) = &a.b {
        panic!("should be borrowed")
    }
    assert_eq!(s, a)
}

#[test]
fn derive_deserialize_borrow_bytes() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A<'a> {
        a: u64,
        #[columnar(borrow)]
        b: Cow<'a, [u8]>,
    }
    let s = A {
        a: 1,
        b: Cow::Borrowed(&[4, 5, 6]),
    };
    let bytes = to_vec(&s).unwrap();
    let a: A = from_bytes(&bytes).unwrap();
    if let Cow::Owned(_) = &a.b {
        panic!("should be borrowed")
    }
    assert_eq!(s, a)
}

#[test]
fn derive_deserialize_borrow_str_optional() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A<'a> {
        a: u64,
        #[columnar(borrow, optional, index = 0)]
        b: Cow<'a, str>,
    }
    let s = A {
        a: 1,
        b: Cow::Borrowed("hello"),
    };
    let bytes = to_vec(&s).unwrap();
    let a: A = from_bytes(&bytes).unwrap();
    if let Cow::Owned(_) = &a.b {
        panic!("should be borrowed")
    }
    assert_eq!(s, a)
}

#[test]
fn table_optional_serialize() {
    #[columnar(ser)]
    struct A {
        default: u64,
        #[columnar(optional, index = 0)]
        a: u32,
    }
    let bytes = to_vec(&A { default: 1, a: 2 }).unwrap();
    println!("{:?}", bytes);
    // insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn table_optional_deserialize() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A {
        default: u64,
        #[columnar(optional, index = 0)]
        a: u32,
    }
    let a: A = from_bytes(&[2, 1, 0, 1, 2]).unwrap();
    assert_eq!(
        a,
        A {
            default: 1u64,
            a: 2u32
        }
    );
    insta::assert_yaml_snapshot!(a);
}

#[test]
fn table_add_optional() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A {
        default: u64,
        #[columnar(optional, index = 0)]
        a: u32,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B {
        default: u64,
        #[columnar(optional, index = 0)]
        a: u32,
        #[columnar(optional, index = 1)]
        b: u32,
    }
    let old_bytes = to_vec(&A { default: 1, a: 2 }).unwrap();
    let read_old_b: B = from_bytes(&old_bytes).unwrap();
    assert_eq!(
        read_old_b,
        B {
            default: 1,
            a: 2,
            b: Default::default()
        }
    )
}

#[test]
fn table_delete_optional() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A {
        default: u64,
        #[columnar(optional, index = 0)]
        a: u32,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B {
        default: u64,
        #[columnar(optional, index = 0)]
        a: u32,
        #[columnar(optional, index = 1)]
        b: u32,
    }
    let old_bytes = to_vec(&B {
        default: 1,
        a: 2,
        b: 3,
    })
    .unwrap();
    let read_old_a: A = from_bytes(&old_bytes).unwrap();
    assert_eq!(read_old_a, A { default: 1, a: 2 })
}

#[test]
fn table_resort_optional() {
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct A {
        default: u64,
        #[columnar(optional, index = 0)]
        a: u32,
        #[columnar(optional, index = 2)]
        b: String,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B {
        default: u64,
        #[columnar(optional, index = 2)]
        b: String,
        #[columnar(optional, index = 0)]
        a: u32,
    }
    let old_bytes = to_vec(&A {
        default: 1,
        a: 2,
        b: "a".to_string(),
    })
    .unwrap();
    let read_old_b: B = from_bytes(&old_bytes).unwrap();
    assert_eq!(
        read_old_b,
        B {
            default: 1,
            a: 2,
            b: "a".to_string()
        }
    )
}

#[test]
fn row_vec_ser() {
    #[columnar(vec, ser)]
    #[derive(Clone)]
    struct A {
        a: u64,
    }
    #[columnar(ser)]
    struct B {
        #[columnar(class = "vec")]
        data: Vec<A>,
    }
    let init_b = B {
        data: vec![A { a: 1 }, A { a: 2 }],
    };
    let bytes = to_vec(&init_b).unwrap();
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn row_vec_de() {
    #[columnar(vec, ser, de)]
    #[derive(Clone, Debug, PartialEq)]
    struct A {
        a: u64,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B {
        #[columnar(class = "vec")]
        data: Vec<A>,
    }
    let init_b = B {
        data: vec![A { a: 1 }, A { a: 2 }],
    };
    let bytes = vec![1, 1, 3, 2, 1, 2];
    let b: B = from_bytes(&bytes).unwrap();
    assert_eq!(init_b, b);
}

#[test]
fn row_map_ser() {
    #[columnar(map, ser)]
    #[derive(Clone)]
    struct A {
        a: u64,
    }
    #[columnar(ser)]
    struct B {
        #[columnar(class = "map")]
        data: BTreeMap<u8, A>,
    }
    let data = vec![(1, A { a: 1 }), (2, A { a: 2 })].into_iter().collect();
    let init_b = B { data };
    let bytes = to_vec(&init_b).unwrap();
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn row_map_de() {
    #[columnar(map, ser, de)]
    #[derive(Clone, Debug, PartialEq)]
    struct A {
        a: u64,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B {
        #[columnar(class = "map")]
        data: HashMap<u8, A>,
    }
    let data = vec![(1, A { a: 1 }), (2, A { a: 2 })].into_iter().collect();
    let init_b = B { data };
    let bytes = vec![1, 2, 2, 1, 2, 3, 2, 1, 2];
    let b: B = from_bytes(&bytes).unwrap();
    assert_eq!(init_b, b);
}

#[test]
fn used_columnar() {
    #[columnar(vec, ser)]
    #[derive(Clone)]
    struct A {
        a: u64,
    }
    #[columnar(ser)]
    struct B {
        #[columnar(class = "vec")]
        data: Vec<A>,
    }

    #[columnar(vec, ser)]
    #[derive(Clone)]
    struct MA {
        a: u64,
    }

    #[columnar(ser)]
    struct MB {
        data: Vec<MA>,
    }
    let init_b = B {
        data: vec![A { a: 1 }, A { a: 2 }],
    };
    let init_mb = MB {
        data: vec![MA { a: 1 }, MA { a: 2 }],
    };
    let columnar_bytes = to_vec(&init_b).unwrap();
    let m_bytes = to_vec(&init_mb).unwrap();
    assert_ne!(columnar_bytes, m_bytes);
}

#[test]
fn row_optional() {
    #[columnar(vec, map, ser, de)]
    #[derive(Clone, Debug, PartialEq)]
    struct A {
        #[columnar(strategy = "DeltaRle")]
        a: u64,
        #[columnar(strategy = "BoolRle", optional, index = 0)]
        b: bool,
        #[columnar(optional, index = 1)]
        c: f32,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B {
        #[columnar(class = "vec")]
        vec: Vec<A>,
        #[columnar(class = "map")]
        map: BTreeMap<u8, A>,
    }
    #[columnar(vec, map, ser, de)]
    #[derive(Clone, Debug, PartialEq)]
    struct NA {
        #[columnar(strategy = "DeltaRle")]
        a: u64,
        #[columnar(optional, index = 1)]
        c: f32,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct NB {
        #[columnar(class = "vec")]
        vec: Vec<NA>,
        #[columnar(class = "map")]
        map: HashMap<u8, NA>,
    }
    let map = vec![
        (
            1,
            A {
                a: 1,
                b: true,
                c: 0.1,
            },
        ),
        (
            2,
            A {
                a: 2,
                b: false,
                c: 0.2,
            },
        ),
    ]
    .into_iter()
    .collect();
    let vec = vec![
        A {
            a: 1,
            b: true,
            c: 0.1,
        },
        A {
            a: 2,
            b: false,
            c: 0.2,
        },
    ];
    let init_b = B { vec, map };
    let bytes = to_vec(&init_b).unwrap();
    let nb = NB {
        vec: vec![NA { a: 1, c: 0.1 }, NA { a: 2, c: 0.2 }],
        map: vec![(1, NA { a: 1, c: 0.1 }), (2, NA { a: 2, c: 0.2 })]
            .into_iter()
            .collect(),
    };
    let read_nb: NB = from_bytes(&bytes).unwrap();
    assert_eq!(nb, read_nb);
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn row_generics() {
    #[columnar(vec, ser, de)]
    #[derive(Clone, Debug, PartialEq)]
    struct A<P: Rleable, Q: DeltaRleable> {
        a: u64,
        #[columnar(strategy = "Rle")]
        p: P,
        #[columnar(strategy = "DeltaRle")]
        q: Q,
    }
    #[columnar(ser, de)]
    #[derive(Debug, PartialEq)]
    struct B<P: Rleable, Q: DeltaRleable> {
        #[columnar(class = "vec")]
        data: Vec<A<P, Q>>,
    }
    let init_b = B {
        data: vec![
            A {
                a: 1,
                p: 2u8,
                q: 1u8,
            },
            A { a: 2, p: 2, q: 2 },
        ],
    };
    let bytes = to_vec(&init_b).unwrap();
    let b: B<u8, u8> = from_bytes(&bytes).unwrap();
    assert_eq!(init_b, b);
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn nested() {
    #[columnar(vec, map, ser, de)]
    #[derive(Debug, Clone, PartialEq)]
    struct A {
        a: u64,
    }
    #[columnar(vec, map, ser, de)]
    #[derive(Debug, Clone, PartialEq)]
    struct B {
        #[columnar(class = "vec")]
        vec: Vec<A>,
        #[columnar(class = "map")]
        map: BTreeMap<u8, A>,
        #[columnar(strategy = "BoolRle")]
        b: bool,
    }

    #[columnar(vec, map, ser, de)]
    #[derive(Debug, PartialEq)]
    struct C {
        #[columnar(class = "vec")]
        vec: Vec<B>,
        #[columnar(class = "map")]
        map: BTreeMap<u8, B>,
    }
    let a = A { a: 1 };
    let b = B {
        vec: vec![a.clone(), a.clone()],
        map: vec![(1, a.clone()), (2, a)].into_iter().collect(),
        b: true,
    };
    let c = C {
        vec: vec![b.clone(), b.clone()],
        map: vec![(1, b.clone()), (2, b)].into_iter().collect(),
    };
    let bytes = to_vec(&c).unwrap();
    let read_c = from_bytes(&bytes).unwrap();
    assert_eq!(c, read_c);
    insta::assert_yaml_snapshot!(bytes);
}

#[test]
fn delta_of_delta() {
    #[columnar(vec, ser, de)]
    #[derive(Debug, Clone, PartialEq)]
    struct A {
        #[columnar(strategy = "DeltaOfDelta")]
        a: i64,
    }

    #[columnar(ser, de)]
    #[derive(Debug, Clone, PartialEq)]
    struct Table {
        #[columnar(class = "vec")]
        data: Vec<A>,
    }

    let table = Table {
        data: vec![A { a: 1 }, A { a: 2 }, A { a: 3 }],
    };
    let bytes = to_vec(&table).unwrap();
    let read_table = from_bytes(&bytes).unwrap();
    assert_eq!(table, read_table);
    insta::assert_yaml_snapshot!(bytes);
}
