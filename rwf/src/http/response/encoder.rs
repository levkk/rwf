use super::Error;
use brotli::CompressorWriter;
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use std::io::Write;

pub enum EncodingAlgorithm {
    Gzip,
    Deflate,
    Brotli,
}

pub(crate) enum Encoder {
    Gzip(GzEncoder<Vec<u8>>),
    Deflate(DeflateEncoder<Vec<u8>>),
    Brotli(CompressorWriter<Vec<u8>>),
}

impl Encoder {
    pub fn encoder(algorithm: EncodingAlgorithm) -> Encoder {
        match algorithm {
            EncodingAlgorithm::Gzip => {
                Encoder::Gzip(GzEncoder::new(Vec::new(), Compression::default()))
            }
            EncodingAlgorithm::Deflate => {
                Encoder::Deflate(DeflateEncoder::new(Vec::new(), Compression::default()))
            }
            EncodingAlgorithm::Brotli => {
                Encoder::Brotli(CompressorWriter::new(Vec::new(), 32 * 1024, 3, 22))
            }
        }
    }

    pub fn encode(self, original_body: &[u8]) -> Result<Vec<u8>, Error> {
        match self {
            Encoder::Gzip(mut gzip) => {
                gzip.write_all(&original_body)?;
                Ok(gzip.finish()?)
            }
            Encoder::Deflate(mut deflate) => {
                deflate.write_all(&original_body)?;
                Ok(deflate.finish()?)
            }
            Encoder::Brotli(mut brotli) => {
                brotli.write_all(original_body)?;
                brotli.flush()?;
                Ok(brotli.into_inner())
            }
        }
    }
}
