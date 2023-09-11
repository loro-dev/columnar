use serde_columnar::columnar;

#[test]
fn row_iterable_strategy() {
    #[columnar(iterable)]
    struct A {
        #[columnar(strategy = "Rle")]
        _rle: String,
        #[columnar(strategy = "DeltaRle")]
        _delta_rle: u32,
        #[columnar(strategy = "BoolRle")]
        _bool_rle: bool,
    }
    let _ = IterableA {
        _rle: AnyRleIter::new(&[]),
        _delta_rle: DeltaRleIter::new(&[]),
        _bool_rle: BoolRleIter::new(&[]),
    };
}
