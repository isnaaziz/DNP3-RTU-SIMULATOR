#!/bin/bash
set -euo pipefail

APP_NAME="simulator-rtu-dnp3"
REGISTRY_PATH="${REGISTRY_PATH:-10.14.152.233:8080/scada}"
NAMESPACE="${NAMESPACE:-simulator-dnp3}"
PLATFORM="${PLATFORM:-linux/amd64}"
TAG="${TAG:-latest}"
MANIFEST="k8s/simulator-rtu-dnp3.yaml"
IMAGE="$REGISTRY_PATH/$APP_NAME:$TAG"

cd "$(dirname "$0")"

echo "Build image $IMAGE ($PLATFORM)..."
docker buildx build --platform "$PLATFORM" -t "$IMAGE" --push .

echo "Pastikan namespace $NAMESPACE ada..."
kubectl get namespace "$NAMESPACE" >/dev/null 2>&1 || kubectl create namespace "$NAMESPACE"

echo "Apply $MANIFEST..."
sed "s|image: .*/$APP_NAME:.*|image: $IMAGE|" "$MANIFEST" | kubectl apply -f -

echo "Restart pod agar image terbaru dipakai..."
kubectl -n "$NAMESPACE" rollout restart deployment/"$APP_NAME"
kubectl -n "$NAMESPACE" rollout status deployment/"$APP_NAME" --timeout=180s

NODE_IP="$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="InternalIP")].address}')"
echo
echo "Selesai."
echo "  UI   : http://$NODE_IP:30791"
echo "  DNP3 : $NODE_IP:30790 (outstation 10, master 1 sesuai pengaturan di UI)"
