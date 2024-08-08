variable "region" {
  description = "The AWS region to deploy in"
  type        = string
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
}

variable "cidr_blocks" {
  description = "List of CIDR blocks for security group"
  type        = list(string)
}

variable "vpc_id" {
  description = "The VPC ID where the instance will be deployed"
  type        = string
}

variable "subnet_id" {
  description = "The Subnet ID where the instance will be deployed"
  type        = string
}

# variable "ami_id" {
#   description = "AMI ID to use for the EC2 instance"
#   type        = string
# }

variable "security_group_name" {
  description = "Name of the security group"
  type        = string
}

variable "instance_name" {
  description = "The name tag for the EC2 instance"
  type        = string
}

variable "key_name" {
  description = "The name of the PEM key to use for the instance"
  type        = string
}

variable "ingress_rules" {
  description = "List of ingress rules"
  type = list(object({
    from_port   = number
    to_port     = number
    protocol    = string
    cidr_blocks = list(string)
  }))
}

variable "egress_rules" {
  description = "List of egress rules"
  type = list(object({
    from_port   = number
    to_port     = number
    protocol    = string
    cidr_blocks = list(string)
  }))
}

variable "role_name" {
  description = "instance role for ssm login"
  type        = string
}
