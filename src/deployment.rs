// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
// enum DeployStatus {
//     Submitted,
//     Validated,
//     Pending,
//     Deployed,
//     Cancelled
// }

use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use cqrs_es::{Aggregate, DomainEvent, EventEnvelope, Query};
use serde::{Deserialize, Serialize};

use crate::helper::execute_task;

#[derive(Debug, Deserialize)]
pub enum DeploymentCommand {
    SubmitDeployment,
    ValidateManifest,
    ProcessDeployment,
    Deploy,
    CancelDeployment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentEvent {
    DeploymentSubmited { status: String },
    ManifestValidated { status: String },
    DeploymentProcessed { status: String },
    Deployed { status: String },
    DeploymentCanceled { status: String },
}

impl DomainEvent for DeploymentEvent {
    fn event_type(&self) -> String {
        let event_type: &str = match self {
            DeploymentEvent::DeploymentSubmited { .. } => "DeploymentSubmited",
            DeploymentEvent::ManifestValidated { .. } => "ManifestValidated",
            DeploymentEvent::DeploymentProcessed { .. } => "DeploymentProcessed",
            DeploymentEvent::Deployed { .. } => "Deployed",
            DeploymentEvent::DeploymentCanceled { .. } => "DeploymentCanceled",
        };
        event_type.to_string()
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct DeploymentError(String);

impl Display for DeploymentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DeploymentError {}

impl From<&str> for DeploymentError {
    fn from(message: &str) -> Self {
        DeploymentError(message.to_string())
    }
}

pub struct DeploymentServices;

impl DeploymentServices {
    pub async fn validate_manifest(&self, _manifest: String) -> Result<String, DeploymentError> {
        Ok("validated".to_string())
    }
}

#[derive(Debug,Serialize, Default, Deserialize, Clone)]
pub struct Deployment {
   pub deployment_id: String,
   pub status: String,
}

#[async_trait]
impl Aggregate for Deployment {
    type Command = DeploymentCommand;
    type Event = DeploymentEvent;
    type Error = DeploymentError;
    type Services = DeploymentServices;

    // This identifier should be unique to the system.
    fn aggregate_type() -> String {
        "Account".to_string()
    }

    // The aggregate logic goes here. Note that this will be the _bulk_ of a CQRS system
    // so expect to use helper functions elsewhere to keep the code clean.
    async fn handle(
        &self,
        command: Self::Command,
        services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            DeploymentCommand::SubmitDeployment => Ok(vec![DeploymentEvent::DeploymentSubmited {
                status: "submitted".to_string(),
            }]),
            DeploymentCommand::ValidateManifest { .. } => {
                Ok(vec![DeploymentEvent::ManifestValidated {
                    status: services.validate_manifest(String::new()).await.unwrap(),
                }])
            }
            DeploymentCommand::ProcessDeployment => {
                let pending_result: Result<(), DeploymentError> = execute_task(|| Ok(())) // Simule un succès ou échec
                    .await;

                if pending_result.is_ok() {
                    Ok(vec![DeploymentEvent::Deployed {
                        status: "deployed".to_string(),
                    }])
                } else {
                    Ok(vec![DeploymentEvent::DeploymentCanceled {
                        status: "cancelled".to_string(),
                    }])
                }
            }
            
            DeploymentCommand::Deploy => Ok(vec![DeploymentEvent::Deployed {
                status: "deployed".to_string(),
            }]),
            DeploymentCommand::CancelDeployment => Ok(vec![DeploymentEvent::DeploymentCanceled {
                status: "cancelled".to_string(),
            }]),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            DeploymentEvent::DeploymentSubmited { .. } => println!("Submitting Deployment ..."),

            DeploymentEvent::ManifestValidated { .. } => {
                println!("Validating manifest ...");
            }

            DeploymentEvent::DeploymentProcessed { status, .. } => {
                self.status = status;
                println!("Processing Deployment ...");
            }

            DeploymentEvent::Deployed { status, .. } => {
                self.status = status;
                println!("Deploying ...");
            }

            DeploymentEvent::DeploymentCanceled { status, .. } => {
                self.status = status;
                println!("Cancelling ...");
            }
        }
    }
}

// struct GetDeployStateQuery {}

// #[async_trait]
// impl Query<Deployment> for GetDeployStateQuery {
//     async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<Deployment>]) {
//         for event in events {
//             println!("{}-{}\n{:#?}", aggregate_id, event.sequence, &event.payload);
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct DeploymentStatusQuery {}

#[async_trait]
impl Query<Deployment> for DeploymentStatusQuery {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<Deployment>]) {
        let mut status = "unknown".to_string();

        for event in events {
            match &event.payload {
                DeploymentEvent::DeploymentSubmited { status: s }
                | DeploymentEvent::ManifestValidated { status: s }
                | DeploymentEvent::DeploymentProcessed { status: s }
                | DeploymentEvent::Deployed { status: s }
                | DeploymentEvent::DeploymentCanceled { status: s } => {
                    status = s.clone();
                }
            }
        }

        println!(
            "Statut actuel du déploiement pour {} : {}",
            aggregate_id, status
        );
    }
}
