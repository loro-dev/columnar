use std::{borrow::Cow, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use columnar::{
    columnar_decode, columnar_encode, ser::ColumnarSerializer, CellData, ColumnAttr,
    ColumnOriented, ColumnarError, Columns, Row, Strategy,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
// #[derive(Row)]
struct Data {
    // #[columnar(strategy = "RLE")]
    id: u64,
    name: String,
    age: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SubData {
    id: u8,
    map: HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct HasSubData {
    id: u64,
    name: String,
    age: u32,
    sub: SubData,
    list: Vec<SubData>,
}

impl Row for Data {
    fn get_attrs() -> Vec<ColumnAttr> {
        vec![
            ColumnAttr {
                index: 1,
                strategy: None,
            },
            ColumnAttr {
                index: 2,
                strategy: None,
            },
            ColumnAttr {
                index: 3,
                strategy: None,
            },
        ]
    }

    fn get_cells_data<'a: 'c, 'c>(&'a self) -> Vec<CellData<'c>> {
        vec![
            CellData::U64(self.id),
            CellData::String(Cow::Borrowed(&self.name)),
            CellData::U64(self.age as u64),
        ]
    }

    fn from_cells_data(cells_data: Vec<CellData>) -> Result<Self, ColumnarError> {
        let mut cells_data = cells_data.into_iter();
        let id: u64 = cells_data.next().unwrap().try_into()?;
        let string: String = cells_data.next().unwrap().try_into()?;
        let age: u64 = cells_data.next().unwrap().try_into()?;
        Ok(Data {
            id,
            name: string,
            age: age as u32,
        })
    }
}

impl Row for HasSubData {
    fn get_attrs() -> Vec<ColumnAttr> {
        vec![
            ColumnAttr {
                index: 1,
                strategy: Some(Strategy::Rle),
            },
            ColumnAttr {
                index: 2,
                strategy: None,
            },
            ColumnAttr {
                index: 3,
                strategy: None,
            },
            ColumnAttr {
                index: 4,
                strategy: None,
            },
            ColumnAttr {
                index: 5,
                strategy: None,
            },
        ]
    }

    fn get_cells_data<'a: 'c, 'c>(&'a self) -> Vec<CellData<'c>> {
        vec![
            CellData::U64(self.id),
            CellData::String(Cow::Borrowed(&self.name)),
            CellData::U64(self.age as u64),
            CellData::Bytes(Cow::Owned(columnar_encode(&self.sub))),
            CellData::Columns(self.list.get_columns()),
        ]
    }

    fn from_cells_data(cells_data: Vec<CellData>) -> Result<Self, ColumnarError> {
        let mut cells_data = cells_data.into_iter();
        let id = cells_data.next().unwrap().try_into()?;
        let name = cells_data.next().unwrap().try_into()?;
        let age: u64 = cells_data.next().unwrap().try_into()?;
        let sub: Vec<u8> = cells_data.next().unwrap().try_into()?;
        let sub = columnar_decode::<SubData>(&sub);
        let list: Columns = cells_data.next().unwrap().try_into()?;
        let list = Vec::<SubData>::from_columns(list)?;
        let data = Self {
            id,
            name,
            age: age as u32,
            sub,
            list,
        };
        Ok(data)
    }
}

impl Row for SubData {
    fn get_attrs() -> Vec<ColumnAttr> {
        vec![
            ColumnAttr {
                index: 1,
                strategy: Some(Strategy::Rle),
            },
            // ColumnAttr {
            //     index: 2,
            //     strategies: vec![Strategy::Plain],
            // },
        ]
    }

    fn get_cells_data<'a: 'c, 'c>(&'a self) -> Vec<CellData<'c>> {
        vec![
            CellData::U64(self.id as u64),
            // CellData::Map(
            //     self.map
            //         .iter()
            //         .map(|(k, v)| (Cow::Borrowed(k), CellData::I64(*v as i64)))
            //         .collect(),
            // ),
        ]
    }

    fn from_cells_data(cells_data: Vec<CellData>) -> Result<Self, ColumnarError> {
        let first = cells_data[0].clone();
        let id: u64 = first.try_into()?;

        Ok(Self {
            id: id as u8,
            map: HashMap::new(),
        })
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Store {
    #[serde_as(as = "Columns")]
    pub a: Vec<Data>,
    pub b: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct StoreWithSubData {
    pub a: Vec<HasSubData>,
    pub b: String,
}

#[test]
fn test_cell_data() {}

#[test]
fn test_serializer() {
    let store = Store {
        a: vec![
            Data {
                id: 2,
                name: "a".to_string(),
                age: 1000,
            },
            Data {
                id: 2,
                name: "b".to_string(),
                age: 2,
            },
            Data {
                id: 2,
                name: "c".to_string(),
                age: 3,
            },
        ],
        b: "b".to_string(),
    };
    let mut columnar = ColumnarSerializer::new();
    store.serialize(&mut columnar).unwrap();
    println!("{:?}", columnar.to_bytes());
}

#[test]
fn test_sub_data() {
    let store = StoreWithSubData {
        a: vec![HasSubData {
            id: 10,
            name: "a".to_string(),
            age: 20,
            sub: SubData {
                id: 30,
                map: HashMap::new(),
            },
            list: vec![
                SubData {
                    id: 40,
                    map: HashMap::new(),
                },
                SubData {
                    id: 50,
                    map: HashMap::new(),
                },
            ],
        }],
        b: "b".to_string(),
    };
    let bytes = columnar_encode(store);
    println!("{:?}", bytes);
}

#[test]
fn test_decode() {
    let store = Store {
        a: vec![
            Data {
                id: 2,
                name: "a".to_string(),
                age: 10,
            },
            Data {
                id: 2,
                name: "b".to_string(),
                age: 20,
            },
            Data {
                id: 2,
                name: "c".to_string(),
                age: 30,
            },
        ],
        b: "b".to_string(),
    };
    let bytes = columnar_encode(&store);
    let decode_store: Store = columnar_decode(&bytes);
    assert_eq!(store, decode_store);
}

#[test]
fn test_decode_nested() {
    let store = StoreWithSubData {
        a: vec![HasSubData {
            id: 10,
            name: "a".to_string(),
            age: 20,
            sub: SubData {
                id: 30,
                map: HashMap::new(),
            },
            list: vec![
                SubData {
                    id: 40,
                    map: HashMap::new(),
                },
                SubData {
                    id: 50,
                    map: HashMap::new(),
                },
            ],
        }],
        b: "b".to_string(),
    };
    let bytes = columnar_encode(&store);
    println!("{:?}", bytes);
    let decode_store: StoreWithSubData = columnar_decode(&bytes);
    assert_eq!(store, decode_store);
}
