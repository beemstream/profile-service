#!/bin/bash

cargo bump patch

version=$(awk -F'[ ="]+' '$1 == "version" { print $2 }' Cargo.toml)

docker build -t beemstream/profile-service:$version .
docker push beemstream/profile-service:$version

# Tag image as latest
docker pull beemstream/profile-service:$version
docker tag beemstream/profile-service:$version beemstream/profile-service:latest
docker push beemstream/profile-service:latest

ssh root@157.245.43.172 "docker service update --image beemstream/profile-service beemstream_profile_service"
