use std::sync::{Arc, RwLock};

use error_stack::Report;
use thiserror::Error;

use crate::transport::{TransportContainer, TransportLayer};

pub struct LocalTransportLayer<PacketOut, PacketIn, DatagramOut, DatagramIn> {
    egress: crossbeam::channel::Sender<TransportContainer<PacketOut, DatagramOut>>,
    ingress: crossbeam::channel::Receiver<TransportContainer<PacketIn, DatagramIn>>,
    is_closed: Arc<RwLock<bool>>,
}

#[derive(Debug, Error)]
#[error("Local transport error")]
pub struct LocalTransportError;

impl<PacketOut, PacketIn, DatagramOut, DatagramIn>
TransportLayer<PacketOut, PacketIn, DatagramOut, DatagramIn, LocalTransportError>
for LocalTransportLayer<PacketOut, PacketIn, DatagramOut, DatagramIn>
where
    PacketOut: Send,
    PacketIn: Send,
    DatagramOut: Send,
    DatagramIn: Send,
{
    fn send_packet(&self, packet: PacketOut) -> error_stack::Result<(), LocalTransportError> {
        let container = TransportContainer::Packet(packet);
        self.egress.send(container)
            .map_err(|_| Report::from(LocalTransportError))?;

        Ok(())
    }

    fn send_datagram(&self, datagram: DatagramOut) -> error_stack::Result<(), LocalTransportError> {
        let container = TransportContainer::Datagram(datagram);
        self.egress.send(container)
            .map_err(|_| Report::from(LocalTransportError))?;
        
        Ok(())
    }

    fn drain(
        &mut self,
        into: &mut Vec<TransportContainer<PacketIn, DatagramIn>>,
    ) -> error_stack::Result<(), LocalTransportError> {
        while let Ok(container) = self.ingress.try_recv() {
            into.push(container);
        }
        Ok(())
    }

    fn close(&self) {
        *self.is_closed.write().expect("cannot write lock local agent close") = true;
    }

    fn is_closed(&self) -> bool {
        *self.is_closed.read().expect("cannot read lock local agent close")
    }
}
