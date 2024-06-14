use serde_columnar::columnar;

#[test]
fn row_iterable_strategy() {
    #[columnar(iterable)]
    struct A {
        #[columnar(strategy = "Rle")]
        _rle: String,
        #[columnar(strategy = "DeltaRle")]
        _delta_rle: u32,
        #[columnar(strategy = "BoolRle")]
        _bool_rle: bool,
    }
    let _ = IterableA {
        _rle: AnyRleIter::new(&[]),
        _delta_rle: DeltaRleIter::new(&[]),
        _bool_rle: BoolRleIter::new(&[]),
    };
}

#[test]
fn iterable_without_strategy() {
    #[columnar(vec, ser, de, iterable)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Row {
        a: u32,
        b: u8,
    }

    #[columnar(ser, de)]
    #[derive(Debug, PartialEq, Eq)]
    struct ATable {
        #[columnar(class = "vec", iter = "Row")]
        a: Vec<Row>,
    }

    let t = ATable {
        a: vec![
            Row { a: 100, b: 100 },
            Row { a: 101, b: 101 },
            Row { a: 102, b: 102 },
        ],
    };

    let encode = serde_columnar::to_vec(&t).unwrap();
    let decode = serde_columnar::iter_from_bytes::<ATable>(&encode).unwrap();
    let ans: Vec<Row> = decode.a.map(|x| x.unwrap()).collect();
    assert_eq!(t.a, ans);
}
