use cqrs_es::Aggregate;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use cqrs_es::{mem_store::MemStore, CqrsFramework, EventStore, Query};
use deploy_with_cqrs::{
    deployment::{Deployment, DeploymentCommand, DeploymentServices, DeploymentStatusQuery},
    state::{new_app_state, AppState},
};

// pub async fn handle_command(deployments: HashMap<i32, Deployment>, id: i32, command: DeploymentCommand) -> Result<(), String> {
//     if let Some(state) = deployments.get(&id) {
//         let mut new_state = state.clone();
//         let events = new_state.ha(command).await;
//         for event in events {
//             new_state.apply(event);
//         }
//         manager.update_deployment(&id, new_state);
//     }
//     Ok(())
// }

#[post("/deploy")]
async fn deploy(data: web::Data<AppState>) -> impl Responder {
    let mut manager = data.manager.lock().unwrap();
    let cqrs = &data.cqrs;

    let id = manager.create_deployment();

    let aggregate_id = id.to_string();

    // Étape 1 : Soumission du déploiement
    if let Err(e) = cqrs
        .execute(&aggregate_id, DeploymentCommand::SubmitDeployment)
        .await
    {
        return HttpResponse::InternalServerError().body(format!(
            "Erreur lors de la soumission du déploiement : {:?}",
            e
        ));
    }

    // Étape 2 : Validation du manifeste
    if let Err(e) = cqrs
        .execute(&aggregate_id, DeploymentCommand::ValidateManifest)
        .await
    {
        return HttpResponse::InternalServerError().body(format!(
            "Erreur lors de la validation du manifeste : {:?}",
            e
        ));
    }

    // Étape 3 : Traitement du déploiement (état Pending)
    if let Err(e) = cqrs
        .execute(&aggregate_id, DeploymentCommand::ProcessDeployment)
        .await
    {
        return HttpResponse::InternalServerError().body(format!(
            "Erreur lors du traitement du déploiement : {:?}",
            e
        ));
    }

    // Étape 4 : Gestion du succès ou de l’échec de l'état Pending
    let deployment_success = true; // Simulez la réussite ou l'échec ici.
    if deployment_success {
        if let Err(e) = cqrs.execute(&aggregate_id, DeploymentCommand::Deploy).await {
            return HttpResponse::InternalServerError()
                .body(format!("Erreur lors du déploiement : {:?}", e));
        }
    } else {
        if let Err(e) = cqrs
            .execute(&aggregate_id, DeploymentCommand::CancelDeployment)
            .await
        {
            return HttpResponse::InternalServerError().body(format!(
                "Erreur lors de l'annulation du déploiement : {:?}",
                e
            ));
        }
    }

    HttpResponse::Ok().body("Processus de déploiement terminé.")
}

#[get("/status/{id}")]
async fn get_deploy_status(data: web::Data<AppState>, path: web::Path<i32>) -> impl Responder {
    let manager = data.manager.lock().unwrap();
    let query = data.query.as_ref();
    
    let id = path.into_inner();
    
    query.dispatch(&id.to_string(), events);

    HttpResponse::Ok().json(format!("status : {:?}", status.unwrap()))
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(new_app_state().await);

    HttpServer::new(move || {
        App::new().app_data(state.clone()).service(deploy)
        .service(get_deploy_status)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

/// Tests
mod aggregate_tests {
    use super::*;
    use cqrs_es::{mem_store::MemStore, test::TestFramework, CqrsFramework};
    use deploy_with_cqrs::deployment::DeploymentEvent;

    type DeploymentTestFramework = TestFramework<Deployment>;

    #[test]
    fn submit_deployment() {
        let expected = DeploymentEvent::DeploymentSubmited {
            status: "submitted".to_string(),
        };
        DeploymentTestFramework::with(DeploymentServices)
            .given_no_previous_events()
            .when(DeploymentCommand::SubmitDeployment)
            .then_expect_events(vec![expected]);
    }

    #[tokio::test]
    async fn test_event_store() {
        let event_store = MemStore::<Deployment>::default();
        let query = DeploymentStatusQuery {};
        let cqrs = CqrsFramework::new(event_store, vec![Box::new(query)], DeploymentServices);

        let aggregate_id = "aggregate-instance-A";

        // deposit $1000
        cqrs.execute(aggregate_id, DeploymentCommand::SubmitDeployment)
            .await
            .unwrap();

        // write a check for $236.15
        cqrs.execute(aggregate_id, DeploymentCommand::ValidateManifest)
            .await
            .unwrap();
    }
}
