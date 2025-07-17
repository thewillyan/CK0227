use std::{collections::VecDeque, fmt, time::Duration};

use iceoryx2::{
    active_request::ActiveRequest,
    node::{Node, NodeBuilder, NodeCreationFailure, NodeWaitFailure},
    pending_response::PendingResponse,
    port::{
        LoanError, ReceiveError, SendError,
        client::{Client, RequestSendError},
        server::Server,
    },
    prelude::*,
    response::Response,
    service::{
        builder::request_response::{RequestResponseCreateError, RequestResponseOpenError},
        port_factory::{
            client::ClientCreateError, request_response::PortFactory, server::ServerCreateError,
        },
        service_name::ServiceNameError,
    },
};
use thiserror::Error;

use crate::data::{
    DestinationHeader, MemShareableAdjMatrix, SimpleHeader, TopologyReq, TopologyResp,
};

/// Represents all possible failures when building or operating an STB node
#[derive(Debug, Error)]
pub enum StbNodeError {
    #[error("Node creation failed: {0}")]
    NodeCreationFailure(#[from] NodeCreationFailure),
    #[error("Invalid service name: {0}")]
    InvalidServiceName(#[from] ServiceNameError),
    #[error("Service creation failed: {0}")]
    ServiceCreateError(#[from] RequestResponseCreateError),
    #[error("Service opening failed: {0}")]
    ServiceOpenError(#[from] RequestResponseOpenError),
    #[error("Server creation failed: {0}")]
    ServerCreateError(#[from] ServerCreateError),
    #[error("Client creation failed: {0}")]
    ClientCreateError(#[from] ClientCreateError),
    #[error("Node wait failed: {0}")]
    WaitFailure(#[from] NodeWaitFailure),
    #[error("Loan failed: {0}")]
    LoanError(#[from] LoanError),
    #[error("Request send failed: {0}")]
    ReqSendError(#[from] RequestSendError),
    #[error("Receive failed: {0}")]
    ReceiveError(#[from] ReceiveError),
    #[error("Send failed: {0}")]
    SendError(#[from] SendError),
    #[error("Operation failed: {0}")]
    OperationError(String),
}

/// Builder for creating a `TopologyUnawareStbNode` with configurable parameters
#[derive(Debug, Clone)]
pub struct StbNodeBuilder {
    id: usize,
    neighbors: Vec<usize>,
    config: Config,
    max_connection_attempts: u64,
    connection_retry_interval: Duration,
}

impl StbNodeBuilder {
    /// Creates a new builder for a node with the specified ID
    pub fn new(id: usize) -> Self {
        Self {
            id,
            neighbors: Vec::new(),
            config: Config::default(),
            max_connection_attempts: 10,
            connection_retry_interval: Duration::from_millis(100),
        }
    }

    /// Sets the neighborhood of the node
    pub fn with_neighbors(mut self, neighbors: Vec<usize>) -> Self {
        self.neighbors = neighbors;
        self
    }

    /// Sets the node configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Sets the maximum connection attempts for neighbors
    pub fn with_max_connection_attempts(mut self, attempts: u64) -> Self {
        self.max_connection_attempts = attempts;
        self
    }

    /// Sets the retry interval between connection attempts
    pub fn with_connection_retry_interval(mut self, interval: Duration) -> Self {
        self.connection_retry_interval = interval;
        self
    }

    /// Generates a service name from prefix and node ID
    fn service_name(prefix: &str, id: usize) -> Result<ServiceName, ServiceNameError> {
        format!("{prefix}/Node{id}").as_str().try_into()
    }

    /// Builds a `TopologyUnawareStbNode` with the configured parameters
    pub fn build<const NUM_NODES: usize>(
        mut self,
    ) -> Result<TopologyUnawareStbNode<NUM_NODES>, StbNodeError> {
        let node = NodeBuilder::new()
            .config(&self.config)
            .create::<ipc::Service>()?;

        let service_name = Self::service_name("Probecho/UnawareStb", self.id)?;
        let service = node
            .service_builder(&service_name)
            .request_response::<TopologyReq<NUM_NODES>, TopologyResp<NUM_NODES>>()
            .request_user_header::<SimpleHeader>()
            .response_user_header::<SimpleHeader>()
            .max_servers(1)
            .max_clients(self.neighbors.len())
            .create()?;

        let server = service.server_builder().create()?;

        // Sort neighbors for efficient lookups
        self.neighbors.sort_unstable();
        let neighbors = self
            .neighbors
            .iter()
            .map(|&n| {
                self.connect_to_neighbor::<NUM_NODES>(&node, n)
                    .map_err(|e| {
                        StbNodeError::OperationError(format!(
                            "Failed connecting to neighbor {}: {}",
                            n, e
                        ))
                    })
            })
            .collect::<Result<_, _>>()?;

        Ok(TopologyUnawareStbNode {
            id: self.id,
            node,
            server,
            neighbors,
            state: UnawareState::WaitingGetReq,
        })
    }

    /// Connects to a neighbor node with retry logic
    fn connect_to_neighbor<const NUM_NODES: usize>(
        &self,
        node: &Node<ipc::Service>,
        neighbor_id: usize,
    ) -> Result<TopologyUnawareStbNeighbor<NUM_NODES>, StbNodeError> {
        let service_name = Self::service_name("Probecho/UnawareStb", neighbor_id)?;

        for attempt in 0..self.max_connection_attempts {
            match node
                .service_builder(&service_name)
                .request_response::<TopologyReq<NUM_NODES>, TopologyResp<NUM_NODES>>()
                .request_user_header::<SimpleHeader>()
                .response_user_header::<SimpleHeader>()
                .open()
            {
                Ok(service) => {
                    let client = service.client_builder().create()?;
                    return Ok(TopologyUnawareStbNeighbor {
                        id: neighbor_id,
                        client,
                    });
                }
                Err(e) if attempt == self.max_connection_attempts - 1 => {
                    return Err(e.into());
                }
                _ => {
                    node.wait(self.connection_retry_interval)?;
                }
            }
        }

        Err(StbNodeError::OperationError(
            "Connection attempts exhausted".into(),
        ))
    }
}

/// State machine for topology discovery
#[derive(Debug)]
enum UnawareState<const NUM_NODES: usize> {
    /// Waiting for initial topology request
    WaitingGetReq,
    /// Gathering local topology from neighbors
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
        curr_topology: MemShareableAdjMatrix<NUM_NODES>,
    },
    /// Waiting for global topology broadcast
    WaitingGlobalTopology,
    /// Received and stored global topology
    ReceivedGlobalTopology {
        topology: MemShareableAdjMatrix<NUM_NODES>,
    },
}

impl<const NUM_NODES: usize> UnawareState<NUM_NODES> {
    /// Returns true if topology discovery is complete
    fn is_complete(&self) -> bool {
        matches!(self, Self::ReceivedGlobalTopology { .. })
    }
}

/// Merges two topology matrices using bitwise OR operation
fn merge_topologies<const N: usize>(
    base: &mut MemShareableAdjMatrix<N>,
    addition: &MemShareableAdjMatrix<N>,
) {
    for i in 0..N {
        for j in 0..N {
            base[i][j] |= addition[i][j];
        }
    }
}

/// Node in topology discovery phase
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
    neighbors: Vec<TopologyUnawareStbNeighbor<NUM_NODES>>,
    state: UnawareState<NUM_NODES>,
}

impl<const NUM_NODES: usize> TopologyUnawareStbNode<NUM_NODES> {
    /// Get the node identifier.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Runs the node as topology discovery initiator
    pub fn run_initiator<Req, ReqHeader, Resp, RespHeader>(
        mut self,
    ) -> Result<TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, NUM_NODES>, StbNodeError>
    where
        Req: fmt::Debug + ZeroCopySend,
        ReqHeader: fmt::Debug + ZeroCopySend,
        Resp: fmt::Debug + ZeroCopySend,
        RespHeader: fmt::Debug + ZeroCopySend,
    {
        self.request_topology(None)?;
        self.run()
    }

    /// Runs the node as topology discovery participant
    pub fn run<Req, ReqHeader, Resp, RespHeader>(
        mut self,
    ) -> Result<TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, NUM_NODES>, StbNodeError>
    where
        Req: fmt::Debug + ZeroCopySend,
        ReqHeader: fmt::Debug + ZeroCopySend,
        Resp: fmt::Debug + ZeroCopySend,
        RespHeader: fmt::Debug + ZeroCopySend,
    {
        while !self.state.is_complete() {
            self.handle_request()?;
            self.gather_topology()?;
        }

        let topology = match &self.state {
            UnawareState::ReceivedGlobalTopology { topology } => topology.clone(),
            _ => return Err(StbNodeError::OperationError("Invalid final state".into())),
        };

        self.transition_to_aware_node(topology)
    }

    /// Requests topology from neighbors
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

        for neighbor in &self.neighbors {
            let mut sample = neighbor.client.loan_uninit()?;

            sample.user_header_mut().src_id = self.id;
            sample.user_header_mut().dst_id = neighbor.id;

            let sample = sample.write_payload(TopologyReq::Get);
            pending_responses.push(sample.send()?);
        }

        self.state = UnawareState::GatheringLocalTopology {
            origin_req,
            pending_responses,
            curr_topology: MemShareableAdjMatrix::default(),
        };
        Ok(())
    }

    /// Handles incoming requests
    fn handle_request(&mut self) -> Result<(), StbNodeError> {
        let Some(req) = self.server.receive()? else {
            return Ok(());
        };

        // Clone payload to avoid borrow issues
        let payload = req.payload().clone();
        match payload {
            TopologyReq::Get => self.handle_get_request(req),
            TopologyReq::FullTopology(t) => self.handle_full_topology(req, t),
        }
    }

    /// Handles GET requests based on current state
    fn handle_get_request(
        &mut self,
        req: ActiveRequest<
            ipc::Service,
            TopologyReq<NUM_NODES>,
            SimpleHeader,
            TopologyResp<NUM_NODES>,
            SimpleHeader,
        >,
    ) -> Result<(), StbNodeError> {
        match &self.state {
            UnawareState::WaitingGetReq => {
                // Only process GET requests in initial state
                self.request_topology(Some(req))?;
            }
            _ => {
                // In all other states, respond with empty topology
                self.send_empty_topology_response(req)?;
            }
        }
        Ok(())
    }

    /// Handles full topology broadcasts in any state
    fn handle_full_topology(
        &mut self,
        req: ActiveRequest<
            ipc::Service,
            TopologyReq<NUM_NODES>,
            SimpleHeader,
            TopologyResp<NUM_NODES>,
            SimpleHeader,
        >,
        topology: MemShareableAdjMatrix<NUM_NODES>,
    ) -> Result<(), StbNodeError> {
        // Always process full topology requests regardless of state
        self.state = UnawareState::ReceivedGlobalTopology {
            topology: topology.clone(),
        };
        self.broadcast_topology()?;

        // Always respond to confirm receipt
        let dst_id = req.user_header().src_id;
        let mut response = req.loan_uninit()?;
        response.user_header_mut().src_id = self.id;
        response.user_header_mut().dst_id = dst_id;
        let response = response.write_payload(TopologyResp::Stored);
        response.send()?;

        Ok(())
    }

    /// Sends empty topology response
    fn send_empty_topology_response(
        &self,
        req: ActiveRequest<
            ipc::Service,
            TopologyReq<NUM_NODES>,
            SimpleHeader,
            TopologyResp<NUM_NODES>,
            SimpleHeader,
        >,
    ) -> Result<(), StbNodeError> {
        let dst_id = req.user_header().src_id;
        let mut response = req.loan_uninit()?;
        response.user_header_mut().src_id = self.id;
        response.user_header_mut().dst_id = dst_id;
        let response = response.write_payload(TopologyResp::LocalTopology(None));
        response.send()?;
        Ok(())
    }

    /// Processes topology gathering in GatheringLocalTopology state
    fn gather_topology(&mut self) -> Result<(), StbNodeError> {
        let (origin_req, mut pending_responses, mut local_topology) = match &mut self.state {
            UnawareState::GatheringLocalTopology {
                origin_req,
                pending_responses,
                curr_topology,
            } => (
                origin_req.take(),
                std::mem::take(pending_responses),
                curr_topology.clone(),
            ),
            _ => return Ok(()),
        };

        let mut new_pending_responses = Vec::new();
        let mut all_responses_received = true;

        for pending in pending_responses.drain(..) {
            match pending.receive()? {
                Some(response) => {
                    if let TopologyResp::LocalTopology(Some(t)) = response.payload() {
                        merge_topologies(&mut local_topology, &t);
                    }
                }
                None => {
                    new_pending_responses.push(pending);
                    all_responses_received = false;
                }
            }
        }

        if !all_responses_received {
            // Restore state for next attempt
            self.state = UnawareState::GatheringLocalTopology {
                origin_req,
                pending_responses: new_pending_responses,
                curr_topology: local_topology,
            };
            return Ok(());
        }

        match origin_req {
            Some(req) => {
                // Respond to origin request
                let dst_id = req.user_header().src_id;
                let mut response = req.loan_uninit()?;
                response.user_header_mut().src_id = self.id;
                response.user_header_mut().dst_id = dst_id;
                let response =
                    response.write_payload(TopologyResp::LocalTopology(Some(local_topology)));
                response.send()?;

                self.state = UnawareState::WaitingGlobalTopology;
            }
            None => {
                // We're the initiator
                self.state = UnawareState::ReceivedGlobalTopology {
                    topology: local_topology,
                };
                self.broadcast_topology()?;
            }
        }

        Ok(())
    }

    /// Broadcasts global topology to neighbors
    fn broadcast_topology(&self) -> Result<(), StbNodeError> {
        let topology = match &self.state {
            UnawareState::ReceivedGlobalTopology { topology } => topology,
            _ => {
                return Err(StbNodeError::OperationError(
                    "Cannot broadcast without complete topology".into(),
                ));
            }
        };

        for neighbor in &self.neighbors {
            let mut sample = neighbor.client.loan_uninit()?;
            sample.user_header_mut().src_id = self.id;
            sample.user_header_mut().dst_id = neighbor.id;
            let sample = sample.write_payload(TopologyReq::FullTopology(topology.clone()));
            sample.send()?;
        }
        Ok(())
    }

    /// Transitions to topology-aware operation mode
    fn transition_to_aware_node<Req, ReqHeader, Resp, RespHeader>(
        self,
        topology: MemShareableAdjMatrix<NUM_NODES>,
    ) -> Result<TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, NUM_NODES>, StbNodeError>
    where
        Req: fmt::Debug + ZeroCopySend,
        ReqHeader: fmt::Debug + ZeroCopySend,
        Resp: fmt::Debug + ZeroCopySend,
        RespHeader: fmt::Debug + ZeroCopySend,
    {
        let service_name = StbNodeBuilder::service_name("Probecho/AwareStb", self.id)?;
        let service = self
            .node
            .service_builder(&service_name)
            .request_response::<Req, Resp>()
            .request_user_header::<ReqHeader>()
            .response_user_header::<RespHeader>()
            .max_servers(1)
            .max_clients(self.neighbors.len())
            .create()?;

        let server = service.server_builder().create()?;

        let mut aware_neighbors = Vec::with_capacity(self.neighbors.len());
        for neighbor in &self.neighbors {
            let service_name = StbNodeBuilder::service_name("Probecho/AwareStb", neighbor.id)?;
            let service =
                self.connect_to_aware_neighbor::<Req, ReqHeader, Resp, RespHeader>(&service_name)?;
            let client = service.client_builder().create()?;
            aware_neighbors.push(TopologyAwareStbNeighbor {
                id: neighbor.id,
                client,
            });
        }

        let shortest_path_next = Self::compute_shortest_paths(self.id, &topology);

        Ok(TopologyAwareStbNode {
            id: self.id,
            node: self.node,
            server,
            neighbors: aware_neighbors,
            shortest_path_next,
        })
    }

    /// Connects to a neighbor in aware phase
    fn connect_to_aware_neighbor<Req, ReqHeader, Resp, RespHeader>(
        &self,
        service_name: &ServiceName,
    ) -> Result<PortFactory<ipc::Service, Req, ReqHeader, Resp, RespHeader>, StbNodeError>
    where
        Req: fmt::Debug + ZeroCopySend,
        ReqHeader: fmt::Debug + ZeroCopySend,
        Resp: fmt::Debug + ZeroCopySend,
        RespHeader: fmt::Debug + ZeroCopySend,
    {
        for attempt in 0..10 {
            match self
                .node
                .service_builder(service_name)
                .request_response::<Req, Resp>()
                .request_user_header::<ReqHeader>()
                .response_user_header::<RespHeader>()
                .open()
            {
                Ok(service) => return Ok(service),
                Err(e) if attempt == 9 => return Err(e.into()),
                _ => self.node.wait(Duration::from_millis(100))?,
            }
        }
        Err(StbNodeError::OperationError(
            "Connection attempts exhausted".into(),
        ))
    }

    /// Computes next hops for shortest paths using BFS
    fn compute_shortest_paths(
        src: usize,
        adj: &MemShareableAdjMatrix<NUM_NODES>,
    ) -> [Option<usize>; NUM_NODES] {
        let mut dist = [None; NUM_NODES];
        let mut next_node = [None; NUM_NODES];
        let mut queue = VecDeque::new();

        dist[src] = Some(0);
        queue.push_back(src);

        while let Some(current) = queue.pop_front() {
            for neighbor in 0..NUM_NODES {
                if !adj[current][neighbor] {
                    continue;
                }

                if dist[neighbor].is_none() {
                    dist[neighbor] = Some(dist[current].unwrap() + 1);
                    next_node[neighbor] = if current == src {
                        Some(neighbor)
                    } else {
                        next_node[current]
                    };
                    queue.push_back(neighbor);
                }
            }
        }
        next_node
    }
}

/// Neighbor connection during topology discovery phase
struct TopologyUnawareStbNeighbor<const NUM_NODES: usize> {
    id: usize,
    client: Client<
        ipc::Service,
        TopologyReq<NUM_NODES>,
        SimpleHeader,
        TopologyResp<NUM_NODES>,
        SimpleHeader,
    >,
}

/// Node with full topology awareness
pub struct TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, const NUM_NODES: usize>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    id: usize,
    node: Node<ipc::Service>,
    server: Server<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
    shortest_path_next: [Option<usize>; NUM_NODES],
    neighbors: Vec<TopologyAwareStbNeighbor<Req, ReqHeader, Resp, RespHeader>>,
}

impl<Req, ReqHeader, Resp, RespHeader, const NUM_NODES: usize>
    TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, NUM_NODES>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    /// Returns the node identifier.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Tries to receive a message.
    pub fn receive(
        &self,
    ) -> Result<Option<StbActiveRequest<Req, ReqHeader, Resp, RespHeader>>, StbNodeError> {
        match self.server.receive() {
            Ok(Some(req)) => Ok(Some(StbActiveRequest { inner: req })),
            Ok(None) => Ok(None),
            Err(e) => Err(StbNodeError::ReceiveError(e)),
        }
    }

    /// Make the node sleep for the given `time`.
    pub fn wait(&self, time: Duration) -> Result<(), StbNodeError> {
        self.node.wait(time).map_err(StbNodeError::WaitFailure)
    }

    /// Sends a messag to the the node with id `dst_id`.
    pub fn send_to_neighboor(
        &self,
        dst_id: &usize,
        data: Req,
        header: ReqHeader,
    ) -> Result<StbPendingResponse<Req, ReqHeader, Resp, RespHeader>, StbNodeError> {
        let neigh = self.neighbors.binary_search_by_key(dst_id, |n| n.id);
        let client = match neigh {
            Ok(idx) => &self.neighbors[idx].client,
            Err(_) => {
                return Err(StbNodeError::OperationError(format!(
                    "Could not find a neighboor of {} with id {}",
                    self.id, dst_id
                )));
            }
        };

        let mut request = client.loan_uninit().map_err(StbNodeError::LoanError)?;
        *request.user_header_mut() = header;
        let request = request.write_payload(data);
        let pending_resp = request.send().map_err(StbNodeError::ReqSendError)?;
        Ok(StbPendingResponse {
            inner: pending_resp,
        })
    }
}

impl<Req, ReqHeader, Resp, RespHeader, const NUM_NODES: usize>
    TopologyAwareStbNode<Req, ReqHeader, Resp, RespHeader, NUM_NODES>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend + DestinationHeader,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    /// Sends a message to the destination prescribed on the header.
    /// In this case the destination does not need to be neighboor of the node.
    pub fn send(
        &self,
        data: Req,
        header: ReqHeader,
    ) -> Result<StbPendingResponse<Req, ReqHeader, Resp, RespHeader>, StbNodeError> {
        let dst = header.destination();

        if let Some(next_node) = &self.shortest_path_next[dst] {
            self.send_to_neighboor(next_node, data, header)
        } else {
            let err = if dst == self.id {
                StbNodeError::OperationError("Can't send message to itself!".to_owned())
            } else {
                StbNodeError::OperationError(format!(
                    "Node {} is unreachable from node {}",
                    dst, self.id
                ))
            };
            Err(err)
        }
    }
}

/// An request that still on-going.
pub struct StbActiveRequest<Req, ReqHeader, Resp, RespHeader>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    inner: ActiveRequest<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
}

impl<Req, ReqHeader, Resp, RespHeader> StbActiveRequest<Req, ReqHeader, Resp, RespHeader>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    /// Gets the header of the message.
    pub fn header(&self) -> &ReqHeader {
        self.inner.user_header()
    }

    /// Gets the data of the message.
    pub fn data(&self) -> &Req {
        self.inner.payload()
    }

    /// Reply the request.
    pub fn reply(&self, payload: Resp, header: RespHeader) -> Result<(), StbNodeError> {
        let mut response = self.inner.loan_uninit().map_err(StbNodeError::LoanError)?;

        *response.user_header_mut() = header;
        let response = response.write_payload(payload);

        response.send().map_err(StbNodeError::SendError)
    }
}

/// A pending response.
pub struct StbPendingResponse<Req, ReqHeader, Resp, RespHeader>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    inner: PendingResponse<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
}

impl<Req, ReqHeader, Resp, RespHeader> StbPendingResponse<Req, ReqHeader, Resp, RespHeader>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    /// Tries to receive the pending response.
    pub fn receive(&self) -> Result<Option<StbRespData<Resp, RespHeader>>, StbNodeError> {
        match self.inner.receive() {
            Ok(Some(resp)) => Ok(Some(StbRespData { inner: resp })),
            Ok(None) => Ok(None),
            Err(e) => Err(StbNodeError::ReceiveError(e)),
        }
    }
}

/// The response data of an request.
pub struct StbRespData<Resp, RespHeader>
where
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    inner: Response<ipc::Service, Resp, RespHeader>,
}

impl<Resp, RespHeader> StbRespData<Resp, RespHeader>
where
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    /// Returns a reference to the header of the response.
    pub fn header(&self) -> &RespHeader {
        self.inner.user_header()
    }

    /// Returns a reference to the data of the response.
    pub fn data(&self) -> &Resp {
        self.inner.payload()
    }
}

/// Neighbor connection in topology-aware phase
struct TopologyAwareStbNeighbor<Req, ReqHeader, Resp, RespHeader>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    id: usize,
    client: Client<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
}
