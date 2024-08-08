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

terraform {
  backend "s3" {
  }
}

provider "kubernetes" {
  kubernetes_version = var.kubernetes_version
}

resource "k8s_cluster" "local" {
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
    command = "minikube update-context --profile=${var.cluster_name}"
  }
}
