use cqrs_es::Aggregate;
use uuid::Uuid;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use cqrs_es::{mem_store::MemStore, CqrsFramework, EventStore, Query};
use deploy_with_cqrs::{
    deployment::{Deployment, DeploymentCommand, DeploymentEvent, DeploymentServices, DeploymentStatusQuery},
    state::{new_app_state, AppState},
};


#[post("/deploy")]
async fn deploy(event_store: web::Data<MemStore<Deployment>>) -> impl Responder {
    let query = DeploymentStatusQuery {};
    let cqrs = CqrsFramework::new(
        event_store.as_ref().clone(),
        vec![Box::new(query)],
        DeploymentServices,
    );

    let aggregate_id = Uuid::new_v4().to_string();

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
    // let deployment_success = true; // Simulez la réussite ou l'échec ici.
    // if deployment_success {
    //     if let Err(e) = cqrs.execute(&aggregate_id, DeploymentCommand::Deploy).await {
    //         return HttpResponse::InternalServerError()
    //             .body(format!("Erreur lors du déploiement : {:?}", e));
    //     }
    // } else {
    //     if let Err(e) = cqrs
    //         .execute(&aggregate_id, DeploymentCommand::CancelDeployment)
    //         .await
    //     {
    //         return HttpResponse::InternalServerError().body(format!(
    //             "Erreur lors de l'annulation du déploiement : {:?}",
    //             e
    //         ));
    //     }
    // }

    HttpResponse::Ok().body("Processus de déploiement terminé.")
}

#[get("/status/{id}")]
async fn get_status(
    id: actix_web::web::Path<String>,
    event_store: web::Data<MemStore<Deployment>>,
) -> impl Responder {
    let aggregate_id = id.into_inner();

    // Charge les événements directement à partir du magasin d'événements
    let events = event_store
        .as_ref()
        .load_events(&aggregate_id)
        .await
        .unwrap_or_else(|_| vec![]);

    if events.is_empty() {
        return HttpResponse::NotFound().body("Déploiement introuvable.");
    }

    // Parcourt les événements pour trouver le dernier statut
    let mut status = "unknown".to_string();
    for event in &events {
        match &event.payload {
            DeploymentEvent::DeploymentSubmited { .. } => status = "submitted".to_string(),
            DeploymentEvent::ManifestValidated { .. } => status = "validated".to_string(),
            DeploymentEvent::DeploymentProcessed { .. } => status = "processing".to_string(),
            DeploymentEvent::Deployed { .. } => status = "deployed".to_string(),
            DeploymentEvent::DeploymentCanceled { .. } => status = "cancelled".to_string(),
        }
    }

    HttpResponse::Ok().body(format!("Statut actuel : {}", status))
}


#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    // let state = web::Data::new(new_app_state().await);
    let event_store = web::Data::new(MemStore::<Deployment>::default());

    HttpServer::new(move || {
        App::new().app_data(event_store.clone()).service(deploy)
        .service(get_status)
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
