use probecho::network::TopologyBuilder;
use std::{process::Command, thread::sleep, time::Duration};

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

const BIN_NAME: &str = "ping-pong-node";
const RELEASE_BINS_PATH: &str = "target/release";
const DEBUG_BINS_PATH: &str = "target/debug";
const NUM_NODES: usize = 4;

fn build_cmd() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    cmd.arg("--features").arg("bins").arg("--bin").arg(BIN_NAME);
    cmd
}

fn run_cmd(id: usize, neighs: &[usize], start_gossip: bool) -> Command {
    let bin_path = if cfg!(debug_assertions) {
        DEBUG_BINS_PATH
    } else {
        RELEASE_BINS_PATH
    };

    let neighs = neighs
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let mut cmd = Command::new(format!("{bin_path}/{BIN_NAME}"));
    cmd.arg("-i").arg(id.to_string()).arg("-n").arg(neighs);

    if start_gossip {
        cmd.arg("-s");
    }

    cmd
}

fn main() -> anyhow::Result<()> {
    let topology = TopologyBuilder::full().build::<NUM_NODES>()?;
    let mut node_procs = Vec::with_capacity(NUM_NODES);
    build_cmd().output()?;

    for id in (0..NUM_NODES).rev() {
        let neighs: Vec<_> = topology
            .connections_unchecked(id)
            .iter()
            .enumerate()
            .filter_map(
                |(node_idx, connected)| {
                    if *connected { Some(node_idx) } else { None }
                },
            )
            .collect();

        let process = run_cmd(id, &neighs, id == 0).spawn()?;
        node_procs.push(process);
    }

    let duration = Duration::from_secs(5);
    sleep(duration);

    for p in node_procs.iter_mut() {
        kill(Pid::from_raw(p.id() as i32), Signal::SIGTERM)?;
    }
    Ok(())
}
