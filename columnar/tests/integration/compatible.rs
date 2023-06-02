#[cfg(test)]
pub mod table {
    use serde_columnar::columnar;

    #[columnar(compatible, ser, de)]
    #[derive(Debug, Clone, PartialEq)]
    struct VecStore {
        data: Vec<u64>,
        #[columnar(optional, index = 0)]
        id: u64,
    }

    #[columnar(compatible, ser, de)]
    #[derive(Debug, Default, Clone, PartialEq)]
    struct MoreVecStore {
        data: Vec<u64>,
        #[columnar(optional, index = 0)]
        id: u64,
        #[columnar(optional, index = 1)]
        id2: Option<u64>,
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
    use serde::{Deserialize, Serialize};
    use serde_columnar::columnar;
    type ID = u64;

    #[columnar(vec, ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Data {
        #[columnar(strategy = "DeltaRle")]
        id: ID,
        #[columnar(strategy = "Rle", optional, index = 0)]
        name: String,
    }

    #[columnar(vec, ser, de)]
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NewData {
        #[columnar(strategy = "DeltaRle")]
        id: ID,
        #[columnar(strategy = "Rle", optional, index = 0)]
        name: String,
        #[columnar(optional, index = 1)]
        id2: Option<u64>,
    }

    #[columnar(compatible, ser, de)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct VecStore {
        #[columnar(class = "vec")]
        pub data: Vec<Data>,
        pub id: u32,
    }

    #[columnar(compatible, ser, de)]
    #[derive(Debug, Clone, PartialEq, Eq)]
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
        let new = serde_columnar::from_bytes::<NewVecStore>(&new_bytes).unwrap();
        assert_eq!(new, new_store);
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
