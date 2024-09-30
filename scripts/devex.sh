#!/usr/bin/env bash

CONTAINER_NAME="sealedinfra_devcontainer-development"

docker_instance() {
    docker ps | grep "$CONTAINER_NAME" | awk '{print $1}'
}

exec_instance() {
    local docker_instance=$(docker_instance)
    if [[ -z "$docker_instance" ]]; then
        printf "No container found"
        exit 1
    fi
    docker exec -it ${docker_instance} /usr/bin/zsh
}

exec_instance