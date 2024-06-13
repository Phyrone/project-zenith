use integer_encoding::{VarIntAsyncReader, VarIntAsyncWriter, VarIntWriter};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::proto::common::CompressionAlgorithm;
use crate::COMPRESSION_THRESHOLD;

mod test {}

#[derive(Debug, Error)]
pub enum WriteMessageError {
    #[error("io error: {0}")]
    Io(std::io::Error),
}

pub trait WriteMessageExt {
    async fn write_message<M>(
        &mut self,
        message: M,
        compression: CompressionAlgorithm,
    ) -> Result<(), WriteMessageError>
    where
        M: prost::Message + Default;
}

#[derive(Debug, Error)]
pub enum ReadMessageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("decode error: {0}")]
    Prost(#[from] prost::DecodeError),
    #[error("message size {0} exceeds limit {1}")]
    LimitExceeded(usize, usize),
    #[error("lz4 decompression error: {0}")]
    Lz4DecompressError(#[from] lz4_flex::block::DecompressError),
}

pub trait ReadMessageExt {
    async fn read_message<M>(&mut self, size_limit: Option<usize>) -> Result<M, ReadMessageError>
    where
        M: prost::Message + Default;
}

impl<W> WriteMessageExt for W
where
    W: AsyncWrite + Unpin,
{
    async fn write_message<M>(
        &mut self,
        message: M,
        compression: CompressionAlgorithm,
    ) -> Result<(), WriteMessageError>
    where
        M: prost::Message + Default,
    {
        let encoded = message.encode_to_vec();
        let size = message.encoded_len();
        if size < COMPRESSION_THRESHOLD {
            self.write_varint_async(size as u64)
                .await
                .map_err(WriteMessageError::Io)?;
            self.write_all(&encoded)
                .await
                .map_err(WriteMessageError::Io)?;
        } else {
            let payload = match compression {
                CompressionAlgorithm::CompressionNone => encoded,
                CompressionAlgorithm::CompressionLz4 => lz4_flex::compress_prepend_size(&encoded),
            };
            self.write_varint_async(payload.len() as u64)
                .await
                .map_err(WriteMessageError::Io)?;
            self.write_varint_async((compression as i32) as u32)
                .await
                .map_err(WriteMessageError::Io)?;
            self.write_all(&payload)
                .await
                .map_err(WriteMessageError::Io)?;
        }
        Ok(())
    }
}

impl<R> ReadMessageExt for R
where
    R: AsyncRead + Unpin,
{
    async fn read_message<M>(&mut self, size_limit: Option<usize>) -> Result<M, ReadMessageError>
    where
        M: prost::Message + Default,
    {
        let size = self.read_varint_async::<u64>().await?;
        if let Some(size_limit) = size_limit {
            if size as usize > size_limit {
                return Err(ReadMessageError::LimitExceeded(size as usize, size_limit));
            }
        }

        let compression = if size < COMPRESSION_THRESHOLD as u64 {
            CompressionAlgorithm::CompressionNone
        } else {
            let compression = self.read_varint_async::<u32>().await? as i32;
            CompressionAlgorithm::try_from(compression)?
        };

        let mut buffer = vec![0u8; size as usize];
        self.read_exact(&mut buffer).await?;
        let payload = match compression {
            CompressionAlgorithm::CompressionNone => buffer,
            CompressionAlgorithm::CompressionLz4 => lz4_flex::decompress_size_prepended(&buffer)
                .map_err(ReadMessageError::Lz4DecompressError)?,
        };
        M::decode(&payload[..]).map_err(ReadMessageError::Prost)
    }
}
