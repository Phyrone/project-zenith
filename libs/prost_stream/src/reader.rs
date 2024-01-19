use bytes::BytesMut;

pub struct ProtoReader<R> {
    reader: R,
    buffer: BytesMut,
}
