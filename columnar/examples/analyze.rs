use serde::Serialize;
use serde_columnar::FieldAnalyze;

#[derive(Debug, Clone, FieldAnalyze, Serialize)]
struct Foo {
    #[analyze]
    a: Vec<u32>,
    #[analyze]
    b: Vec<String>,
}

fn main() {
    let foo = Foo {
        a: vec![1, 2, 3],
        b: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    };
    let result = foo.analyze();
    println!("{}", result);

    serde_columnar::izip!(0..10, 2..12, 3..13);
}
