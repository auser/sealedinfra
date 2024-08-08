terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    minikube = {
      source  = "scott-the-programmer/minikube"
      version = "~> 2.0"
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

provider "minikube" {
  kubernetes_version = var.kubernetes_version
  driver             = var.driver
}

resource "null_resource" "kubectl_config" {
  depends_on = [minikube_cluster.local]

  provisioner "local-exec" {
    command = "minikube update-context --profile=${var.cluster_name}"
  }
}
