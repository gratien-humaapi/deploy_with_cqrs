## Installation

Clone this repository

    git clone https://github.com/gratien-humaapi/deploy_with_cqrs.git

Start the application

    cargo run

Submit a deployment

    curl -i -X POST http://localhost:8080/deploy --header 'Content-Type: application/json' --data-raw '{"SubmitDeployment": {"deployment_id": "1"}}'


Validate a manifest

    curl -i -X POST http://localhost:8080/deploy --header 'Content-Type: application/json' --data-raw '{"ValidateManifest": {"deployment_id": "1"}}'


Get deploy status

    curl -i http://127.0.0.1:8080/status/1


