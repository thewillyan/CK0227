use std::{time::Duration, usize};

use iceoryx2::{
    active_request::ActiveRequest,
    node::{NodeCreationFailure, NodeWaitFailure},
    pending_response::PendingResponse,
    port::{
        LoanError, ReceiveError, SendError,
        client::{Client, RequestSendError},
        server::Server,
    },
    prelude::*,
    service::{
        builder::request_response::{RequestResponseCreateError, RequestResponseOpenError},
        port_factory::{client::ClientCreateError, server::ServerCreateError},
        service_name::ServiceNameError,
    },
};
use thiserror::Error;

use crate::data::{MemShareableAdjMatrix, SimpleHeader, TopologyReq, TopologyResp};

const INF: usize = usize::MAX;

/// Represents a failure when trying to build a `StbNode`.
#[derive(Debug, Error)]
pub enum StbNodeError {
    #[error(transparent)]
    NodeCreationFailure(NodeCreationFailure),
    #[error(transparent)]
    InvalidServiceName(ServiceNameError),
    #[error(transparent)]
    ServiceCreateError(RequestResponseCreateError),
    #[error(transparent)]
    ServiceOpenError(RequestResponseOpenError),
    #[error(transparent)]
    ServerCreateError(ServerCreateError),
    #[error(transparent)]
    ClientCreateError(ClientCreateError),
    #[error(transparent)]
    WaitFailure(NodeWaitFailure),
    #[error(transparent)]
    LoanError(LoanError),
    #[error(transparent)]
    ReqSendError(RequestSendError),
    #[error(transparent)]
    ReceiveError(ReceiveError),
    #[error(transparent)]
    SendError(SendError),
}

/// Builder of a `TopologyUnawareStbNode`.
#[derive(Debug, Clone)]
pub struct StbNodeBuilder {
    id: usize,
    neighbors: Vec<usize>,
    config: Config,
}

impl StbNodeBuilder {
    /// Creates a new `StbNodeBuilder`.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            neighbors: Vec::new(),
            config: Config::default(),
        }
    }

    /// Sets the neighborhood of the node.
    pub fn with_neighboors(mut self, neighbors: Vec<usize>) -> Self {
        self.neighbors = neighbors;
        self
    }

    /// Sets the node config.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Get service name for a node of given `id`.
    fn unaware_service_name(id: usize) -> Result<ServiceName, StbNodeError> {
        format!("Probecho/UnawareStb/Node{}", id)
            .as_str()
            .try_into()
            .map_err(StbNodeError::InvalidServiceName)
    }

    /// Build a new `TopologyUnawareStbNode`.
    pub fn build<const NUM_NODES: usize>(
        mut self,
    ) -> Result<TopologyUnawareStbNode<NUM_NODES>, StbNodeError> {
        let node = NodeBuilder::new()
            .config(&self.config)
            .create::<ipc::Service>()
            .map_err(StbNodeError::NodeCreationFailure)?;

        let service = node
            .service_builder(&Self::unaware_service_name(self.id)?)
            .request_response::<TopologyReq<NUM_NODES>, TopologyResp<NUM_NODES>>()
            .request_user_header::<SimpleHeader>()
            .response_user_header::<SimpleHeader>()
            .max_servers(1)
            .max_clients(self.neighbors.len())
            .create()
            .map_err(StbNodeError::ServiceCreateError)?;

        let server = service
            .server_builder()
            .create()
            .map_err(StbNodeError::ServerCreateError)?;

        let mut neighbors = Vec::with_capacity(self.neighbors.len());
        // Make neighboors list ordered so binary search is possible.
        self.neighbors.sort();
        for n in self.neighbors {
            // Attempt `max_attempts` times to connect to the neighbors.
            let max_attempts: u64 = 10;
            let mut attempts: u64 = 1;
            let interval = Duration::from_millis(100);
            let service = loop {
                let service_result = node
                    .service_builder(&Self::unaware_service_name(n)?)
                    .request_response::<TopologyReq<NUM_NODES>, TopologyResp<NUM_NODES>>()
                    .request_user_header::<SimpleHeader>()
                    .response_user_header::<SimpleHeader>()
                    .open()
                    .map_err(StbNodeError::ServiceOpenError);

                // If max attempts reacher "it is what it is"
                if attempts == max_attempts {
                    break service_result?;
                }

                // If not, verify if sucessfull if not, try again after waiting for `interval`
                if let Ok(s) = service_result {
                    break s;
                }
                attempts += 1;
                node.wait(interval).map_err(StbNodeError::WaitFailure)?;
            };

            let client = service
                .client_builder()
                .create()
                .map_err(StbNodeError::ClientCreateError)?;

            neighbors.push(TopologyUnawareStbNeighboor { id: n, client });
        }

        Ok(TopologyUnawareStbNode {
            id: self.id,
            node,
            server,
            neighbors,
            state: UnawareState::WaitingGetReq,
        })
    }
}

/// A node that implements `broadcast` by using neighbor sets.
pub struct TopologyUnawareStbNode<const NUM_NODES: usize> {
    id: usize,
    node: Node<ipc::Service>,
    server: Server<
        ipc::Service,
        TopologyReq<NUM_NODES>,
        SimpleHeader,
        TopologyResp<NUM_NODES>,
        SimpleHeader,
    >,
    neighbors: Vec<TopologyUnawareStbNeighboor<NUM_NODES>>,
    state: UnawareState<NUM_NODES>,
}

enum UnawareState<const NUM_NODES: usize> {
    WaitingGetReq,
    GatheringLocalTopology {
        origin_req: Option<
            ActiveRequest<
                ipc::Service,
                TopologyReq<NUM_NODES>,
                SimpleHeader,
                TopologyResp<NUM_NODES>,
                SimpleHeader,
            >,
        >,
        pending_responses: Vec<
            PendingResponse<
                ipc::Service,
                TopologyReq<NUM_NODES>,
                SimpleHeader,
                TopologyResp<NUM_NODES>,
                SimpleHeader,
            >,
        >,
    },
    WaitingGlobalTopology,
    ReceivedGlobalTopology {
        topology: MemShareableAdjMatrix<NUM_NODES>,
    },
}

impl<const NUM_NODES: usize> TopologyUnawareStbNode<NUM_NODES> {
    pub fn run_initiantor<Req, ReqHeader, Resp, RespHeader>(
        mut self,
    ) -> Result<TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, NUM_NODES>, StbNodeError>
    where
        Req: std::fmt::Debug + ZeroCopySend,
        ReqHeader: std::fmt::Debug + ZeroCopySend,
        Resp: std::fmt::Debug + ZeroCopySend,
        RespHeader: std::fmt::Debug + ZeroCopySend,
    {
        self.request_topology(None)?;
        self.run()
    }

    pub fn run<Req, ReqHeader, Resp, RespHeader>(
        mut self,
    ) -> Result<TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, NUM_NODES>, StbNodeError>
    where
        Req: std::fmt::Debug + ZeroCopySend,
        ReqHeader: std::fmt::Debug + ZeroCopySend,
        Resp: std::fmt::Debug + ZeroCopySend,
        RespHeader: std::fmt::Debug + ZeroCopySend,
    {
        let topology = loop {
            if let UnawareState::ReceivedGlobalTopology { topology } = self.state {
                break topology;
            }
            self.handle_request()?;

            if let UnawareState::GatheringLocalTopology {
                origin_req: _,
                pending_responses: _,
            } = self.state
            {
                self.gather_topology()?;
            }
        };

        let service = self
            .node
            .service_builder(&Self::aware_service_name(self.id)?)
            .request_response::<Req, Resp>()
            .request_user_header::<ReqHeader>()
            .response_user_header::<RespHeader>()
            .max_servers(1)
            .max_clients(self.neighbors.len())
            .create()
            .map_err(StbNodeError::ServiceCreateError)?;

        let server = service
            .server_builder()
            .create()
            .map_err(StbNodeError::ServerCreateError)?;

        let mut neighbors = Vec::with_capacity(self.neighbors.len());
        for n in self.neighbors {
            // Attempt `max_attempts` times to connect to the neighbors.
            let max_attempts: u64 = 10;
            let mut attempts: u64 = 1;
            let interval = Duration::from_millis(100);
            let service = loop {
                let service_result = self
                    .node
                    .service_builder(&Self::aware_service_name(n.id)?)
                    .request_response::<Req, Resp>()
                    .request_user_header::<ReqHeader>()
                    .response_user_header::<RespHeader>()
                    .open()
                    .map_err(StbNodeError::ServiceOpenError);

                // If max attempts reacher "it is what it is"
                if attempts == max_attempts {
                    break service_result?;
                }

                // If not, verify if sucessfull if not, try again after waiting for `interval`
                if let Ok(s) = service_result {
                    break s;
                }
                attempts += 1;
                self.node
                    .wait(interval)
                    .map_err(StbNodeError::WaitFailure)?;
            };

            let client = service
                .client_builder()
                .create()
                .map_err(StbNodeError::ClientCreateError)?;

            neighbors.push(TopologyAwareStbNeighboor { id: n.id, client });
        }

        let next_node = Self::next_node_arr(self.id, &topology);
        Ok(TopologyAwareStbNode {
            id: self.id,
            node: self.node,
            server,
            neighbors,
            next_node,
        })
    }

    fn next_node_arr(
        src: usize,
        adj: &MemShareableAdjMatrix<NUM_NODES>,
    ) -> [Option<usize>; NUM_NODES] {
        let mut dist = [[INF; NUM_NODES]; NUM_NODES];
        let mut next_node = [[None; NUM_NODES]; NUM_NODES];

        for i in 0..NUM_NODES {
            for j in 0..NUM_NODES {
                if i == j {
                    dist[i][j] = 0;
                    next_node[i][j] = Some(j);
                } else if adj[i][j] {
                    dist[i][j] = 1;
                    next_node[i][j] = Some(j);
                }
            }
        }

        for k in 0..NUM_NODES {
            for i in 0..NUM_NODES {
                if dist[i][k] == INF {
                    continue;
                }
                for j in 0..NUM_NODES {
                    if dist[i][k] + dist[k][j] < dist[i][j] {
                        dist[i][j] = dist[i][k] + dist[k][j];
                        next_node[i][j] = next_node[i][k];
                    }
                }
            }
        }

        let shortest_path_next = [None; NUM_NODES];
        for j in 0..NUM_NODES {}
        shortest_path_next
    }

    /// Get `AwareStbNode` service name for a node of given `id`.
    fn aware_service_name(id: usize) -> Result<ServiceName, StbNodeError> {
        format!("Probecho/AwareStb/Node{}", id)
            .as_str()
            .try_into()
            .map_err(StbNodeError::InvalidServiceName)
    }

    /// Request the topology for all the neighbors ignoring the
    /// `origin_id`, if setted. Returns the pending response
    fn request_topology(
        &mut self,
        origin_req: Option<
            ActiveRequest<
                ipc::Service,
                TopologyReq<NUM_NODES>,
                SimpleHeader,
                TopologyResp<NUM_NODES>,
                SimpleHeader,
            >,
        >,
    ) -> Result<(), StbNodeError> {
        let mut pending_responses = Vec::with_capacity(self.neighbors.len());
        for neigh in &self.neighbors {
            // get empty sample of a message
            let mut sample = neigh
                .client
                .loan_uninit()
                .map_err(StbNodeError::LoanError)?;

            // write header
            let header = sample.user_header_mut();
            header.src_id = self.id;
            header.dst_id = neigh.id;

            // write payload
            let payload = TopologyReq::Get;
            let sample = sample.write_payload(payload);
            let pending = sample.send().map_err(StbNodeError::ReqSendError)?;
            pending_responses.push(pending);
        }
        self.state = UnawareState::GatheringLocalTopology {
            origin_req,
            pending_responses,
        };
        Ok(())
    }

    fn gather_topology(&mut self) -> Result<(), StbNodeError> {
        if let UnawareState::GatheringLocalTopology {
            origin_req,
            pending_responses,
        } = &self.state
        {
            // start local topology with all values setted to default (which for booleans in `false`)
            let mut local_topology: MemShareableAdjMatrix<NUM_NODES> = Default::default();

            // merge the received topologies
            for pending in pending_responses {
                let resp = pending.receive().map_err(StbNodeError::ReceiveError)?;
                if let Some(value) = resp {
                    match value.payload() {
                        TopologyResp::LocalTopology(Some(t)) => {
                            for i in 0..NUM_NODES {
                                for j in 0..NUM_NODES {
                                    local_topology[i][j] = local_topology[i][j] || t[i][j]
                                }
                            }
                        }
                        TopologyResp::LocalTopology(None) => (), // empty topology, ignore
                        TopologyResp::Stored => unreachable!(
                            "Should not receive Stored when state is GatheringLocalTopology"
                        ),
                    }
                } else {
                    // if a single response has not arrived, abort!
                    break;
                }
            }

            match origin_req {
                Some(req) => {
                    // the node is not the initiator
                    let dst_id = req.user_header().src_id;
                    let mut response = req.loan_uninit().map_err(StbNodeError::LoanError)?;

                    // write header
                    let header = response.user_header_mut();
                    header.src_id = self.id;
                    header.dst_id = dst_id;

                    // write payload
                    let payload = TopologyResp::LocalTopology(Some(local_topology));
                    let response = response.write_payload(payload);
                    response.send().map_err(StbNodeError::SendError)?;
                    self.state = UnawareState::WaitingGlobalTopology;
                }
                None => {
                    // the node is the initiator
                    self.state = UnawareState::ReceivedGlobalTopology {
                        topology: local_topology,
                    };
                    self.broadcast_topology()?;
                }
            }
        }
        Ok(())
    }

    fn broadcast_topology(&self) -> Result<(), StbNodeError> {
        if let UnawareState::ReceivedGlobalTopology { topology } = &self.state {
            for neigh in &self.neighbors {
                let mut sample = neigh
                    .client
                    .loan_uninit()
                    .map_err(StbNodeError::LoanError)?;

                // write header
                let header = sample.user_header_mut();
                header.src_id = self.id;
                header.dst_id = neigh.id;

                // write payload
                let payload = TopologyReq::FullTopology(topology.clone());
                let sample = sample.write_payload(payload);
                sample.send().map_err(StbNodeError::ReqSendError)?;
            }
        }
        Ok(())
    }

    /// Handle a single topology request.
    fn handle_request(&mut self) -> Result<(), StbNodeError> {
        if let Some(req) = self.server.receive().map_err(StbNodeError::ReceiveError)? {
            let payload = req.payload();
            match payload {
                TopologyReq::Get => {
                    if let UnawareState::WaitingGetReq = self.state {
                        self.request_topology(Some(req))?;
                    } else {
                        let dst_id = req.user_header().dst_id;
                        let mut response = req.loan_uninit().map_err(StbNodeError::LoanError)?;

                        // write header
                        let header = response.user_header_mut();
                        header.src_id = self.id;
                        header.dst_id = dst_id;

                        // write payload
                        let payload = TopologyResp::LocalTopology(None);
                        let response = response.write_payload(payload);

                        // send message
                        response.send().map_err(StbNodeError::SendError)?;
                    }
                }
                TopologyReq::FullTopology(t) => {
                    // store global topology
                    self.state = UnawareState::ReceivedGlobalTopology {
                        topology: t.clone(),
                    };

                    // broadcast global topology
                    self.broadcast_topology()?;

                    // respond request
                    let dst_id = req.user_header().dst_id;
                    let mut response = req.loan_uninit().map_err(StbNodeError::LoanError)?;

                    // write header
                    let header = response.user_header_mut();
                    header.src_id = self.id;
                    header.dst_id = dst_id;

                    // write payload
                    let payload = TopologyResp::Stored;
                    let response = response.write_payload(payload);

                    // send message
                    response.send().map_err(StbNodeError::SendError)?;
                }
            }
        }
        Ok(())
    }
}

/// A neighboor of an node of the Stb network.
pub struct TopologyUnawareStbNeighboor<const NUM_NODES: usize> {
    id: usize,
    client: Client<
        ipc::Service,
        TopologyReq<NUM_NODES>,
        SimpleHeader,
        TopologyResp<NUM_NODES>,
        SimpleHeader,
    >,
}

impl<const NUM_NODES: usize> TopologyUnawareStbNeighboor<NUM_NODES> {}

/// A node that implements `broadcast` by using neighbor sets.
pub struct TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, const NUM_NODES: usize>
where
    Req: std::fmt::Debug + ZeroCopySend,
    ReqHeader: std::fmt::Debug + ZeroCopySend,
    Resp: std::fmt::Debug + ZeroCopySend,
    RespHeader: std::fmt::Debug + ZeroCopySend,
{
    id: usize,
    node: Node<ipc::Service>,
    server: Server<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
    // for every index i, next_node[i] contains the id of the next node in the shortest path
    // where the destination id is i.
    next_node: [Option<usize>; NUM_NODES],
    neighbors: Vec<TopologyAwareStbNeighboor<Req, ReqHeader, Resp, RespHeader>>,
}

/// A neighboor of an `TopologyAwareStbNode`.
pub struct TopologyAwareStbNeighboor<Req, ReqHeader, Resp, RespHeader>
where
    Req: std::fmt::Debug + ZeroCopySend,
    ReqHeader: std::fmt::Debug + ZeroCopySend,
    Resp: std::fmt::Debug + ZeroCopySend,
    RespHeader: std::fmt::Debug + ZeroCopySend,
{
    id: usize,
    client: Client<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
}
