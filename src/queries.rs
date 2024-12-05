use async_trait::async_trait;
use cqrs_es::persist::GenericQuery;
use cqrs_es::{EventEnvelope, Query, View};
use serde::{Deserialize, Serialize};
use sqlite_es::SqliteViewRepository;

use crate::deployment::{Deployment, DeploymentEvent};

pub struct SimpleLoggingQuery {}

// Our simplest query, this is great for debugging but absolutely useless in production.
// This query just pretty prints the events as they are processed.
#[async_trait]
impl Query<Deployment> for SimpleLoggingQuery {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<Deployment>]) {
        for event in events {
            println!("{}-{}\n{:?}", aggregate_id, event.sequence, &event.payload);
        }
    }
}

// Our second query, this one will be handled with an SQLite `GenericQuery`
// which will serialize and persist our view after it is updated. It also
// provides a `load` method to deserialize the view on request.
pub type DeploymentQuery =
    GenericQuery<SqliteViewRepository<DeploymentView, Deployment>, DeploymentView, Deployment>;

// The view for Deployment query, for a standard http application this should
// be designed to reflect the response dto that will be returned to a user.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeploymentView {
    deployment_id: Option<String>,
    status: Option<String>,
}

// This updates the view with events as they are committed.
impl View<Deployment> for DeploymentView {
    fn update(&mut self, event: &EventEnvelope<Deployment>) {
        match &event.payload {
            DeploymentEvent::DeploymentSubmited {
                status,
                deployment_id,
            } => {
                self.deployment_id = Some(deployment_id.clone());
                self.status = Some(status.clone());
            }

            DeploymentEvent::ManifestValidated {
                deployment_id,
                status,
            } => {
                self.deployment_id = Some(deployment_id.clone());
                self.status = Some(status.clone());
            }
            DeploymentEvent::DeploymentProcessed {
                deployment_id,
                status,
            } => {
                self.deployment_id = Some(deployment_id.clone());
                self.status = Some(status.clone());
            }
            DeploymentEvent::Deployed {
                deployment_id,
                status,
            } => {
                self.deployment_id = Some(deployment_id.clone());
                self.status = Some(status.clone());
            }
            DeploymentEvent::DeploymentCanceled {
                deployment_id,
                status,
            } => {
                self.deployment_id = Some(deployment_id.clone());
                self.status = Some(status.clone());
            }
        }
    }
}
