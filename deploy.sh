#!/bin/bash
docker build -t beemstream/profile-service .
docker push beemstream/profile-service
ssh root@157.245.43.172 "docker service update --image beemstream/profile-service beemstream_profile_service"

