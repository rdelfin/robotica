image := "ghcr.io/rdelfin/robotica-ci"
tag := "latest"
platforms := "linux/amd64,linux/arm64"

# Build the CI image for all platforms.
docker-build:
    docker buildx build --platform {{platforms}} -f ci.Dockerfile -t {{image}}:{{tag}} --load .

# Push the built CI image to ghcr.
docker-push:
    docker push {{image}}:{{tag}}
