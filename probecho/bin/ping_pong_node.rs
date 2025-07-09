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

enum LogAction {
    Send,
    Receive,
}

fn log(node_id: usize, action: &LogAction, payload: &PingPongPayload, header: &PingPongHeader) {
    let sep_char = match action {
        LogAction::Send => "→",
        LogAction::Receive => "←",
    };
    println!("[n{}] {} {} [{}]", node_id, sep_char, payload, header);
}

fn main() -> anyhow::Result<()> {
    let mut msg_id: u64 = 0;
    let cli = Cli::parse();
    let interval = Duration::from_millis(cli.interval);

    println!("[n{}] Starting...", cli.id);
    let nsb_node = NsbNodeBuilder::new(cli.id)
        .with_neighboors(cli.neighboors)
        .build_with_header::<PingPongPayload, PingPongHeader, PingPongPayload, PingPongHeader>()?;

    if cli.start_gossip {
        // Send ping to all nodes
        for neigh in nsb_node.neighbors() {
            let mut sample = neigh.client().loan_uninit()?;

            // Write header
            sample.user_header_mut().msg_id = msg_id;
            sample.user_header_mut().src_id = cli.id;
            sample.user_header_mut().dst_id = neigh.id();
            sample.user_header_mut().answering_to = None;

            let sample = sample.write_payload(PingPongPayload::Ping);

            // Log basic info
            log(
                cli.id,
                &LogAction::Send,
                sample.payload(),
                sample.user_header(),
            );

            sample.send()?;
            msg_id += 1;
        }
    }

    // Start listening for messanges
    while nsb_node.inner_node().wait(interval).is_ok() {
        while let Some(sample) = nsb_node.server().receive()? {
            let header = sample.user_header();
            let payload = sample.payload();

            // Log basic info
            log(cli.id, &LogAction::Receive, payload, header);

            // Send Ping again, after Pong.
            let neigh = nsb_node
                .neighboor(&header.src_id)
                .expect("Should not receive a message from a non-neighbor node.");

            let mut client_sample = neigh.client().loan_uninit()?;

            client_sample.user_header_mut().msg_id = msg_id;
            client_sample.user_header_mut().src_id = header.dst_id;
            client_sample.user_header_mut().dst_id = header.src_id;
            client_sample.user_header_mut().answering_to = Some(header.msg_id);

            let send_payload = match payload {
                PingPongPayload::Ping => PingPongPayload::Pong,
                PingPongPayload::Pong => PingPongPayload::Ping,
            };
            let client_sample = client_sample.write_payload(send_payload);

            // Log basic info
            log(
                cli.id,
                &LogAction::Send,
                client_sample.payload(),
                client_sample.user_header(),
            );

            client_sample.send()?;
            msg_id += 1;
        }
    }
    Ok(())
}
