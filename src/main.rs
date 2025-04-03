use k8s_openapi::api::core::v1::{Pod, PodSpec, Volume, VolumeMount};
use kube::{api::PostParams, Api, Client};
use std::env;
use tokio::{self, io::AsyncReadExt};
use uuid;

async fn create_pod(client: Client) -> Result<(), kube::Error> {
    // Generate a unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create a Pod with a terminal
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
        spec: Some(PodSpec {
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
        }),
        ..Default::default()
    };

    pods.create(&PostParams::default(), &pod).await?;
    println!("Created a new pod: user-terminal-{}", session_id);
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
    // stdout_reader.read_exact(&mut buf).await;
    loop {
        let result_length = match stdout_reader.read(&mut buf).await {
            Ok(size) => size, // Extract the usize value
            Err(e) => {
                eprintln!("Error reading from stdout: {}", e);
                return Err(kube::Error::ReadEvents(e));
            }
        };
        println!("Read {} bytes", result_length);
        output.push_str(&String::from_utf8_lossy(&buf[..result_length]));
        if result_length < 1024 {
            break; // End of stream
        }
    }

    println!("Command output: {}", output);

    // Get the stdout stream
    // if let Some(mut stdout_reader) = exec.stdout() {
    //     let mut output = String::new();
    //     let mut buf = [0u8; 1024]; // Use a larger buffer for efficiency

    //     // Read the output in chunks until the stream ends
    //     loop {
    //         let n = stdout_reader.read_exact(&mut buf).await;
    //         if n == 0 {
    //             break; // End of stream
    //         }
    //         output.push_str(&String::from_utf8_lossy(&buf[..n]));
    //     }

    //     // Print the full output
    //     println!("Command output: {}", output);
    // } else {
    //     println!("No output received from the command.");
    // }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Create a Kubernetes client
    let client = Client::try_default().await?;

    // Check if the "new" parameter is passed
    if args.len() > 1 && args[1] == "new" {
        create_pod(client).await?;
    } else if args.len() > 2 {
        // If session ID and command are provided
        let session_id = args[1].clone();
        let command = args[2..].join(" "); // Combine all remaining arguments as the command
        execute_command(client, session_id, command).await?;
    } else {
        println!("Usage:");
        println!("  new - Create a new pod");
        println!("  <session_id> <command> - Execute a command in the pod");
    }

    Ok(())
}
