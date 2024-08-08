variable "region" {
  type    = string
  default = "us-east-1"
}

variable "profile" {
  type    = string
  default = "default"
}

variable "kubernetes_version" {
  type    = string
  default = "1.29.15"
}

variable "driver" {
  type    = string
  default = "docker"
}

variable "cluster_name" {
  type    = string
  default = "local-k8s"
}

variable "cpus" {
  type    = number
  default = 2
}

variable "memory" {
  type    = number
  default = 2048
}
