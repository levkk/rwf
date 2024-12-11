use brotli::CompressorWriter;
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use std::io::Write;

#[derive(Debug)]
pub enum EncodingAlgorithm {
    Gzip,
    Deflate,
    Brotli,
    Identity,
}

pub(crate) enum Encoder {
    Gzip(GzEncoder<Vec<u8>>),
    Deflate(DeflateEncoder<Vec<u8>>),
    Brotli(CompressorWriter<Vec<u8>>),
    Identity,
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
            EncodingAlgorithm::Identity => Encoder::Identity,
        }
    }

    pub fn encode(self, original_body: &[u8]) -> Result<Vec<u8>, std::io::Error> {
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
            Encoder::Identity => Ok(original_body.to_vec()),
        }
    }
}
