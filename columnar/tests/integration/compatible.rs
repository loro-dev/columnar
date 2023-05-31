#[cfg(test)]
pub mod table {
    use std::collections::HashMap;

    use serde::de::Error as DeError;
    use serde::de::Visitor;
    use serde::ser::SerializeSeq;
    use serde::ser::{Error as SerError, SerializeTuple};
    use serde::{Deserialize, Serialize};

    // #[columnar(compatible)]
    #[derive(Debug, Clone, PartialEq)]
    struct VecStore {
        // #[columnar(index=0)]
        data: Vec<u64>,
        id: u64,
    }

    #[derive(Debug, Default, Clone, PartialEq)]
    struct MoreVecStore {
        data: Vec<u64>,
        id: u64,
        // #[columnar(optional)]
        id2: Option<u64>,
    }

    impl Serialize for VecStore {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut tuple = serializer.serialize_seq(Some(2))?;
            tuple.serialize_element(&self.data)?;
            tuple.serialize_element(&self.id)?;
            tuple.end()
        }
    }

    impl Serialize for MoreVecStore {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut tuple = serializer.serialize_seq(Some(3))?;
            tuple.serialize_element(&self.data)?;
            tuple.serialize_element(&self.id)?;
            let index_0_bytes = postcard::to_allocvec(&self.id2).map_err(S::Error::custom)?;
            tuple.serialize_element(&(0u8, index_0_bytes))?;
            tuple.end()
        }
    }

    impl<'de> Deserialize<'de> for VecStore {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct VecStoreVisitor;
            impl<'de> Visitor<'de> for VecStoreVisitor {
                type Value = VecStore;
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("VecStore")
                }
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let data = seq
                        .next_element()?
                        .ok_or_else(|| A::Error::custom("data"))?;
                    let id = seq.next_element()?.ok_or_else(|| A::Error::custom("id"))?;
                    while let Ok(Some((_index, _bytes))) = seq.next_element::<(u8, Vec<u8>)>() {
                        // ignore
                    }
                    Ok(VecStore { data, id })
                }
            }
            deserializer.deserialize_seq(VecStoreVisitor)
        }
    }

    impl<'de> Deserialize<'de> for MoreVecStore {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct MoreVecStoreVisitor;
            impl<'de> Visitor<'de> for MoreVecStoreVisitor {
                type Value = MoreVecStore;
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("MoreVecStore")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let data = seq
                        .next_element()?
                        .ok_or_else(|| A::Error::custom("data"))?;
                    let id = seq.next_element()?.ok_or_else(|| A::Error::custom("id"))?;
                    // optional
                    let mut mapping = HashMap::new();

                    while let Ok(Some((index, bytes))) = seq.next_element::<(u8, Vec<u8>)>() {
                        // ignore
                        mapping.insert(index, bytes);
                    }
                    let id2 = if let Some(bytes) = mapping.remove(&0) {
                        Some(postcard::from_bytes(&bytes).map_err(A::Error::custom)?)
                    } else {
                        Default::default()
                    };
                    Ok(MoreVecStore { data, id, id2 })
                }
            }
            deserializer.deserialize_seq(MoreVecStoreVisitor)
        }
    }

    #[test]
    fn add_field() {
        // forward compatible
        let new = MoreVecStore {
            data: vec![1, 2, 3],
            id: 1,
            id2: Some(1),
        };
        let old = VecStore {
            data: vec![1, 2, 3],
            id: 1,
        };
        let bytes = serde_columnar::to_vec(&new).unwrap();
        println!("{:?}", bytes);
        let old_new = serde_columnar::from_bytes::<VecStore>(&bytes).unwrap();
        assert_eq!(old, old_new);

        // backward compatible
        let new = MoreVecStore {
            data: vec![1, 2, 3],
            id: 1,
            id2: None,
        };
        let bytes = serde_columnar::to_vec(&old).unwrap();
        println!("{:?}", bytes);
        let de_new = serde_columnar::from_bytes::<MoreVecStore>(&bytes).unwrap();
        assert_eq!(new, de_new);
    }
}

#[cfg(test)]
pub mod row {
    use serde::de::Visitor;
    use serde::{Deserialize, Serialize};
    use serde_columnar::columnar;
    type ID = u64;

    // #[columnar(vec, ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Data {
        // #[columnar(strategy = "DeltaRle", original_type = "u64")]
        id: ID,
        // #[columnar(strategy = "Rle")]
        name: String,
    }

    const _: () = {
        use serde::ser::SerializeSeq;
        #[automatically_derived]
        impl<IT> ::serde_columnar::RowSer<IT> for Data
        where
            for<'c> &'c IT: IntoIterator<Item = &'c Self>,
        {
            const FIELD_NUM: usize = 2usize;
            fn serialize_columns<S>(rows: &IT, ser: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                let column0 = rows
                    .into_iter()
                    .map(|row| row.id)
                    .collect::<::std::vec::Vec<_>>();
                let column0 = ::serde_columnar::DeltaRleColumn::<ID>::new(
                    column0,
                    ::serde_columnar::ColumnAttr { index: 0usize },
                );
                let column1 = rows
                    .into_iter()
                    .map(|row| std::borrow::Cow::Borrowed(&row.name))
                    .collect::<::std::vec::Vec<_>>();
                let column1 = ::serde_columnar::RleColumn::<std::borrow::Cow<String>>::new(
                    column1,
                    ::serde_columnar::ColumnAttr { index: 1usize },
                );
                let mut seq_encoder = ser.serialize_seq(Some(2usize))?;
                seq_encoder.serialize_element(&column0)?;
                seq_encoder.serialize_element(&column1)?;
                seq_encoder.end()
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl<'de, IT> ::serde_columnar::RowDe<'de, IT> for Data
        where
            IT: FromIterator<Self> + Clone,
        {
            const FIELD_NUM: usize = 2usize;
            fn deserialize_columns<D>(de: D) -> Result<IT, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct DataVisitor<IT>(::std::marker::PhantomData<IT>);
                impl<'de, IT> Visitor<'de> for DataVisitor<IT>
                where
                    IT: FromIterator<Data>,
                {
                    type Value = IT;
                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("Data")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let column0: ::serde_columnar::DeltaRleColumn<ID> =
                            seq.next_element()?.unwrap();
                        let column1: ::serde_columnar::RleColumn<::std::borrow::Cow<String>> =
                            seq.next_element()?.unwrap();
                        // maybe optional
                        // match index {
                        //     Ok(_index) => {
                        //         let _: Vec<u8> = seq.next_element()?.unwrap();
                        //     }
                        //     Err(_) => {}
                        // };
                        while let Ok(Some(_index)) = seq.next_element::<u8>() {
                            let _: Vec<u8> = seq.next_element()?.unwrap();
                        }
                        let ans = ::serde_columnar::izip!(
                            column0.data.into_iter(),
                            column1.data.into_iter()
                        )
                        .map(|(id, name)| Data {
                            id: id,
                            name: name.into_owned(),
                        })
                        .collect();
                        Ok(ans)
                    }
                }
                let visitor = DataVisitor(::std::marker::PhantomData);
                de.deserialize_seq(visitor)
            }
        }
    };

    // #[columnar(vec, ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NewData {
        // #[columnar(strategy = "DeltaRle", original_type = "u64")]
        id: ID,
        // #[columnar(strategy = "Rle")]
        name: String,
        // #[columnar(optional)]
        id2: Option<u64>,
    }

    const _: () = {
        use serde::ser::SerializeSeq;
        #[automatically_derived]
        impl<IT> ::serde_columnar::RowSer<IT> for NewData
        where
            for<'c> &'c IT: IntoIterator<Item = &'c Self>,
        {
            const FIELD_NUM: usize = 3usize;
            fn serialize_columns<S>(rows: &IT, ser: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                let column0 = rows
                    .into_iter()
                    .map(|row| row.id)
                    .collect::<::std::vec::Vec<_>>();
                let column0 = ::serde_columnar::DeltaRleColumn::<ID>::new(
                    column0,
                    ::serde_columnar::ColumnAttr { index: 0usize },
                );
                let column1 = rows
                    .into_iter()
                    .map(|row| std::borrow::Cow::Borrowed(&row.name))
                    .collect::<::std::vec::Vec<_>>();
                let column1 = ::serde_columnar::RleColumn::<std::borrow::Cow<String>>::new(
                    column1,
                    ::serde_columnar::ColumnAttr { index: 1usize },
                );

                // optional

                let column2 = rows
                    .into_iter()
                    .map(|row| std::borrow::Cow::Borrowed(&row.id2))
                    .collect::<::std::vec::Vec<_>>();
                let mut seq_encoder = ser.serialize_seq(Some(4usize))?;
                seq_encoder.serialize_element(&column0)?;
                seq_encoder.serialize_element(&column1)?;
                seq_encoder.serialize_element(&0u8)?;
                // let col2_bytes = postcard::to_allocvec(&column2).map_err(S::Error::custom)?;
                seq_encoder.serialize_element(&column2)?;
                seq_encoder.end()
            }
        }
    };
    const _: () = {
        use serde::ser::SerializeTuple;
        #[automatically_derived]
        impl<'de, IT> ::serde_columnar::RowDe<'de, IT> for NewData
        where
            IT: FromIterator<Self> + Clone,
        {
            const FIELD_NUM: usize = 3usize;
            fn deserialize_columns<D>(de: D) -> Result<IT, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct NewDataVisitor<IT>(::std::marker::PhantomData<IT>);
                impl<'de, IT> Visitor<'de> for NewDataVisitor<IT>
                where
                    IT: FromIterator<NewData>,
                {
                    type Value = IT;
                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("NewData")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let column0: ::serde_columnar::DeltaRleColumn<ID> =
                            seq.next_element()?.unwrap();
                        let column1: ::serde_columnar::RleColumn<::std::borrow::Cow<String>> =
                            seq.next_element()?.unwrap();
                        // optional
                        let index = seq.next_element::<u8>();
                        let column2: Option<::std::vec::Vec<::std::borrow::Cow<Option<u64>>>> =
                            match index {
                                Ok(Some(_index2)) => Ok(Some(seq.next_element()?.unwrap())),
                                Ok(None) => Ok(None),
                                Err(e) => {
                                    if e.to_string() == "Hit the end of buffer, expected more data"
                                    {
                                        Ok(None)
                                    } else {
                                        Err(e)
                                    }
                                }
                            }?;
                        if let Some(column2) = column2 {
                            let ans = ::serde_columnar::izip!(
                                column0.data.into_iter(),
                                column1.data.into_iter(),
                                column2.into_iter()
                            )
                            .map(|(id, name, id2)| NewData {
                                id: id,
                                name: name.into_owned(),
                                id2: id2.into_owned(),
                            })
                            .collect();
                            Ok(ans)
                        } else {
                            let ans = ::serde_columnar::izip!(
                                column0.data.into_iter(),
                                column1.data.into_iter()
                            )
                            .map(|(id, name)| NewData {
                                id: id,
                                name: name.into_owned(),
                                id2: None,
                            })
                            .collect();
                            Ok(ans)
                        }
                    }
                }
                let visitor = NewDataVisitor(::std::marker::PhantomData);
                de.deserialize_seq(visitor)
            }
        }
    };

    #[columnar(ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct VecStore {
        #[columnar(class = "vec")]
        pub data: Vec<Data>,
        pub id: u32,
    }

    #[columnar(ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NewVecStore {
        #[columnar(class = "vec")]
        pub data: Vec<NewData>,
        pub id: u32,
    }

    #[test]
    fn add_field_forward() {
        // forward compatible
        let store = VecStore {
            data: vec![
                Data {
                    id: 1,
                    name: "a".to_string(),
                },
                Data {
                    id: 2,
                    name: "b".to_string(),
                },
            ],
            id: 1,
        };
        let new_store = NewVecStore {
            data: vec![
                NewData {
                    id: 1,
                    name: "a".to_string(),
                    id2: Some(1),
                },
                NewData {
                    id: 2,
                    name: "b".to_string(),
                    id2: Some(2),
                },
            ],
            id: 1,
        };
        let new_bytes = serde_columnar::to_vec(&new_store).unwrap();
        let old_new = serde_columnar::from_bytes::<VecStore>(&new_bytes).unwrap();
        assert_eq!(store, old_new);
    }

    #[test]
    fn add_field_backward() {
        // backward compatible
        let new_store = NewVecStore {
            data: vec![
                NewData {
                    id: 1,
                    name: "a".to_string(),
                    id2: None,
                },
                NewData {
                    id: 2,
                    name: "b".to_string(),
                    id2: None,
                },
            ],
            id: 1,
        };
        let store = VecStore {
            data: vec![
                Data {
                    id: 1,
                    name: "a".to_string(),
                },
                Data {
                    id: 2,
                    name: "b".to_string(),
                },
            ],
            id: 1,
        };
        let old_bytes = serde_columnar::to_vec(&store).unwrap();
        let new_old = serde_columnar::from_bytes::<NewVecStore>(&old_bytes).unwrap();
        assert_eq!(new_store, new_old);
    }
}
