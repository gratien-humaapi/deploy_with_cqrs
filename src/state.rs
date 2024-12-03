use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cqrs_es::{mem_store::MemStore, CqrsFramework};

use crate::deployment::{Deployment, DeploymentServices, DeploymentStatusQuery};

#[derive(Debug, Default, Clone)]
pub struct DeploymentManager {
    pub deployments: HashMap<i32, Deployment>,
}

impl DeploymentManager {
    pub fn new() -> Self {
        Self {
            deployments: HashMap::new(),
        }
    }

    pub fn create_deployment(&mut self) -> i32 {
        let id = self.deployments.len().try_into().unwrap();
        self.deployments.insert(id, Deployment::default());
        id
    }

    pub fn get_deployment(&self, id: &i32) -> Option<Deployment> {
        self.deployments.get(id).cloned()
    }

    pub fn update_deployment(&mut self, id: &i32, state: Deployment) {
        self.deployments.insert(*id, state);
    }
}

#[derive(Clone)]
pub struct AppState {
    pub cqrs: Arc<CqrsFramework<Deployment, MemStore<Deployment>>>,
    pub query : Arc<DeploymentStatusQuery>,
    pub manager: Arc<Mutex<DeploymentManager>>,
}

pub async fn new_app_state() -> AppState {
    let event_store = MemStore::<Deployment>::default();
    let query: DeploymentStatusQuery = DeploymentStatusQuery {};
    let cqrs: CqrsFramework<Deployment, MemStore<Deployment>> =
        CqrsFramework::new(event_store, vec![Box::new(query.clone())], DeploymentServices);
    AppState {
        cqrs: Arc::new(cqrs),
       query:  Arc::new(query),
        manager: Arc::new(Mutex::new(DeploymentManager::new())),
    }
}
