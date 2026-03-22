# SWINDLE ESP32S3

This is a quick port of the swindle SWD debugger on the ESP32S3/Wifi.

## Pinout

### S3 Mini

![screenshot](assets/web/s3mini.png?raw=true "front")

- GPIO21: Pin to WS2812
- GPIO1 : Ground to reset provisioning
- GPIO7 : SWDIO
- GPIO8 : SWDCLK
- GPIO9 : RESET
- GPIO10: ADC (read target voltage, capped at 3.1v)

### S3 Dev Board

- GPIO48: Pin to WS2812
- GPIO1 : Ground to reset provisioning
- GPIO18: SWDIO
- GPIO17: SWDCLK
- GPIO2 : RESET
- GPIO4 : ADC (read target voltage, capped at 3.1v)

## PROVISIONING

Use the Espressif BLE provisioning app to give wifi credentials. The device name is SWINDLE_ESP32.

The pop is the default (abcd1234)

## LED COLOR

- Red : Waiting for provisioning
- Pink White : Successfully reset provisionning
- Yellow : Trying to connect to network
- Green : Connected to network
- Blue : Attached to a debugger

## Building

There is a helper script to build all the components in the right order : build_all.sh

The build targets idf 5.4

## Flashing

- Download the .gz binary
- Gunzip it if not done automatically
- espflash flash -M swindle_s3

## Using

target extended-remote 192.168.0.149:8080

mon swdp_scan

mon attach 1
