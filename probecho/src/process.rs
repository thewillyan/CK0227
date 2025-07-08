use std::time::Duration;

use iceoryx2::{
    node::{NodeCreationFailure, NodeWaitFailure},
    port::{
        publisher::{Publisher, PublisherCreateError},
        subscriber::{Subscriber, SubscriberCreateError},
    },
    prelude::*,
    service::{
        builder::publish_subscribe::{
            PublishSubscribeOpenError, PublishSubscribeOpenOrCreateError,
        },
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
    ServiceOpenOrCreationFailure(PublishSubscribeOpenOrCreateError),
    #[error(transparent)]
    ServiceOpenFailure(PublishSubscribeOpenError),
    #[error(transparent)]
    PublisherCreationFailure(PublisherCreateError),
    #[error(transparent)]
    SubscriberCreationFailure(SubscriberCreateError),
    #[error(transparent)]
    WaitFailure(NodeWaitFailure),
}

/// Builder of a `NsbNode`.
#[derive(Debug, Clone)]
pub struct NsbNodeBuilder {
    id: usize,
    neighboors: Vec<usize>,
    config: Config,
}

impl NsbNodeBuilder {
    /// Creates a new `NsbNodeBuilder`.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            neighboors: Vec::new(),
            config: Config::default(),
        }
    }

    /// Sets the neighborhood of the node.
    pub fn with_neighboors(mut self, neighboors: Vec<usize>) -> Self {
        self.neighboors = neighboors;
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

    /// Build a new `NsbNode`.
    pub fn build<P, H>(mut self) -> Result<NsbNode<P, H>, NsbNodeBuildError>
    where
        P: ZeroCopySend + std::fmt::Debug + 'static,
        H: ZeroCopySend + std::fmt::Debug + 'static,
    {
        let node = NodeBuilder::new()
            .config(&self.config)
            .create::<ipc::Service>()
            .map_err(NsbNodeBuildError::NodeCreationFailure)?;

        let service = node
            .service_builder(&Self::service_name(self.id)?)
            .publish_subscribe::<P>()
            .max_publishers(self.neighboors.len())
            .max_subscribers(1)
            .user_header::<H>()
            .open_or_create()
            .map_err(NsbNodeBuildError::ServiceOpenOrCreationFailure)?;

        let inbox = service
            .subscriber_builder()
            .create()
            .map_err(NsbNodeBuildError::SubscriberCreationFailure)?;

        let mut outboxes = Vec::with_capacity(self.neighboors.len());
        // Make neighboors list ordered so binary search is possible.
        self.neighboors.sort();
        for n in self.neighboors {
            // Attempt `max_attempts` times to connect to the neighbors.
            let max_attempts: u64 = 10;
            let mut attempts: u64 = 1;
            let interval = Duration::from_millis(100);
            let service = loop {
                let service_result = node
                    .service_builder(&Self::service_name(n)?)
                    .publish_subscribe::<P>()
                    .user_header::<H>()
                    .open()
                    .map_err(NsbNodeBuildError::ServiceOpenFailure);

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

            let sender = service
                .publisher_builder()
                .create()
                .map_err(NsbNodeBuildError::PublisherCreationFailure)?;

            outboxes.push(NsbOutbox { to: n, sender });
        }

        Ok(NsbNode {
            id: self.id,
            node,
            inbox,
            outboxes,
        })
    }
}

/// A node that implements `broadcast` by using neighbor sets.
pub struct NsbNode<P, H>
where
    P: ZeroCopySend + std::fmt::Debug + 'static,
    H: ZeroCopySend + std::fmt::Debug + 'static,
{
    id: usize,
    node: Node<ipc::Service>,
    inbox: Subscriber<ipc::Service, P, H>,
    outboxes: Vec<NsbOutbox<P, H>>,
}

impl<P, H> NsbNode<P, H>
where
    P: ZeroCopySend + std::fmt::Debug + 'static,
    H: ZeroCopySend + std::fmt::Debug + 'static,
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
    pub fn inbox(&self) -> &Subscriber<ipc::Service, P, H> {
        &self.inbox
    }

    /// Return all `NsbOutbox`'es corresponding to all neighboors of the node.
    pub fn outboxes(&self) -> &[NsbOutbox<P, H>] {
        &self.outboxes
    }

    /// Return the `NsbOutbox` corresponding to neighboors of id `dest_id` of the node.
    /// Returns `None` if no neighboor of id `dest_id` was found.
    pub fn outbox(&self, dest_id: &usize) -> Option<&NsbOutbox<P, H>> {
        let outbox_idx = self
            .outboxes
            .binary_search_by_key(dest_id, |outbox| outbox.to)
            .ok()?;
        Some(&self.outboxes[outbox_idx])
    }
}

/// A neighboor of an node of the Nsb network.
pub struct NsbOutbox<P, H>
where
    P: ZeroCopySend + std::fmt::Debug + 'static,
    H: ZeroCopySend + std::fmt::Debug + 'static,
{
    to: usize,
    sender: Publisher<ipc::Service, P, H>,
}

impl<P, H> NsbOutbox<P, H>
where
    P: ZeroCopySend + std::fmt::Debug + 'static,
    H: ZeroCopySend + std::fmt::Debug + 'static,
{
    /// Returns the neighboor id.
    pub fn destination(&self) -> usize {
        self.to
    }

    /// Returns the subscriber service provided by the neighboor.
    pub fn sender(&self) -> &Publisher<ipc::Service, P, H> {
        &self.sender
    }
}
