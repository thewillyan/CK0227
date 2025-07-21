use std::{process::Command, time::Duration};

use clap::{Args, Parser, Subcommand};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use probecho::{
    data::{PingPongPayload, SimpleHeader},
    network::{TopologyBuilder, TopologyKind},
    process::stb::{
        StbActiveRequest, StbNodeBuilder, StbNodeError, StbPendingResponse, TopologyAwareStbNode,
    },
};

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
    /// Runs a Spanning Tree Broadcast Simulation Manager.
    Manager(ManagerArgs),
}

#[derive(Args, Debug)]
struct NodeArgs {
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
    #[arg(long, default_value_t = 200, value_name = "MILLIS")]
    interval: u64,
}

#[derive(Args, Debug)]
struct ManagerArgs {
    #[arg(short, long, value_enum)]
    topology: TopologyKind,
}

const NUM_NODES: usize = 5;

enum WorkerState {
    WaitingRequest,
    PendingResponse {
        request: StbActiveRequest<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>,
        pending: StbPendingResponse<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>,
    },
    Responded,
}

fn log(id: usize, data: &PingPongPayload, header: &SimpleHeader) {
    let sep_char = if id == header.src_id {
        // is sending
        "→"
    } else {
        // is receiving
        "←"
    };
    println!("[n{}] {} {} [{}]", id, sep_char, data, header);
}

fn node(args: NodeArgs) -> Result<(), StbNodeError> {
    let interval = Duration::from_millis(args.interval);
    let node = StbNodeBuilder::new(args.id)
        .with_neighbors(args.neighboors)
        .build::<NUM_NODES>()?;

    if args.start_gossip {
        // We're the initiator process
        let node =
            node.run_initiator::<PingPongPayload, SimpleHeader, PingPongPayload, SimpleHeader>()?;
        let target = (args.id + NUM_NODES / 2) % NUM_NODES;

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
                        state = handle_request(args.id, &node, req)?;
                    }
                }
                WorkerState::PendingResponse { request, pending } => {
                    if let Some(resp) = pending.receive()? {
                        let data = resp.data().clone();
                        let header = resp.header().clone();
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
        .with_base(args.topology)
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

        if id == 0 {
            cmd.arg("-s");
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
