use std::borrow::Cow;

#[derive(Debug)]
pub enum ColumnData<'c> {
    U64(u64),
    U32(u32),
    String(Cow<'c, str>),
}
