use std::ops::Range;
use std::{process::Command, time::Duration};

use clap::{Args, Parser, Subcommand, ValueEnum};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use probecho::{
    data::{PingPongPayload, SimpleHeader},
    network::{TopologyBuilder, TopologyKind},
    process::stb::{
        StbActiveRequest, StbNodeBuilder, StbNodeError, StbPendingResponse, TopologyAwareStbNode,
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

/// Simulates a Spanning Tree Broadcast with 8 nodes where the initiator node sends a message
/// to an arbtrary node.
///
/// The simulation ends when the source receives the response from the destination.
#[derive(Parser)]
#[command(about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs a Spanning Tree Broadcast Node process.
    Node(NodeArgs),
    /// Runs a Spanning Tree Broadcast Simulation Manager (start the simulation).
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
    /// Set's to which node to send a ping (in this case the node is the initiator)
    #[arg(short, long, value_parser = nid_in_range)]
    ping: Option<usize>,
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
    /// Which node is the initiator
    #[arg(short, long, default_value_t = 0, value_parser = nid_in_range)]
    initiator: usize,
    /// To which node the initiator should send the message
    #[arg(short, long, value_parser = nid_in_range)]
    destination: usize,
    /// What is the topology of the network
    #[arg(short, long, value_enum)]
    topology: TopologyArg,
}

enum WorkerState {
    WaitingRequest,
    PendingResponse {
        request: StbActiveRequest<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>,
        pending: StbPendingResponse<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>,
    },
    Responded,
}

fn log(id: usize, data: &PingPongPayload, header: &SimpleHeader) {
    if id == header.src_id {
        // is sending
        println!(
            "[n{}] {} → n{} (orgin:{}→target:{})",
            id, data, header.dst_id, header.origin, header.target
        );
    } else {
        // is receiving
        println!(
            "[n{}] {} ← n{} (orgin:{}→target:{})",
            id, data, header.src_id, header.origin, header.target
        );
    };
}

fn node(args: NodeArgs) -> Result<(), StbNodeError> {
    let interval = Duration::from_millis(args.interval);
    let node = StbNodeBuilder::new(args.id)
        .with_neighbors(args.neighboors)
        .build::<NUM_NODES>()?;

    if let Some(target) = args.ping {
        // We're the initiator process
        let node =
            node.run_initiator::<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>()?;

        // Send ping to a far away node
        let pending = node.send(PingPongPayload::Ping, target, |src_id, dst_id| {
            SimpleHeader {
                src_id,
                dst_id,
                target,
                origin: args.id,
            }
        })?;
        log(args.id, pending.data(), pending.header());

        let answer = loop {
            if let Some(data) = pending.receive()? {
                break data;
            }
            node.wait(interval)?;
        };
        log(args.id, answer.data(), answer.header());
    } else {
        // We're a worker process
        let node = node.run()?;
        let mut state = WorkerState::WaitingRequest;

        // Start waiting for requests
        while node.wait(interval).is_ok() {
            match &state {
                WorkerState::WaitingRequest => {
                    if let Some(req) = node.receive()? {
                        log(args.id, req.data(), req.header());
                        state = handle_request(args.id, &node, req)?;
                    }
                }
                WorkerState::PendingResponse { request, pending } => {
                    if let Some(resp) = pending.receive()? {
                        log(args.id, resp.data(), resp.header());
                        let data = resp.data().clone();
                        let mut header = resp.header().clone();
                        header.src_id = args.id;
                        header.dst_id = request.header().src_id;

                        log(args.id, &data, &header);
                        request.reply(data, header)?;
                        state = WorkerState::Responded;
                    }
                }
                WorkerState::Responded => break,
            }
        }
    };
    Ok(())
}

fn handle_request(
    id: usize,
    node: &TopologyAwareStbNode<
        PingPongPayload,
        SimpleHeader,
        PingPongPayload,
        SimpleHeader,
        NUM_NODES,
    >,
    req: StbActiveRequest<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>,
) -> Result<WorkerState, StbNodeError> {
    let mut header = req.header().clone();
    let data = req.data().clone();

    let state = if header.target == id {
        let data = match req.data() {
            PingPongPayload::Ping => PingPongPayload::Pong,
            PingPongPayload::Pong => PingPongPayload::Ping,
        };
        let header = req.header().clone().into_reply();
        log(id, &data, &header);
        req.reply(data, header)?;
        WorkerState::Responded
    } else {
        let pending = node.send(data, header.target, |src_id, dst_id| {
            header.src_id = src_id;
            header.dst_id = dst_id;
            header
        })?;
        log(id, pending.data(), pending.header());
        WorkerState::PendingResponse {
            request: req,
            pending,
        }
    };

    Ok(state)
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
            cmd.arg("-p").arg(args.destination.to_string());
        }
        nodes_procs.push(cmd.spawn()?);
    }

    // wait for the gossiper to finish
    nodes_procs[0].wait()?;

    // kill other processes that dont have finished yet
    for p in nodes_procs.iter_mut().skip(1) {
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
