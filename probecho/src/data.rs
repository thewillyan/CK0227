use iceoryx2::prelude::*;
use iceoryx2_bb_container::vec::FixedSizeVec;

pub type MemShareableMatrix<T, const M: usize, const N: usize> =
    FixedSizeVec<FixedSizeVec<T, N>, M>;
pub type MemShareableAdjMatrix<const N: usize> = MemShareableMatrix<bool, N, N>;

pub fn adjacency_matrix<const N: usize>() -> MemShareableAdjMatrix<N> {
    let mut line = FixedSizeVec::new();
    line.extend_from_slice(&[false; N]);

    let mut m = FixedSizeVec::new();
    for _ in 0..N {
        m.push(line.clone());
    }
    m
}

pub trait PathHeader {
    /// Returns the node id of the origin (first sender) of the message.
    fn origin(&self) -> usize;
    /// Returns the node id of the target (final destination) of the message.
    fn target(&self) -> usize;
    /// Returns the sender of the message.
    fn src(&self) -> usize;
    /// Returns the destination of the message.
    fn dst(&self) -> usize;

    /// Returns a mutable reference to the node id of the origin (first sender) of the message.
    fn origin_mut(&mut self) -> &mut usize;
    /// Returns a mutable reference to the node id of the target (final destination) of the message.
    fn target_mut(&mut self) -> &mut usize;
    /// Returns a mutable reference to the sender of the message.
    fn src_mut(&mut self) -> &mut usize;
    /// Returns a mutable reference to the destination of the message.
    fn dst_mut(&mut self) -> &mut usize;
}

/// A simple payload header.
///
/// Fields:
/// - `src_id`: the source node id of the message;
/// - `dst_id`: the destination node id of the message.
/// - `target`: the final target of the message.
#[derive(Debug, Clone, Default, ZeroCopySend)]
#[type_name("SimpleHeader")]
#[repr(C)]
pub struct SimpleHeader {
    pub src_id: usize,
    pub dst_id: usize,
    pub target: usize,
    pub origin: usize,
}

impl SimpleHeader {
    /// Swaps the `src_id` <-> `dst_id` and `target` <-> `origin` of the header, effectively
    /// creating a "reply" header.
    pub fn into_reply(mut self) -> Self {
        std::mem::swap(&mut self.src_id, &mut self.dst_id);
        std::mem::swap(&mut self.origin, &mut self.target);
        self
    }
}

impl PathHeader for SimpleHeader {
    fn origin(&self) -> usize {
        self.origin
    }

    fn target(&self) -> usize {
        self.target
    }

    fn src(&self) -> usize {
        self.src_id
    }
    fn dst(&self) -> usize {
        self.src_id
    }

    fn origin_mut(&mut self) -> &mut usize {
        &mut self.origin
    }

    fn target_mut(&mut self) -> &mut usize {
        &mut self.target
    }

    fn src_mut(&mut self) -> &mut usize {
        &mut self.src_id
    }

    fn dst_mut(&mut self) -> &mut usize {
        &mut self.dst_id
    }
}

impl std::fmt::Display for SimpleHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "src:{}→dst:{} (origin:{}→target:{})",
            self.src_id, self.dst_id, self.origin, self.target
        ))
    }
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
