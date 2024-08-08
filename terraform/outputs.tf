output "cluster_ip" {
  value = k8s_cluster.local.ip
}

output "cluster_port" {
  value = k8s_cluster.local.port
}
