use k8s_openapi::api::core::v1::{Pod, PodSpec, Volume, VolumeMount};
use kube::{api::PostParams, Api, Client};
use std::env;
use tokio::{self, io::AsyncReadExt};
use uuid;

fn create_pod_spec() -> PodSpec {
    PodSpec {
        containers: vec![k8s_openapi::api::core::v1::Container {
            name: "terminal-container".to_string(),
            image: Some("ubuntu:latest".to_string()),
            command: Some(vec![
                "/bin/bash".to_string(),
                "-c".to_string(),
                "tail -f /dev/null".to_string(),
            ]),
            volume_mounts: Some(vec![VolumeMount {
                name: "user-data".to_string(),
                mount_path: "/home/user".to_string(),
                ..Default::default()
            }]),
            ..Default::default()
        }],
        volumes: Some(vec![Volume {
            name: "user-data".to_string(),
            empty_dir: Some(k8s_openapi::api::core::v1::EmptyDirVolumeSource::default()),
            ..Default::default()
        }]),
        ..Default::default()
    }
}

async fn create_pod(client: Client) -> Result<(), kube::Error> {
    let session_id = uuid::Uuid::new_v4().to_string();

    let pods: Api<Pod> = Api::default_namespaced(client);
    let pod = Pod {
        metadata: kube::core::ObjectMeta {
            name: Some(format!("user-terminal-{}", session_id)),
            labels: Some(std::collections::BTreeMap::from([(
                "session".to_string(),
                session_id.clone(),
            )])),
            ..Default::default()
        },
        spec: Some(create_pod_spec()),
        ..Default::default()
    };

    pods.create(&PostParams::default(), &pod).await?;
    println!("Created a new pod: user-terminal-{}", session_id);
    Ok(())
}

async fn delete_pod(client: Client, session_id: String) -> Result<(), kube::Error> {
    let pods: Api<Pod> = Api::default_namespaced(client);
    let pod_name = format!("user-terminal-{}", session_id);
    pods.delete(&pod_name, &Default::default()).await?;
    println!("Deleted pod: {}", pod_name);
    Ok(())
}

async fn list_pods(client: Client) -> Result<(), kube::Error> {
    let pods: Api<Pod> = Api::default_namespaced(client);
    let pod_list = pods.list(&Default::default()).await?;
    for pod in pod_list.items {
        println!("Pod: {}", pod.metadata.name.unwrap());
    }
    Ok(())
}

async fn execute_command(
    client: Client,
    session_id: String,
    command: String,
) -> Result<(), kube::Error> {
    let pods: Api<Pod> = Api::default_namespaced(client);
    let pod_name = format!("user-terminal-{}", session_id);

    let mut exec = pods
        .exec(
            &pod_name,
            vec!["/bin/bash", "-c", &command],
            &Default::default(),
        )
        .await?;

    let mut stdout_reader = exec.stdout().unwrap();
    let mut output = String::new();
    let mut buf = [0u8; 1024];
    loop {
        let result_length = match stdout_reader.read(&mut buf).await {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Error reading from stdout: {}", e);
                return Err(kube::Error::ReadEvents(e));
            }
        };
        println!("Read {} bytes", result_length);
        output.push_str(&String::from_utf8_lossy(&buf[..result_length]));
        if result_length < 1024 {
            break;
        }
    }

    println!("Command output: {}", output);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let client = Client::try_default().await?;

    if args.len() > 1 && args[1] == "new" {
        create_pod(client).await?;
    } else if args.len() > 1 && args[1] == "delete" {
        if args.len() < 3 {
            println!("Usage: delete <session_id>");
            return Ok(());
        }
        let session_id = args[2].clone();
        delete_pod(client, session_id).await?;
    } else if args.len() > 1 && args[1] == "list" {
        list_pods(client).await?;
    } else if args.len() > 2 {
        let session_id = args[1].clone();
        let command = args[2..].join(" ");
        execute_command(client, session_id, command).await?;
    } else {
        println!("Usage:");
        println!("  new - Create a new pod");
        println!("  delete <session_id> - Delete the pod with the given session ID");
        println!("  list - List all pods");
        println!("  <session_id> <command> - Execute a command in the pod");
    }

    Ok(())
}
