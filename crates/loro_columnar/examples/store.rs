extern crate columnar;
extern crate loro_columnar;
use columnar::{Row, Columns, Column, ColumnData, ColumnAttr, Strategy, ColumnOriented, Encodable, Encoder};
use loro_columnar::LoroEncoder;

struct Test {
    a: String,
    b: u64,
}

impl Row for Test {}
    

struct TestColumn(Vec<String>, Vec<u64>);

pub struct Store(Vec<Test>);

impl Columns for TestColumn {
    type Row = Test;
    // fn attr_num() -> usize {
    //     2
    // }

    fn column_data(&self) -> Vec<Column> {
        vec![
            Column {
                data: self
                    .0
                    .iter()
                    .map(|x| ColumnData::String(x.clone()))
                    .collect(),
                attr: ColumnAttr {
                    index: 0,
                    strategies: vec![Strategy::Plain],
                },
            },
            Column {
                data: self.1.iter().map(|x| ColumnData::U64(*x)).collect(),
                attr: ColumnAttr {
                    index: 1,
                    strategies: vec![Strategy::ULeb128],
                },
            },
        ]
    }
}

impl ColumnOriented<TestColumn> for Vec<Test> {
    fn get_column_data(&self) -> TestColumn {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for i in self {
            a.push(i.a.clone());
            b.push(i.b);
        }
        TestColumn(a, b)
    }
}

impl Encodable for Store {
    fn encode<E>(&self, encoder: &mut E) -> Result<E::Ok, E::Error>
    where
        E: Encoder,
    {
        encoder.encode_column_oriented(&self.0)
    }
}
extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};
use serde_json::from_str;
fn main() {
    // let store = Store(vec![
    //     Test {
    //         a: "a".to_string(),
    //         b: 1,
    //     },
    //     Test {
    //         a: "b".to_string(),
    //         b: 2,
    //     },
    // ]);
    // let mut encoder = LoroEncoder::new();
    // store.encode(&mut encoder).unwrap();
    // println!("{:?}", encoder.data);
    
    #[derive(Debug, Serialize, Deserialize)]
    struct A{
        a: u32
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct B{
        a: u32,
        b: u64
    }
    println!("{:?}", from_str::<A>("{\"a\": 1, \"b\": 1}").unwrap());
}