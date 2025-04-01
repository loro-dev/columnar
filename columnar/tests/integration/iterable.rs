use serde_columnar::columnar;

#[test]
fn row_iterable_strategy() {
    // #[columnar(iterable)]
    struct A {
        // #[columnar(strategy = "Rle")]
        _rle: String,
        // #[columnar(strategy = "DeltaRle")]
        _delta_rle: u32,
        // #[columnar(strategy = "BoolRle")]
        _bool_rle: bool,
        // #[columnar(strategy = "DeltaOfDelta")]
        _delta_of_delta: u32,
    }
    let _ = IterableA {
        _rle: AnyRleIter::new(&[]),
        _delta_rle: DeltaRleIter::new(&[]),
        _bool_rle: BoolRleIter::new(&[]),
        _delta_of_delta: DeltaOfDeltaIter::new(&[]),
    };
    use ::serde_columnar::iterable::*;
    #[derive(::serde_columnar::__private_consume_columnar_attributes)]
    struct IterableA<'__iter> {
        _rle: AnyRleIter<'__iter, String>,
        _delta_rle: DeltaRleIter<'__iter, u32>,
        _bool_rle: BoolRleIter<'__iter>,
        _delta_of_delta: DeltaOfDeltaIter<'__iter, u32>,
    }
    const _: () = {
        use ::serde::de::Error as DeError;
        use ::serde::de::Visitor;
        use ::std::collections::HashMap;
        impl<'de: '__iter, '__iter> ::serde::de::Deserialize<'de> for IterableA<'__iter> {
            fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
            where
                __D: serde::Deserializer<'de>,
            {
                struct DeVisitor<'de: '__iter, '__iter> {
                    marker: std::marker::PhantomData<IterableA<'__iter>>,
                    lifetime: std::marker::PhantomData<&'de ()>,
                };
                impl<'de: '__iter, '__iter> Visitor<'de> for DeVisitor<'de, '__iter> {
                    type Value = IterableA<'__iter>;
                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("a sequence")
                    }
                    fn visit_seq<__A>(self, mut seq: __A) -> Result<Self::Value, __A::Error>
                    where
                        __A: serde::de::SeqAccess<'de>,
                    {
                        let _rle = seq
                            .next_element()?
                            .ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
                        let _delta_rle = seq
                            .next_element()?
                            .ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
                        let _bool_rle = seq
                            .next_element()?
                            .ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
                        let _delta_of_delta = seq
                            .next_element()?
                            .ok_or_else(|| __A::Error::custom("DeserializeUnexpectedEnd"))?;
                        Ok(IterableA {
                            _rle,
                            _delta_rle,
                            _bool_rle,
                            _delta_of_delta,
                        })
                    }
                }
                deserializer.deserialize_seq(DeVisitor {
                    marker: Default::default(),
                    lifetime: Default::default(),
                })
            }
        }
    };
    impl<'__iter> Iterator for IterableA<'__iter> {
        type Item = ::std::result::Result<A, ::serde_columnar::ColumnarError>;
        fn next(&mut self) -> Option<Self::Item> {
            let _rle = match self._rle.next().transpose() {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let _delta_rle = match self._delta_rle.next().transpose() {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let _bool_rle = match self._bool_rle.next().transpose() {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let _delta_of_delta = match self._delta_of_delta.next().transpose() {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let (Some(_rle), Some(_delta_rle), Some(_bool_rle), Some(_delta_of_delta)) =
                (_rle, _delta_rle, _bool_rle, _delta_of_delta)
            {
                Some(::std::result::Result::Ok(A {
                    _rle,
                    _delta_rle,
                    _bool_rle,
                    _delta_of_delta,
                }))
            } else {
                None
            }
        }
    }
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
