use std::time::Duration;

use tokio::time::sleep;



pub async fn execute_task<F, T>(task: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        // Simule un délai pour une tâche
        sleep(Duration::from_secs(2)).await;
        task()
    })
    .await
    .expect("Erreur lors de l'exécution de la tâche")
}