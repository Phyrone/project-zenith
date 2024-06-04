use error_stack::{FutureExt, ResultExt};
use integer_encoding::{VarIntAsyncReader, VarIntAsyncWriter, VarIntWriter};
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

mod test {}
