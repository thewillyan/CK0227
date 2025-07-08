use clap::Parser;
use std::time::Duration;

use probecho::data::{PingPongHeader, PingPongPayload};
use probecho::process::NsbNodeBuilder;

/// Runs a Graphnet Node process.
#[derive(Parser, Debug)]
#[command(about)]
struct Cli {
    /// Node ID
    #[arg(short, long, value_name = "ID")]
    id: usize,
    /// Sets the node neighboors
    #[arg(short, long, value_delimiter = ',')]
    neighboors: Vec<usize>,
    /// Sets if the node should start the gossip
    #[arg(short, long, default_value_t = false)]
    start_gossip: bool,
    /// Sets the interval, in milliseconds, for searching the inbox
    #[arg(long, default_value_t = 500, value_name = "MILLIS")]
    interval: u64,
}

fn main() -> anyhow::Result<()> {
    let mut msg_id: u64 = 0;
    let cli = Cli::parse();
    let interval = Duration::from_millis(cli.interval);

    println!("[n{}] Starting...", cli.id);
    let nsb_node = NsbNodeBuilder::new(cli.id)
        .with_neighboors(cli.neighboors)
        .build::<PingPongPayload, PingPongHeader>()?;

    if cli.start_gossip {
        // Send ping to all nodes
        for outbox in nsb_node.outboxes() {
            println!("[n{}] Ping {}", cli.id, outbox.destination());
            let mut sample = outbox.sender().loan_uninit()?;

            // Write header
            sample.user_header_mut().msg_id = msg_id;
            sample.user_header_mut().src_id = cli.id;
            sample.user_header_mut().dst_id = outbox.destination();
            sample.user_header_mut().answering_to = None;

            let sample = sample.write_payload(PingPongPayload::Ping);
            sample.send()?;
            msg_id += 1;
        }
    }

    // Start listening for messanges
    while nsb_node.inner_node().wait(interval).is_ok() {
        while let Some(sample) = nsb_node.inbox().receive()? {
            let header = sample.user_header();
            println!("[n{}] Pong from n{}", cli.id, header.src_id);

            // Send Ping again, after Pong.
            let outbox = nsb_node
                .outbox(&header.src_id)
                .expect("Should not receive a message from a non-neighbor node.");

            let mut sender_sample = outbox.sender().loan_uninit()?;

            sender_sample.user_header_mut().msg_id = msg_id;
            sender_sample.user_header_mut().src_id = header.dst_id;
            sender_sample.user_header_mut().dst_id = header.src_id;
            sender_sample.user_header_mut().answering_to = Some(header.msg_id);

            let sender_sample = sender_sample.write_payload(PingPongPayload::Ping);
            sender_sample.send()?;
            msg_id += 1;
        }
    }
    Ok(())
}
