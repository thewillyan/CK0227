use std::{fmt, time::Duration};

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
        builder::request_response::{
            RequestResponseCreateError, RequestResponseOpenError, RequestResponseOpenOrCreateError,
        },
        port_factory::{
            client::ClientCreateError, server::ServerCreateError,
        },
        service_name::ServiceNameError,
    },
};
use thiserror::Error;

/// Represents all possible failures when building or operating an NSB node
#[derive(Debug, Error)]
pub enum NsbNodeError {
    #[error("Node creation failed: {0}")]
    NodeCreationFailure(#[from] NodeCreationFailure),
    #[error("Invalid service name: {0}")]
    InvalidServiceName(#[from] ServiceNameError),
    #[error("Service opening or creation failed: {0}")]
    ServiceOpenOrCreateError(#[from] RequestResponseOpenOrCreateError),
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

/// Legacy error type for backward compatibility
#[derive(Debug, Error)]
pub enum NsbNodeBuildError {
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
}

/// Builder for creating an NSB node with configurable parameters (STB-like)
#[derive(Debug, Clone)]
pub struct NsbNodeBuilder {
    id: usize,
    neighbors: Vec<usize>,
    config: Config,
    max_connection_attempts: u64,
    connection_retry_interval: Duration,
}

impl NsbNodeBuilder {
    /// Creates a new `NsbNodeBuilder`.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            neighbors: Vec::new(),
            config: Config::default(),
            max_connection_attempts: 10,
            connection_retry_interval: Duration::from_millis(100),
        }
    }

    /// Sets the neighborhood of the node.
    pub fn with_neighbors(mut self, neighbors: Vec<usize>) -> Self {
        self.neighbors = neighbors;
        self
    }

    /// Sets the node config.
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

    /// Get service name for a node of given `id`.
    fn service_name(id: usize) -> Result<ServiceName, NsbNodeError> {
        format!("Probecho/Nsb/Node{}", id)
            .as_str()
            .try_into()
            .map_err(NsbNodeError::InvalidServiceName)
    }

    /// Build an new `NsbNode` with header types.
    pub fn build<Req, ReqHead, Resp, RespHead>(
        self,
    ) -> Result<NsbNode<Req, ReqHead, Resp, RespHead>, NsbNodeError>
    where
        Req: ZeroCopySend + fmt::Debug + Clone,
        ReqHead: ZeroCopySend + fmt::Debug + Clone,
        Resp: ZeroCopySend + fmt::Debug + Clone,
        RespHead: ZeroCopySend + fmt::Debug + Clone,
    {
        self.build_with_header::<Req, ReqHead, Resp, RespHead>()
    }

    /// Build a new `NsbNode` with header types using STB-like connection logic.
    pub fn build_with_header<Req, ReqHead, Resp, RespHead>(
        self,
    ) -> Result<NsbNode<Req, ReqHead, Resp, RespHead>, NsbNodeError>
    where
        Req: ZeroCopySend + fmt::Debug,
        ReqHead: ZeroCopySend + fmt::Debug,
        Resp: ZeroCopySend + fmt::Debug,
        RespHead: ZeroCopySend + fmt::Debug,
    {
        let node = NodeBuilder::new()
            .config(&self.config)
            .create::<ipc::Service>()?;

        let service = node
            .service_builder(&Self::service_name(self.id)?)
            .request_response::<Req, Resp>()
            .request_user_header::<ReqHead>()
            .response_user_header::<RespHead>()
            .max_servers(1)
            .max_clients(self.neighbors.len())
            .create()?;

        let server = service.server_builder().create()?;

        let mut neighbors = Vec::with_capacity(self.neighbors.len());
        let neighbor_ids = self.neighbors.clone(); // Clone to avoid borrow issues
        
        for n in neighbor_ids {
            let neighbor_client = self.connect_to_neighbor::<Req, ReqHead, Resp, RespHead>(&node, n)?;
            neighbors.push(neighbor_client);
        }

        // Sort neighbors for efficient lookups (STB-like)
        neighbors.sort_by_key(|neighbor| neighbor.neigh_id);

        Ok(NsbNode {
            id: self.id,
            node,
            server,
            neighbors,
        })
    }

    /// Connects to a neighbor node with retry logic (STB-like)
    fn connect_to_neighbor<Req, ReqHead, Resp, RespHead>(
        &self,
        node: &Node<ipc::Service>,
        neighbor_id: usize,
    ) -> Result<NsbNeighborClient<Req, ReqHead, Resp, RespHead>, NsbNodeError>
    where
        Req: ZeroCopySend + fmt::Debug,
        ReqHead: ZeroCopySend + fmt::Debug,
        Resp: ZeroCopySend + fmt::Debug,
        RespHead: ZeroCopySend + fmt::Debug,
    {
        let service_name = Self::service_name(neighbor_id)?;

        for attempt in 0..self.max_connection_attempts {
            let service_result = node
                .service_builder(&service_name)
                .request_response::<Req, Resp>()
                .request_user_header::<ReqHead>()
                .response_user_header::<RespHead>()
                .open();

            match service_result {
                Ok(service) => {
                    let client = service.client_builder().create()?;
                    return Ok(NsbNeighborClient {
                        neigh_id: neighbor_id,
                        client,
                    });
                }
                Err(_) if attempt + 1 < self.max_connection_attempts => {
                    // Not the last attempt, wait and retry
                    let _ = node.wait(self.connection_retry_interval);
                    continue;
                }
                Err(e) => {
                    return Err(NsbNodeError::ServiceOpenError(e));
                }
            }
        }

        Err(NsbNodeError::OperationError(
            "Connection attempts exhausted".into(),
        ))
    }
}

/// A node that implements broadcast using neighbor sets with STB-like patterns
pub struct NsbNode<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + fmt::Debug,
    ReqHead: ZeroCopySend + fmt::Debug,
    Resp: ZeroCopySend + fmt::Debug,
    RespHead: ZeroCopySend + fmt::Debug,
{
    id: usize,
    node: Node<ipc::Service>,
    server: Server<ipc::Service, Req, ReqHead, Resp, RespHead>,
    neighbors: Vec<NsbNeighborClient<Req, ReqHead, Resp, RespHead>>,
}

impl<Req, ReqHead, Resp, RespHead> NsbNode<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + fmt::Debug + Clone,
    ReqHead: ZeroCopySend + fmt::Debug + Clone,
    Resp: ZeroCopySend + fmt::Debug + Clone,
    RespHead: ZeroCopySend + fmt::Debug + Clone,
{
    /// Returns the node id.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Return the iceoryx2 inner Node.
    pub fn inner_node(&self) -> &Node<ipc::Service> {
        &self.node
    }

    /// Returns the server service provided by the node.
    pub fn server(&self) -> &Server<ipc::Service, Req, ReqHead, Resp, RespHead> {
        &self.server
    }

    /// Return all neighbor clients corresponding to all neighbors of the node.
    pub fn neighbors(&self) -> &[NsbNeighborClient<Req, ReqHead, Resp, RespHead>] {
        &self.neighbors
    }

    /// Return the neighbor client corresponding to neighbor of id `dest_id`.
    /// Returns `None` if no neighbor of id `dest_id` was found.
    pub fn neighbor(
        &self,
        dest_id: &usize,
    ) -> Option<&NsbNeighborClient<Req, ReqHead, Resp, RespHead>> {
        let neighbor_idx = self
            .neighbors
            .binary_search_by_key(dest_id, |neighbor| neighbor.neigh_id)
            .ok()?;
        Some(&self.neighbors[neighbor_idx])
    }

    /// Wait for a duration
    pub fn wait(&self, interval: Duration) -> Result<(), NsbNodeError> {
        self.node.wait(interval)?;
        Ok(())
    }

    /// Run the node as a regular participant (not initiator)
    pub fn run(self) -> Result<TopologyAwareNsbNode<Req, ReqHead, Resp, RespHead>, NsbNodeError> {
        Ok(TopologyAwareNsbNode { inner: self })
    }

    /// Run the node as the initiator of the broadcast
    pub fn run_initiator(self) -> Result<TopologyAwareNsbNode<Req, ReqHead, Resp, RespHead>, NsbNodeError> {
        Ok(TopologyAwareNsbNode { inner: self })
    }
}

/// Wrapper for received messages with STB-like interface
pub struct NsbReceivedMessage<Req, ReqHead> {
    data: Req,
    header: ReqHead,
}

impl<Req, ReqHead> NsbReceivedMessage<Req, ReqHead> {
    pub fn data(&self) -> &Req {
        &self.data
    }
    
    pub fn header(&self) -> &ReqHead {
        &self.header
    }
}

/// An active request that is still ongoing (STB-like)
pub struct NsbActiveRequest<Req, ReqHeader, Resp, RespHeader>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    inner: ActiveRequest<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
}

impl<Req, ReqHeader, Resp, RespHeader> NsbActiveRequest<Req, ReqHeader, Resp, RespHeader>
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

    /// Reply to the request.
    pub fn reply(&self, payload: Resp, header: RespHeader) -> Result<(), NsbNodeError> {
        let mut response = self.inner.loan_uninit()?;
        *response.user_header_mut() = header;
        let response = response.write_payload(payload);
        response.send()?;
        Ok(())
    }
}

/// A pending response (STB-like)
pub struct NsbPendingResponse<Req, ReqHeader, Resp, RespHeader>
where
    Req: fmt::Debug + ZeroCopySend,
    ReqHeader: fmt::Debug + ZeroCopySend,
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    inner: PendingResponse<ipc::Service, Req, ReqHeader, Resp, RespHeader>,
}

impl<Req, ReqHeader, Resp, RespHeader> NsbPendingResponse<Req, ReqHeader, Resp, RespHeader>
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

    /// Tries to receive the pending response.
    pub fn receive(&self) -> Result<Option<NsbRespData<Resp, RespHeader>>, NsbNodeError> {
        match self.inner.receive() {
            Ok(Some(resp)) => Ok(Some(NsbRespData { inner: resp })),
            Ok(None) => Ok(None),
            Err(e) => Err(NsbNodeError::ReceiveError(e)),
        }
    }
}

/// The response data of a request (STB-like)
pub struct NsbRespData<Resp, RespHeader>
where
    Resp: fmt::Debug + ZeroCopySend,
    RespHeader: fmt::Debug + ZeroCopySend,
{
    inner: Response<ipc::Service, Resp, RespHeader>,
}

impl<Resp, RespHeader> NsbRespData<Resp, RespHeader>
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

/// A topology-aware NSB node with STB-like request-response patterns
pub struct TopologyAwareNsbNode<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + fmt::Debug + Clone,
    ReqHead: ZeroCopySend + fmt::Debug + Clone,
    Resp: ZeroCopySend + fmt::Debug + Clone,
    RespHead: ZeroCopySend + fmt::Debug + Clone,
{
    inner: NsbNode<Req, ReqHead, Resp, RespHead>,
}

impl<Req, ReqHead, Resp, RespHead> TopologyAwareNsbNode<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + fmt::Debug + Clone,
    ReqHead: ZeroCopySend + fmt::Debug + Clone,
    Resp: ZeroCopySend + fmt::Debug + Clone,
    RespHead: ZeroCopySend + fmt::Debug + Clone,
{
    /// Returns the node identifier.
    pub fn id(&self) -> usize {
        self.inner.id()
    }

    /// Send a message to a specific neighbor using STB-like patterns
    pub fn send<F>(&self, data: Req, neighbor_id: usize, header_fn: F) -> Result<(), NsbNodeError>
    where
        F: FnOnce(usize, usize) -> ReqHead,
    {
        if let Some(neighbor) = self.inner.neighbor(&neighbor_id) {
            let mut request = neighbor.client().loan_uninit()?;
            *request.user_header_mut() = header_fn(self.inner.id(), neighbor_id);
            let request = request.write_payload(data);
            request.send()?;
            Ok(())
        } else {
            Err(NsbNodeError::OperationError(format!(
                "Neighbor {} not found",
                neighbor_id
            )))
        }
    }

    /// Send a message to a specific neighbor and get a pending response (STB-like)
    pub fn send_with_response<F>(
        &self, 
        data: Req, 
        neighbor_id: usize, 
        header_fn: F
    ) -> Result<NsbPendingResponse<Req, ReqHead, Resp, RespHead>, NsbNodeError>
    where
        F: FnOnce(usize, usize) -> ReqHead,
    {
        if let Some(neighbor) = self.inner.neighbor(&neighbor_id) {
            let mut request = neighbor.client().loan_uninit()?;
            *request.user_header_mut() = header_fn(self.inner.id(), neighbor_id);
            let request = request.write_payload(data);
            let pending_resp = request.send()?;
            Ok(NsbPendingResponse { inner: pending_resp })
        } else {
            Err(NsbNodeError::OperationError(format!(
                "Neighbor {} not found",
                neighbor_id
            )))
        }
    }

    /// Try to receive a message (backward compatible - returns simple message)
    pub fn receive(&self) -> Result<Option<NsbReceivedMessage<Req, ReqHead>>, NsbNodeError> {
        self.receive_simple()
    }

    /// Try to receive a message (now returns an ActiveRequest for STB-like patterns)
    pub fn receive_request(&self) -> Result<Option<NsbActiveRequest<Req, ReqHead, Resp, RespHead>>, NsbNodeError> {
        match self.inner.server().receive()? {
            Some(req) => Ok(Some(NsbActiveRequest { inner: req })),
            None => Ok(None),
        }
    }

    /// Try to receive a message (simple version for backward compatibility)
    pub fn receive_simple(&self) -> Result<Option<NsbReceivedMessage<Req, ReqHead>>, NsbNodeError> {
        match self.inner.server().receive()? {
            Some(req) => {
                let data = req.payload().clone();
                let header = req.user_header().clone();
                Ok(Some(NsbReceivedMessage { data, header }))
            },
            None => Ok(None),
        }
    }

    /// Wait for a duration
    pub fn wait(&self, interval: Duration) -> Result<(), NsbNodeError> {
        self.inner.wait(interval)
    }
}

/// A neighbor client of an NSB node (STB-like)
pub struct NsbNeighborClient<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + fmt::Debug,
    ReqHead: ZeroCopySend + fmt::Debug,
    Resp: ZeroCopySend + fmt::Debug,
    RespHead: ZeroCopySend + fmt::Debug,
{
    neigh_id: usize,
    client: Client<ipc::Service, Req, ReqHead, Resp, RespHead>,
}

impl<Req, ReqHead, Resp, RespHead> NsbNeighborClient<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + fmt::Debug,
    ReqHead: ZeroCopySend + fmt::Debug,
    Resp: ZeroCopySend + fmt::Debug,
    RespHead: ZeroCopySend + fmt::Debug,
{
    /// Returns the neighbor id.
    pub fn id(&self) -> usize {
        self.neigh_id
    }

    /// Returns the client service provided by the neighbor.
    pub fn client(&self) -> &Client<ipc::Service, Req, ReqHead, Resp, RespHead> {
        &self.client
    }
}
