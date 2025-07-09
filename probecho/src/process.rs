use std::time::Duration;

use iceoryx2::{
    node::{NodeCreationFailure, NodeWaitFailure},
    port::{client::Client, server::Server},
    prelude::*,
    service::{
        builder::request_response::{RequestResponseCreateError, RequestResponseOpenError},
        port_factory::{client::ClientCreateError, server::ServerCreateError},
        service_name::ServiceNameError,
    },
};
use thiserror::Error;

/// Represents a failure when trying to build a `NsbNode`.
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

/// Builder of a `NsbNode`.
#[derive(Debug, Clone)]
pub struct NsbNodeBuilder {
    id: usize,
    neighbors: Vec<usize>,
    config: Config,
}

impl NsbNodeBuilder {
    /// Creates a new `NsbNodeBuilder`.
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
    fn service_name(id: usize) -> Result<ServiceName, NsbNodeBuildError> {
        format!("Probecho/Nsb/Node{}", id)
            .as_str()
            .try_into()
            .map_err(NsbNodeBuildError::InvalidServiceName)
    }

    /// Build an new `NsbNode`.
    pub fn build<Req, Resp>(self) -> Result<NsbNode<Req, (), Resp, ()>, NsbNodeBuildError>
    where
        Req: ZeroCopySend + std::fmt::Debug,
        Resp: ZeroCopySend + std::fmt::Debug,
    {
        self.build_with_header::<Req, (), Resp, ()>()
    }

    /// Build a new `NsbNode` with header types.
    pub fn build_with_header<Req, ReqHead, Resp, RespHead>(
        mut self,
    ) -> Result<NsbNode<Req, ReqHead, Resp, RespHead>, NsbNodeBuildError>
    where
        Req: ZeroCopySend + std::fmt::Debug,
        ReqHead: ZeroCopySend + std::fmt::Debug,
        Resp: ZeroCopySend + std::fmt::Debug,
        RespHead: ZeroCopySend + std::fmt::Debug,
    {
        let node = NodeBuilder::new()
            .config(&self.config)
            .create::<ipc::Service>()
            .map_err(NsbNodeBuildError::NodeCreationFailure)?;

        let service = node
            .service_builder(&Self::service_name(self.id)?)
            .request_response::<Req, Resp>()
            .request_user_header::<ReqHead>()
            .response_user_header::<RespHead>()
            .max_servers(1)
            .max_clients(self.neighbors.len())
            .create()
            .map_err(NsbNodeBuildError::ServiceCreateError)?;

        let server = service
            .server_builder()
            .create()
            .map_err(NsbNodeBuildError::ServerCreateError)?;

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
                    .service_builder(&Self::service_name(n)?)
                    .request_response::<Req, Resp>()
                    .request_user_header::<ReqHead>()
                    .response_user_header::<RespHead>()
                    .open()
                    .map_err(NsbNodeBuildError::ServiceOpenError);

                // If max attempts reacher "it is what it is"
                if attempts == max_attempts {
                    break service_result?;
                }

                // If not, verify if sucessfull if not, try again after waiting for `interval`
                if let Ok(s) = service_result {
                    break s;
                }
                attempts += 1;
                node.wait(interval)
                    .map_err(NsbNodeBuildError::WaitFailure)?;
            };

            let client = service
                .client_builder()
                .create()
                .map_err(NsbNodeBuildError::ClientCreateError)?;

            neighbors.push(NsbNeighboorClient {
                neigh_id: n,
                client,
            });
        }

        Ok(NsbNode {
            id: self.id,
            node,
            server,
            neighbors,
        })
    }
}

/// A node that implements `broadcast` by using neighbor sets.
pub struct NsbNode<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + std::fmt::Debug,
    ReqHead: ZeroCopySend + std::fmt::Debug,
    Resp: ZeroCopySend + std::fmt::Debug,
    RespHead: ZeroCopySend + std::fmt::Debug,
{
    id: usize,
    node: Node<ipc::Service>,
    server: Server<ipc::Service, Req, ReqHead, Resp, RespHead>,
    neighbors: Vec<NsbNeighboorClient<Req, ReqHead, Resp, RespHead>>,
}

impl<Req, ReqHead, Resp, RespHead> NsbNode<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + std::fmt::Debug,
    ReqHead: ZeroCopySend + std::fmt::Debug,
    Resp: ZeroCopySend + std::fmt::Debug,
    RespHead: ZeroCopySend + std::fmt::Debug,
{
    /// Returns the node id.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Return the iceoryx2 inner Node.
    pub fn inner_node(&self) -> &Node<ipc::Service> {
        &self.node
    }

    /// Returns the publisher service probvided by the node.
    pub fn server(&self) -> &Server<ipc::Service, Req, ReqHead, Resp, RespHead> {
        &self.server
    }

    /// Return all `NsbNeighboorClient`'s corresponding to all neighboors of the node.
    pub fn neighbors(&self) -> &[NsbNeighboorClient<Req, ReqHead, Resp, RespHead>] {
        &self.neighbors
    }

    /// Return the `NsbNeighboorClient` corresponding to neighboors of id `dest_id` of the node.
    /// Returns `None` if no neighboor of id `dest_id` was found.
    pub fn neighboor(
        &self,
        dest_id: &usize,
    ) -> Option<&NsbNeighboorClient<Req, ReqHead, Resp, RespHead>> {
        let outbox_idx = self
            .neighbors
            .binary_search_by_key(dest_id, |outbox| outbox.neigh_id)
            .ok()?;
        Some(&self.neighbors[outbox_idx])
    }
}

/// A neighboor of an node of the Nsb network.
pub struct NsbNeighboorClient<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + std::fmt::Debug,
    ReqHead: ZeroCopySend + std::fmt::Debug,
    Resp: ZeroCopySend + std::fmt::Debug,
    RespHead: ZeroCopySend + std::fmt::Debug,
{
    neigh_id: usize,
    client: Client<ipc::Service, Req, ReqHead, Resp, RespHead>,
}

impl<Req, ReqHead, Resp, RespHead> NsbNeighboorClient<Req, ReqHead, Resp, RespHead>
where
    Req: ZeroCopySend + std::fmt::Debug,
    ReqHead: ZeroCopySend + std::fmt::Debug,
    Resp: ZeroCopySend + std::fmt::Debug,
    RespHead: ZeroCopySend + std::fmt::Debug,
{
    /// Returns the neighboor id.
    pub fn id(&self) -> usize {
        self.neigh_id
    }

    /// Returns the subscriber service provided by the neighboor.
    pub fn client(&self) -> &Client<ipc::Service, Req, ReqHead, Resp, RespHead> {
        &self.client
    }
}
