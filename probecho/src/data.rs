use iceoryx2::prelude::*;

/// A ping-pong payload.
#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("PingPongData")]
#[repr(C)]
pub enum PingPongPayload {
    Ping,
    Pong,
}

/// A ping-pong payload header.
#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("PingPongData")]
#[repr(C)]
pub struct PingPongHeader {
    pub msg_id: u64,
    pub src_id: usize,
    pub dst_id: usize,
    pub answering_to: Option<u64>,
}
