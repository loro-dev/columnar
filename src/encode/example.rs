use super::{
    encode::{Encodable, Encoder},
    ColumnAttr, ColumnOriented, Columns, Row, Strategy,
};

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

    fn column_data(&self) -> Vec<crate::Column> {
        vec![
            crate::Column {
                data: self
                    .0
                    .iter()
                    .map(|x| super::ColumnData::String(x.clone()))
                    .collect(),
                attr: ColumnAttr {
                    index: 0,
                    strategies: vec![Strategy::Plain],
                },
            },
            crate::Column {
                data: self.1.iter().map(|x| super::ColumnData::U64(*x)).collect(),
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
