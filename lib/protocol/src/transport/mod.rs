use std::error::Error;

mod local;

#[derive(Debug)]
pub enum TransportContainer<Packet, Datagram> {
    Packet(Packet),
    Datagram(Datagram),
}

pub trait TransportLayer<PacketOut, PacketIn, DatagramOut, DatagramIn, TransportError>
where
    TransportError: Error,
{
    fn send_packet(&self, packet: PacketOut) -> error_stack::Result<(), TransportError>;
    fn send_datagram(&self, datagram: DatagramOut) -> error_stack::Result<(), TransportError>;
    fn drain(
        &mut self,
        into: &mut Vec<TransportContainer<PacketIn, DatagramIn>>,
    ) -> error_stack::Result<(), TransportError>;
    
    fn close(&self);
    
    fn is_closed(&self) -> bool;
}
