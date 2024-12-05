use std::sync::Arc;

use cqrs_es::Query;
use sqlite_es::{SqliteCqrs, SqliteViewRepository};
use sqlx::{Pool, Sqlite};

use crate::{
    deployment::{Deployment, DeploymentServices},
    queries::{DeploymentQuery, DeploymentView, SimpleLoggingQuery},
};

pub fn cqrs_framework(
    pool: Pool<Sqlite>,
) -> (
    Arc<SqliteCqrs<Deployment>>,
    Arc<SqliteViewRepository<DeploymentView, Deployment>>,
) {
    // A very simple query that writes each event to stdout.
    let simple_query = SimpleLoggingQuery {};

    // A query that stores the current state of an individual account.
    let deployment_view_repo = Arc::new(SqliteViewRepository::new("deployment_query", pool.clone()));
    let mut deployment_query = DeploymentQuery::new(deployment_view_repo.clone());

    // Without a query error handler there will be no indication if an
    // error occurs (e.g., database connection failure, missing columns or table).
    // Consider logging an error or panicking in your own application.
    deployment_query.use_error_handler(Box::new(|e| println!("{e}")));

    // Create and return an event-sourced `CqrsFramework`.
    let queries: Vec<Box<dyn Query<Deployment>>> =
        vec![Box::new(simple_query), Box::new(deployment_query)];
    let services = DeploymentServices;
    (
        Arc::new(sqlite_es::sqlite_cqrs(pool, queries, services)),
        deployment_view_repo,
    )
}
