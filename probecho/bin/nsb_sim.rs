use std::ops::Range;
use std::{process::Command, time::Duration};

use clap::{Args, Parser, Subcommand, ValueEnum};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use probecho::{
    data::{PingPongPayload, SimpleHeader},
    network::{TopologyBuilder, TopologyKind},
    process::nsb::{
        NsbNodeBuilder, NsbNodeError,
    },
};

const NUM_NODES: usize = 8;
const NID_RANGE: Range<usize> = 0..NUM_NODES;

fn nid_in_range(s: &str) -> Result<usize, String> {
    let id: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a valid node id"))?;

    if NID_RANGE.contains(&id) {
        Ok(id)
    } else {
        Err(format!(
            "id out of range {}-{}",
            NID_RANGE.start, NID_RANGE.end
        ))
    }
}

/// Simulates a Neighbor Set Broadcast with 8 nodes where the initiator node broadcasts a message
/// to all nodes in the network using the probe/echo paradigm.
///
/// The simulation demonstrates the NSB algorithm where each node forwards the message to all its
/// neighbors and then receives redundant copies which are ignored.
#[derive(Parser)]
#[command(about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs a Neighbor Set Broadcast Node process.
    Node(NodeArgs),
    /// Runs a Neighbor Set Broadcast Simulation Manager (start the simulation).
    Manager(ManagerArgs),
}

#[derive(Args, Debug)]
struct NodeArgs {
    /// Node ID
    #[arg(short, long, value_name = "ID", value_parser = nid_in_range)]
    id: usize,
    /// Sets the node neighboors
    #[arg(short, long, value_delimiter = ',', value_parser = nid_in_range)]
    neighboors: Vec<usize>,
    /// Set's this node as the broadcast initiator
    #[arg(short, long)]
    broadcast: bool,
    /// Sets the interval, in milliseconds, for searching the inbox
    #[arg(long, default_value_t = 100, value_name = "MILLIS")]
    interval: u64,
}

#[derive(Debug, Clone, ValueEnum)]
enum TopologyArg {
    Full,
    Ring,
    Star,
}

impl Into<TopologyKind> for TopologyArg {
    fn into(self) -> TopologyKind {
        match self {
            Self::Full => TopologyKind::Full,
            Self::Ring => TopologyKind::Ring,
            Self::Star => TopologyKind::Star,
        }
    }
}

#[derive(Args, Debug)]
struct ManagerArgs {
    /// Which node is the broadcast initiator
    #[arg(short, long, default_value_t = 0, value_parser = nid_in_range)]
    initiator: usize,
    /// What is the topology of the network
    #[arg(short, long, value_enum)]
    topology: TopologyArg,
}

/// State machine for NSB nodes
enum NsbNodeState {
    /// Waiting for the first message
    WaitingFirstMessage,
    /// Already received and forwarded the message, now receiving redundant copies
    ReceivingRedundantCopies {
        expected_copies: usize,
        received_copies: usize,
    },
    /// Finished processing
    Finished,
}

fn log(id: usize, data: &PingPongPayload, header: &SimpleHeader) {
    if id == header.src_id {
        // is sending
        println!(
            "[n{}] {} → n{} (origin:{}→broadcast)",
            id, data, header.dst_id, header.origin
        );
    } else {
        // is receiving
        println!(
            "[n{}] {} ← n{} (origin:{}→broadcast)",
            id, data, header.src_id, header.origin
        );
    };
}

fn log_ignore(id: usize, data: &PingPongPayload, header: &SimpleHeader) {
    println!(
        "[n{}] {} ← n{} (origin:{}→broadcast) [IGNORED - redundant]",
        id, data, header.src_id, header.origin
    );
}

fn node(args: NodeArgs) -> Result<(), NsbNodeError> {
    let interval = Duration::from_millis(args.interval);
    let node = NsbNodeBuilder::new(args.id)
        .with_neighbors(args.neighboors.clone())
        .build::<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>()?;

    if args.broadcast {
        // We're the initiator process - broadcast to all neighbors
        let node = node.run_initiator()?;
        
        // Send the broadcast message to all neighbors
        let broadcast_message = PingPongPayload::Ping;
        for &neighbor_id in &args.neighboors {
            node.send(broadcast_message, neighbor_id, |src_id, dst_id| {
                SimpleHeader {
                    src_id,
                    dst_id,
                    target: dst_id, // Not used in broadcast, but required
                    origin: args.id, // The original broadcaster
                }
            })?;
            log(args.id, &broadcast_message, &SimpleHeader {
                src_id: args.id,
                dst_id: neighbor_id,
                target: neighbor_id,
                origin: args.id,
            });
        }
        
        // The initiator also needs to implement the NSB protocol to receive messages from others
        let mut state = NsbNodeState::ReceivingRedundantCopies {
            expected_copies: args.neighboors.len(), // Expect to receive from all neighbors
            received_copies: 0,
        };

        // Start waiting for messages
        while node.wait(interval).is_ok() {
            match &mut state {
                NsbNodeState::ReceivingRedundantCopies { expected_copies, received_copies } => {
                    if let Some(msg) = node.receive()? {
                        *received_copies += 1;
                        log_ignore(args.id, msg.data(), msg.header());
                        
                        // Check if we've received all expected copies
                        if *received_copies >= *expected_copies {
                            state = NsbNodeState::Finished;
                        }
                    }
                }
                NsbNodeState::Finished => {
                    break;
                }
                _ => {} // Should not happen for initiator
            }
        }
        
        return Ok(());
    } else {
        // We're a worker process implementing the NSB algorithm
        let node = node.run()?;
        let mut state = NsbNodeState::WaitingFirstMessage;
        let num_neighbors = args.neighboors.len();

        // Start waiting for messages
        while node.wait(interval).is_ok() {
            match &mut state {
                NsbNodeState::WaitingFirstMessage => {
                    if let Some(msg) = node.receive()? {
                        log(args.id, msg.data(), msg.header());
                        
                        // Forward message to all neighbors (including sender)
                        for &neighbor_id in &args.neighboors {
                            node.send(msg.data().clone(), neighbor_id, |src_id, dst_id| {
                                SimpleHeader {
                                    src_id,
                                    dst_id,
                                    target: dst_id,
                                    origin: msg.header().origin, // Preserve original broadcaster
                                }
                            })?;
                            log(args.id, msg.data(), &SimpleHeader {
                                src_id: args.id,
                                dst_id: neighbor_id,
                                target: neighbor_id,
                                origin: msg.header().origin,
                            });
                        }
                        
                        // Now wait for redundant copies (num_neighbors - 1 more copies)
                        if num_neighbors > 1 {
                            state = NsbNodeState::ReceivingRedundantCopies {
                                expected_copies: num_neighbors - 1,
                                received_copies: 0,
                            };
                        } else {
                            state = NsbNodeState::Finished;
                        }
                    }
                }
                NsbNodeState::ReceivingRedundantCopies { expected_copies, received_copies } => {
                    if let Some(msg) = node.receive()? {
                        *received_copies += 1;
                        log_ignore(args.id, msg.data(), msg.header());
                        
                        // Check if we've received all expected redundant copies
                        if *received_copies >= *expected_copies {
                            state = NsbNodeState::Finished;
                        }
                    }
                }
                NsbNodeState::Finished => {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn manager(args: ManagerArgs) -> anyhow::Result<()> {
    let bin_path = std::env::current_exe().unwrap();
    let topology = TopologyBuilder::new()
        .with_base(args.topology.into())
        .build::<NUM_NODES>()?;
    let mut nodes_procs = Vec::with_capacity(NUM_NODES);

    for id in 0..NUM_NODES {
        let neighboors = topology
            .connections_unchecked(id)
            .iter()
            .enumerate()
            .filter_map(|(node_idx, connected)| {
                if *connected {
                    Some(node_idx.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        let mut cmd = Command::new(&bin_path);
        cmd.arg("node")
            .arg("-i")
            .arg(id.to_string())
            .arg("-n")
            .arg(neighboors);

        if id == args.initiator {
            cmd.arg("--broadcast");
        }
        nodes_procs.push(cmd.spawn()?);
    }

    // Give more time for all nodes to process the broadcast
    std::thread::sleep(Duration::from_secs(5));

    // kill all processes
    for p in nodes_procs.iter_mut() {
        if p.try_wait()?.is_none() {
            kill(Pid::from_raw(p.id() as i32), Signal::SIGTERM)?;
        }
    }
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Node(args) => Ok(node(args)?),
        Commands::Manager(args) => manager(args),
    }
}
