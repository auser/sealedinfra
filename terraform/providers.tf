terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.12.1"
    }
  }
}

provider "aws" {
  region = var.region
}

provider "kubernetes" {
  host = var.k8s_host
}

resource "kubernetes_cluster" "local" {
  driver       = var.driver
  cluster_name = var.cluster_name
  nodes        = var.node_count
  cpus         = var.cpus
  memory       = var.memory
  addons = [
    "dashboard",
    "metrics-server",
    "ingress",
  ]
}

resource "null_resource" "kubectl_config" {
  depends_on = [minikube_cluster.local]

  provisioner "local-exec" {
    command = "minikube update-context --profile=${var.k8s_cluster_name}"
  }
}
