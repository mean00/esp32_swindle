# SWINDLE ESP32S3

This is a quick port of the swindle SWD debugger on the ESP32S3/Wifi.

## Pinout

- GPIO48 : Pin to WS2812
- GPIO1: Ground to reset provisioning
- GPIO18: SWDIO
- GPIO17: SWDCLK
- GPIO2: RESET

(you can change the WS2812 pin by altering app/src/settings.rs, the zero is using GPIO21)

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
