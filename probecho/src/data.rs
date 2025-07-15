use iceoryx2::prelude::*;
use iceoryx2_bb_container::vec::FixedSizeVec;

pub type MemShareableMatrix<T, const M: usize, const N: usize> =
    FixedSizeVec<FixedSizeVec<T, N>, M>;
pub type MemShareableAdjMatrix<const N: usize> = MemShareableMatrix<bool, N, N>;

/// A simple payload header.
///
/// Fields:
/// - `src_id`: the source node id of the message;
/// - `dst_id`: the destination node id of the message.
#[derive(Debug, Clone, ZeroCopySend)]
#[type_name("SimpleHeader")]
#[repr(C)]
pub struct SimpleHeader {
    pub src_id: usize,
    pub dst_id: usize,
}

/// An request of the topogoly of an node.
#[derive(Debug, Clone, ZeroCopySend)]
#[type_name("TopologyReq")]
#[repr(C)]
pub enum TopologyReq<const NUM_NODES: usize> {
    Get,
    FullTopology(MemShareableAdjMatrix<NUM_NODES>),
}

/// The response to `TopologyReq` that sends the adjacency matrix of the node.
///
/// Fields:
/// - `topology`: The adjacency matrix representation of the topology, can be set to `None`
///   to indicate that the topology is aredy being computated to other node.
#[derive(Debug, Clone, ZeroCopySend)]
#[type_name("TopologyResp")]
#[repr(C)]
pub enum TopologyResp<const NUM_NODES: usize> {
    LocalTopology(Option<MemShareableAdjMatrix<NUM_NODES>>),
    Stored,
}

/// A ping-pong payload.
#[derive(Debug, Clone, Copy, ZeroCopySend)]
#[type_name("PingPongData")]
#[repr(C)]
pub enum PingPongPayload {
    Ping,
    Pong,
}

impl std::fmt::Display for PingPongPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ping => f.write_str("Ping"),
            Self::Pong => f.write_str("Pong"),
        }
    }
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

impl std::fmt::Display for PingPongHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "msg:{} src:{}→dst:{}",
            self.msg_id, self.src_id, self.dst_id
        ))?;

        if let Some(reply_to) = self.answering_to {
            f.write_fmt(format_args!(" (reply:{})", reply_to))
        } else {
            Ok(())
        }
    }
}
