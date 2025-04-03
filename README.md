# kubernetes-interaction-in-rust

This creates a simple kubernetes deployment and service in rust.

This program needs to be run in a system where kubernetes is running.

Works when minikube is set up in windows.

Setting up minikube: https://minikube.sigs.k8s.io/docs/start/?arch=%2Fwindows%2Fx86-64%2Fstable%2Fwindows+package+manager

Cloud services such as Azure can use a Load Balancer by default and give a external ip to the service. In minikube you have to run the following to get the external ip.

```sh
minikube tunnel
```

## Useful commands

```sh
kubectl get pods
kubectl get deployments
kubectl get services
kubectl delete deployment <name>

cargo fetch
cargo build
cargo run
cargo update
```

## Usage

```
Usage:
  new - Create a new pod
  delete <session_id> - Delete the pod with the given session ID
  list - List all pods
  <session_id> <command> - Execute a command in the pod
```
