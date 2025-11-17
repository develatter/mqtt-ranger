#!/usr/bin/env bash
# Script de prueba para simular actividad de mosquitto en localhost:1883
# Envía mensajes a distintos topics con sleeps intermedios.

MOSQUITTO_HOST="localhost"
MOSQUITTO_PORT=1883

set -e

echo "Iniciando script de prueba de mosquitto en ${MOSQUITTO_HOST}:${MOSQUITTO_PORT}"

# Mensajes iniciales (arranque de dispositivos)
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device1/status" -m "online" &
sleep 0.5
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device2/status" -m "offline" &
sleep 1
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device1/status" -m "offline" &
sleep 0.5
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device2/status" -m "online" &
sleep 1
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device1/status" -m "online" &
sleep 0.5
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device2/status" -m "offline" &
sleep 1

# Actividad en topics de actividad de usuario
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "users/alice/action" -m "login" &
sleep 0.3
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "users/alice/action" -m "open_dashboard" &
sleep 0.6
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "users/alice/action" -m "logout" &
sleep 0.3
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "users/bob/action" -m "login" &
sleep 0.3
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "users/bob/action" -m "open_dashboard" &
sleep 0.7
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "users/bob/action" -m "logout" &
sleep 0.6

# Mensajes de estado y alarmas
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device1/battery" -m "87%" &
sleep 0.4
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "alerts/temperature" -m "device1:high:75.2C" &
sleep 1.5
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device2/battery" -m "45%" &
sleep 0.4
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "alerts/temperature" -m "device2:normal:55.0C" &
sleep 1 
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device1/battery" -m "15%" &
sleep 0.4
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "alerts/battery" -m "device1:low:15%" &
sleep 1

# Burst de mensajes para simular tráfico
for i in {1..5}; do
  mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "telemetry/batch" -m "msg-$i" &
  sleep 2
done

sleep 1
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device2/status" -m "offline" &
sleep 0.5
mosquitto_pub -h "$MOSQUITTO_HOST" -p $MOSQUITTO_PORT -t "devices/device1/status" -m "offline" &

echo "Script completado."
