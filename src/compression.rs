use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    Gzip,
    Bzip2,
    Xz,
    Zstd,
    Lz4,
    Zip,
    Tar,
}

impl CompressionType {
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "gz" | "gzip" => Some(Self::Gzip),
            "bz2" | "bzip2" => Some(Self::Bzip2),
            "xz" => Some(Self::Xz),
            "zst" | "zstd" => Some(Self::Zstd),
            "lz4" => Some(Self::Lz4),
            "zip" => Some(Self::Zip),
            "tar" => Some(Self::Tar),
            _ => None,
        }
    }
}

pub fn is_compressed(path: &Path) -> bool {
    CompressionType::from_extension(path).is_some()
}

pub fn open_compressed_stream(
    path: &Path,
    compression: CompressionType,
) -> io::Result<Box<dyn Read + Send>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    match compression {
        CompressionType::Gzip => {
            let decoder = flate2::read::GzDecoder::new(reader);
            Ok(Box::new(decoder))
        }
        CompressionType::Bzip2 => {
            let decoder = bzip2::read::BzDecoder::new(reader);
            Ok(Box::new(decoder))
        }
        CompressionType::Xz => {
            let decoder = xz2::read::XzDecoder::new(reader);
            Ok(Box::new(decoder))
        }
        CompressionType::Zstd => {
            let decoder = zstd::stream::read::Decoder::new(reader)?;
            Ok(Box::new(decoder))
        }
        CompressionType::Lz4 => {
            let decoder = lz4_flex::frame::FrameDecoder::new(reader);
            Ok(Box::new(decoder))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "Compression type {:?} is not a stream format or not yet supported",
                compression
            ),
        )),
    }
}
