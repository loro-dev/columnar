pub trait Row: Sized {
    const FIELD_NUM: usize;
    // fn types() -> Vec<TypeId>;
    // fn get_attrs() -> Vec<ColumnAttr>;
    // fn get_values<'a: 'c, 'c>(&'a self) -> Vec<Box<dyn Any>>;
    // fn from_values(cells_data: Vec<Box<dyn Any>>) -> Result<Self, ColumnarError>;
    fn serialize_vec_as_columns<S>(rows: &Vec<Self>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    fn deserialize_columns_to_vec<'de, D>(de: D) -> Result<Vec<Self>, D::Error>
    where
        D: serde::Deserializer<'de>;
}
