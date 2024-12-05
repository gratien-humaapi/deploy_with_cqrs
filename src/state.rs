
use serde::{Deserialize, Serialize};
use sqlite_es::{default_sqlite_pool, SqliteCqrs, SqliteViewRepository};
use std::sync::Arc;

use crate::{config::cqrs_framework, deployment::Deployment, queries::DeploymentView};


#[derive(Clone)]
pub struct ApplicationState {
    pub cqrs: Arc<SqliteCqrs<Deployment>>,
    pub deployment_query: Arc<SqliteViewRepository<DeploymentView, Deployment>>,
}

pub async fn new_app_state() -> ApplicationState {
    // Configure the CQRS framework, backed by an SQLite database, along with two queries:
    // - a simply-query prints events to stdout as they are published
    // - `deployment_query` stores the current state of the deployment in a ViewRepository that we can access
    //
    // The needed database tables are automatically configured with `docker-compose up -d`,
    // see init file at `/db/init.sql` for more.
    let pool = default_sqlite_pool("sqlite://demo.db").await;
    sqlx::migrate!().run(&pool).await.unwrap();
    let (cqrs, deployment_query) = cqrs_framework(pool);

    ApplicationState {
        cqrs,
        deployment_query,
    }
}
