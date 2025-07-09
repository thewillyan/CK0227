use iceoryx2::prelude::*;

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
