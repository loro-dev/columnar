use std::borrow::Cow;

use columnar_serde::{Row, ColumnAttr, ColumnData, Strategy, ColumnOriented};

extern crate columnar_serde;

struct Data{
    // #[columnar(strategy = "RLE")]
    id: u64,
    name: String,
    age: u32,
}

impl Row for Data{
    fn get_attrs() -> Vec<ColumnAttr> {
        vec![ColumnAttr{index: 0, strategy: Strategy::RLE}, ColumnAttr{index: 1, strategy: Strategy::Plain}, ColumnAttr{index: 2, strategy: Strategy::Plain}]
    }

    fn get_columns_data<'a: 'c, 'c>(&'a self) -> Vec<ColumnData<'c>>{
        vec![ColumnData::U64(self.id), ColumnData::String(Cow::Borrowed(&self.name)), ColumnData::U32(self.age)]
    }
}

struct Store{
    pub a: Vec<Data>,
    pub b: String
}

fn main(){
    let store = Store{
        a: vec![Data{id: 1, name: "a".to_string(), age: 10}, Data{id: 2, name: "b".to_string(), age: 20}],
        b: "b".to_string()
    };
    let columns = store.a.get_columns();
    println!("{:?}", columns);
}