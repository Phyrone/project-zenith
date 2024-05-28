use error_stack::{FutureExt, ResultExt};
use integer_encoding::{VarIntAsyncReader, VarIntAsyncWriter, VarIntWriter};
use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};





mod test {
    use std::io::Read;

    use bytes::Buf;
    use integer_encoding::{VarIntReader, VarIntWriter};
    use prost::encoding::{DecodeContext, WireType};
    use prost::Message;

    use crate::proto::common::Position;
    use crate::proto::packet::play::client::{PacketPlayClient, PacketPlayClientMove};
    use crate::proto::packet::play::client::packet_play_client::Payload;
    use crate::proto::service::lobby::server::service_lobby_serverlist_client::ServiceLobbyServerlistClient;
    use crate::proto::service::lobby::server::service_lobby_serverlist_server::ServiceLobbyServerlistServer;

    #[tokio::test]
    async fn test_encode() {
        ServiceLobbyServerlistClient::new()
        ServiceLobbyServerlistServer::new()
        
    }

    #[test]
    fn test1() {
        let packet_play_client = PacketPlayClient {
            payload: Some(Payload::Move(PacketPlayClientMove {
                new_position: Some(Position {
                    ..Default::default()
                }),
                ..Default::default()
            })),
        };
        let bytes = packet_play_client.encode_to_vec();
        println!("encoded: {} ({})", hex::encode(&bytes), bytes.len());

        let mut reader = bytes.reader();
        let tag = reader.read_varint::<u64>()
            .expect("Failed to read tag");
        let wire_type = WireType::try_from(tag & 0b111)
            .expect("solar flare?");
        let field_number = tag >> 3;
        assert_eq!(field_number, 257);
        assert_eq!(wire_type, WireType::LengthDelimited);
        let expected_length = reader.read_varint::<u64>()
            .expect("Failed to read length");
        let mut buffer = Vec::with_capacity(expected_length as usize);
        reader.read_to_end(&mut buffer)
            .expect("Failed to read buffer");
        assert_eq!(buffer.len(), expected_length as usize);

        let mut prepend_length = Vec::with_capacity(prost::encoding::encoded_len_varint(expected_length));
        prepend_length.write_varint(expected_length)
            .expect("Failed to write varint");
        buffer.splice(0..0, prepend_length);

        let context = DecodeContext::default();
        let mut build_packet = PacketPlayClient::default();

        build_packet.merge_field(field_number as u32, wire_type, &mut buffer.as_slice(), context.clone())
            .expect("Failed to merge field");
        println!("{:#?}", build_packet);
    }
}
