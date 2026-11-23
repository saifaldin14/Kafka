#!/bin/sh
# Records the packet captures in this directory: a Kafka 4.4.0 broker, and Kafka's own
# console producer (a Java client) creating a topic and producing one record to it,
# captured by tcpdump in the broker's network namespace. Needs Docker and the network
# (it pulls the images and installs tcpdump). Run it from this directory.
#
# - producer-eth0.pcap: classic pcap, Ethernet frames, as `tcpdump -i eth0 -w` writes;
# - producer-any.pcapng: pcapng, Linux cooked frames (SLL2), as `tcpdump -i any`
#   captures them, converted by editcap.
set -eu
net=alluvium-capture
docker network create "$net" >/dev/null
trap 'docker rm -f capture-broker capture-tcpdump >/dev/null 2>&1; docker network rm "$net" >/dev/null' EXIT
docker run -d --name capture-broker --network "$net" --hostname broker \
  -e KAFKA_NODE_ID=1 -e KAFKA_PROCESS_ROLES=broker,controller \
  -e KAFKA_LISTENERS=PLAINTEXT://:9092,CONTROLLER://:9093 \
  -e KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://broker:9092 \
  -e KAFKA_CONTROLLER_LISTENER_NAMES=CONTROLLER \
  -e KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT \
  -e KAFKA_CONTROLLER_QUORUM_VOTERS=1@broker:9093 \
  -e KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1 \
  apache/kafka:4.4.0 >/dev/null
until docker exec capture-broker /opt/kafka/bin/kafka-topics.sh --bootstrap-server broker:9092 --list >/dev/null 2>&1; do sleep 1; done
docker run -d --name capture-tcpdump --network container:capture-broker -v "$PWD":/out alpine:latest \
  sh -c 'apk add -q tcpdump tshark >/dev/null &&
    { tcpdump -p -U -i eth0 -w /out/producer-eth0.pcap tcp port 9092 2>/tmp/eth0.log &
      tcpdump -p -U -i any -w /tmp/any.pcap tcp port 9092 2>/tmp/any.log &
      sleep 999; }' >/dev/null
sleep 8
docker run --rm --network "$net" apache/kafka:4.4.0 sh -c '
  /opt/kafka/bin/kafka-topics.sh --bootstrap-server broker:9092 --create --topic orders --partitions 1 >/dev/null &&
  echo "hello alluvium" | /opt/kafka/bin/kafka-console-producer.sh --bootstrap-server broker:9092 --topic orders'
sleep 2
docker exec capture-tcpdump sh -c 'cat /tmp/*.log; pkill tcpdump; sleep 1; editcap -F pcapng /tmp/any.pcap /out/producer-any.pcapng; chown '"$(id -u):$(id -g)"' /out/producer-*'
ls -l producer-eth0.pcap producer-any.pcapng
