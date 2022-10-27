extern crate columnar_serde;
extern crate serde;
extern crate serde_with;

use std::borrow::Cow;

use columnar_serde::{ColumnAttr, ColumnData, ColumnOriented, Columns, Row, Strategy};
use serde::Serialize;
use serde_with::serde_as;

#[derive(Debug)]
struct Data {
    // #[columnar(strategy = "RLE")]
    id: u64,
    name: String,
    age: u32,
}

impl Row for Data {
    fn get_attrs() -> Vec<ColumnAttr> {
        vec![
            ColumnAttr {
                index: 1,
                strategies: vec![Strategy::Rle],
            },
            ColumnAttr {
                index: 2,
                strategies: vec![Strategy::Plain],
            },
            ColumnAttr {
                index: 3,
                strategies: vec![Strategy::Plain],
            },
        ]
    }

    fn get_columns_data<'a: 'c, 'c>(&'a self) -> Vec<ColumnData<'c>> {
        vec![
            ColumnData::U64(self.id),
            ColumnData::String(Cow::Borrowed(&self.name)),
            ColumnData::U64(self.age as u64),
        ]
    }
}

#[serde_as]
#[derive(Debug, Serialize)]
struct Store {
    #[serde_as(as = "Columns")]
    pub a: Vec<Data>,
    pub b: String,
}

fn main() {
    let store = Store {
        a: vec![
            Data {
                id: 1,
                name: "a".to_string(),
                age: 10,
            },
            Data {
                id: 2,
                name: "b".to_string(),
                age: 20,
            },
        ],
        b: "b".to_string(),
    };
    let columns = store.a.get_columns();
    println!("{:?}", columns);
}
