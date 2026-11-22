#!/usr/bin/env bash
# Regenerates murmur2.txt and partition_for_key.txt by running Kafka 4.4.0's own code
# (kafka_hashes.jsh) in JShell. Needs Docker and network access; the image and jars
# are pinned by digest and checksum.
set -euo pipefail
cd "$(dirname "$0")"

image="eclipse-temurin:17-jdk@sha256:7a0b2bc7e1372bb05ee361fb79e8a9e822df47343b8716253b3bbaa75060653e"
maven="https://repo1.maven.org/maven2"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

fetch() {
    local path="$1" sha256="$2" file
    file="$work/$(basename "$path")"
    curl -sfL "$maven/$path" -o "$file"
    echo "$sha256  $file" | shasum -a 256 -c - >/dev/null
}

fetch org/apache/kafka/kafka-clients/4.4.0/kafka-clients-4.4.0.jar \
    1ab07317eb6633241392449d329dbdc5d9251e6ffc4926860f42dd966e88bdf9
# Utils has a static slf4j logger, so its class needs slf4j-api to load.
fetch org/slf4j/slf4j-api/1.7.36/slf4j-api-1.7.36.jar \
    d3ef575e3e4979678dc01bf1dcce51021493b4d11fb7f1be8ad982877c16a1c0

cp kafka_hashes.jsh "$work/"
docker run --rm -v "$work:/w" "$image" jshell -q \
    --class-path /w/kafka-clients-4.4.0.jar:/w/slf4j-api-1.7.36.jar /w/kafka_hashes.jsh
cp "$work/murmur2.txt" "$work/partition_for_key.txt" .
