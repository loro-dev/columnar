use std::io::Read;

use crate::ColumnarError;
use flate2::bufread::DeflateDecoder;
use flate2::bufread::DeflateEncoder;
use flate2::Compression;

#[derive(Debug, Clone, PartialEq)]
pub struct CompressConfig {
    pub(crate) threshold: usize,
    pub(crate) compression: Compression,
}

const DEFAULT_COMPRESS_THRESHOLD: usize = 256;

impl Default for CompressConfig {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_COMPRESS_THRESHOLD,
            compression: Compression::default(),
        }
    }
}

impl CompressConfig {
    pub fn from_level(threshold: usize, level: u32) -> Self {
        Self {
            threshold,
            compression: Compression::new(level),
        }
    }

    pub fn from_method(threshold: usize, method: String) -> Self {
        match method.as_str() {
            "fast" => Self {
                threshold,
                compression: Compression::fast(),
            },
            "best" => Self {
                threshold,
                compression: Compression::best(),
            },
            _ => Self {
                threshold,
                compression: Compression::default(),
            },
        }
    }
}

pub(crate) fn compress(input: &[u8], cfg: &CompressConfig) -> Result<Vec<u8>, ColumnarError> {
    let mut output = Vec::new();
    let mut encoder = DeflateEncoder::new(input, cfg.compression);
    encoder.read_to_end(&mut output)?;
    Ok(output)
}

pub(crate) fn decompress<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>, ColumnarError> {
    let mut output = Vec::new();
    let mut decoder = DeflateDecoder::new(input.as_ref());
    decoder.read_to_end(&mut output)?;
    Ok(output)
}
