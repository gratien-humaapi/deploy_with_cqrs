use cqrs_es::persist::ViewRepository;

use actix_web::{
    get, post,
    web::{self, Json},
    App, HttpResponse, HttpServer, Responder,
};
use deploy_with_cqrs::{
    deployment::DeploymentCommand,
    state::{new_app_state, ApplicationState},
};

#[post("/deploy")]
async fn deploy(
    app_state: web::Data<ApplicationState>,
    body: Json<DeploymentCommand>,
) -> impl Responder {
    // let aggregate_id = Uuid::new_v4().to_string();
    // Extraire le deployment_id en fonction de la variante
    let deployment_id = match body.clone() {
        DeploymentCommand::SubmitDeployment { deployment_id }
        | DeploymentCommand::ValidateManifest { deployment_id }
        | DeploymentCommand::ProcessDeployment { deployment_id }
        | DeploymentCommand::Deploy { deployment_id }
        | DeploymentCommand::CancelDeployment { deployment_id } => deployment_id,
    };

    // Vous pouvez maintenant utiliser `deployment_id`
    println!("Deployment ID: {}", deployment_id);

    match app_state.cqrs.execute(&deployment_id, body.0).await {
        Ok(_) => HttpResponse::Ok().body("Commande lancée avec succès."),
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("Erreur lors du déploiement {err:?}"))
        }
    }
}

#[get("/status/{id}")]
async fn get_status(
    id: actix_web::web::Path<String>,
    app_state: web::Data<ApplicationState>,
) -> impl Responder {
    let aggregate_id = id.into_inner();

    let view = match app_state.deployment_query.load(&aggregate_id).await {
        Ok(view) => view,
        Err(err) => {
            println!("Error: {:#?}\n", err);
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    match view {
        None => HttpResponse::NotFound().finish(),
        Some(deployment_view) => HttpResponse::Ok().json(deployment_view),
    }
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(new_app_state().await);
    // let event_store = web::Data::new(MemStore::<Deployment>::default());

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(deploy)
            .service(get_status)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// ------------------------------------------------------------------------------------

// /// Tests
// mod aggregate_tests {
//     use super::*;
//     use cqrs_es::{mem_store::MemStore, test::TestFramework, CqrsFramework};
//     use deploy_with_cqrs::deployment::DeploymentEvent;

//     type DeploymentTestFramework = TestFramework<Deployment>;

//     #[test]
//     fn submit_deployment() {
//         let expected = DeploymentEvent::DeploymentSubmited {
//             status: "submitted".to_string(),
//         };
//         DeploymentTestFramework::with(DeploymentServices)
//             .given_no_previous_events()
//             .when(DeploymentCommand::SubmitDeployment)
//             .then_expect_events(vec![expected]);
//     }

//     #[tokio::test]
//     async fn test_event_store() {
//         let event_store = MemStore::<Deployment>::default();
//         let query = DeploymentStatusQuery {};
//         let cqrs = CqrsFramework::new(event_store, vec![Box::new(query)], DeploymentServices);

//         let aggregate_id = "aggregate-instance-A";

//         // deposit $1000
//         cqrs.execute(aggregate_id, DeploymentCommand::SubmitDeployment)
//             .await
//             .unwrap();

//         // write a check for $236.15
//         cqrs.execute(aggregate_id, DeploymentCommand::ValidateManifest)
//             .await
//             .unwrap();
//     }
// }
