use std::fmt::Display;

#[derive(Debug)]
pub struct AnalyzeResult {
    pub field_name: String,
    pub binary_size: usize,
}

pub struct AnalyzeResults(pub Vec<AnalyzeResult>);

impl Display for AnalyzeResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "`Analyze Field Size` is a simple feature that assists in analyzing the size of each field after encoding them into binary. Please use only in a **DEBUG** environment. And as its results may not be precise.")?;
        writeln!(f, "Binary data size of each field:")?;
        for result in &self.0 {
            writeln!(f, "{}: {} bytes", result.field_name, result.binary_size)?;
        }
        Ok(())
    }
}

impl From<Vec<AnalyzeResult>> for AnalyzeResults {
    fn from(v: Vec<AnalyzeResult>) -> Self {
        Self(v)
    }
}

pub trait FieldAnalyze: Clone {
    fn analyze(&self) -> AnalyzeResults;
}
