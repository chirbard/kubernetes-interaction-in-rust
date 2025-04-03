use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Pod, Service};
use k8s_openapi::serde_json;
use k8s_openapi::serde_json::json;
use kube::{api::PostParams, Api, Client};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Kubernetes client
    let client = Client::try_default().await?;

    // Define the namespace
    let namespace = "default";

    // Create the Deployment
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let deployment = serde_json::from_value(json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": "simple-deployment",
            "labels": {
                "app": "simple-app"
            }
        },
        "spec": {
            "replicas": 2,
            "selector": {
                "matchLabels": {
                    "app": "simple-app"
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": "simple-app"
                    }
                },
                "spec": {
                    "containers": [{
                        "name": "simple-container",
                        "image": "nginx:latest",
                        "ports": [{
                            "containerPort": 80
                        }]
                    }]
                }
            }
        }
    }))?;
    match deployments
        .create(&PostParams::default(), &deployment)
        .await
    {
        Ok(o) => println!("Created Deployment: {:?}", o.metadata.name),
        Err(e) => eprintln!("Failed to create Deployment: {}", e),
    }

    // Create the LoadBalancer Service
    let services: Api<Service> = Api::namespaced(client, namespace);
    let service = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": "simple-service"
        },
        "spec": {
            "selector": {
                "app": "simple-app"
            },
            "ports": [{
                "protocol": "TCP",
                "port": 80,
                "targetPort": 80
            }],
            "type": "LoadBalancer"
        }
    }))?;
    match services.create(&PostParams::default(), &service).await {
        Ok(o) => println!("Created Service: {:?}", o.metadata.name),
        Err(e) => eprintln!("Failed to create Service: {}", e),
    }

    Ok(())
}
