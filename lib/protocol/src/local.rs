use std::sync::{Arc, RwLock};

use bevy::prelude::Resource;

use crate::proto::play::client::packet::PacketPlayClient;
use crate::proto::play::server::packet::PacketPlayServer;

pub type ClientLocalConnectionAgent = LocalConnectionAgent<PacketPlayClient, PacketPlayServer>;
pub type ServerLocalConnectionAgent = LocalConnectionAgent<PacketPlayServer, PacketPlayClient>;

#[derive(Debug, Resource)]
pub struct LocalConnectionAgent<O, I> {
    egress: crossbeam::channel::Sender<O>,
    ingress: crossbeam::channel::Receiver<I>,
    is_closed: Arc<RwLock<bool>>,
}

pub fn new_local_agent<C, S>() -> (LocalConnectionAgent<C, S>, LocalConnectionAgent<S, C>) {
    let (tx1, rx1) = crossbeam::channel::unbounded();
    let (tx2, rx2) = crossbeam::channel::unbounded();
    let is_closed = Arc::new(RwLock::new(false));
    (
        LocalConnectionAgent {
            egress: tx1,
            ingress: rx2,
            is_closed: is_closed.clone(),
        },
        LocalConnectionAgent {
            egress: tx2,
            ingress: rx1,
            is_closed,
        },
    )
}

impl<O, I> LocalConnectionAgent<O, I> {
    pub fn send(&self, packet: O) -> bool {
        self.egress.send(packet).is_ok()
    }

    pub fn drain(&self, into: &mut Vec<I>) {
        //once the queue is empty, recv will error and we can continue
        while let Ok(packet) = self.ingress.try_recv() {
            into.push(packet);
        }
    }

    pub fn close(&self) {
        *self
            .is_closed
            .write()
            .expect("cannot write lock local agent close") = true;
    }

    pub fn is_closed(&self) -> bool {
        *self
            .is_closed
            .read()
            .expect("cannot read lock local agent close")
    }
}

impl<I, O> Drop for LocalConnectionAgent<I, O> {
    fn drop(&mut self) {
        self.close();
    }
}
