# Swindle ESP32

[![ESP-IDF 6.0](https://img.shields.io/badge/ESP--IDF-v6.0-red.svg)](https://docs.espressif.com/projects/esp-idf/en/stable/esp32s3/index.html)
[![Rust](https://img.shields.io/badge/Rust-1.77+-orange.svg)](https://rust-lang.org)

Swindle ESP32 is a fast, wireless SWD debugger port for ESP32 microcontrollers. It allows you to debug ARM/WCH RISC-V targets over WiFi using a tiny, low-cost ESP32 board.

> 📸 _Pictured: A tiny ESP32-S3 debugging an STM32 over WiFi. The USB connection is only used for power._
> ![Screenshot](assets/web/s3mini.png?raw=true "front")

---

## 📌 Pinout Configurations

The firmware supports dynamic pinout selection depending on your target MCU and board profile. Pinout headers are located in `modules/swindle_wrapper/include/`.

### ESP32-S3 Mini

| Function       | GPIO     | Notes                                 |
| :------------- | :------- | :------------------------------------ |
| **SWDIO**      | `GPIO7`  | Target SWD I/O                        |
| **SWDCLK**     | `GPIO8`  | Target SWD Clock                      |
| **RESET**      | `GPIO9`  | Target Reset(inverted)                |
| **ADC**        | `GPIO10` | Reads target voltage (capped at 3.1V) |
| **WS2812**     | `GPIO21` | Status LED                            |
| **PROV RESET** | `GPIO1`  | Ground to reset provisioning          |

### ESP32-S3 Dev Board

| Function       | GPIO     | Notes                                 |
| :------------- | :------- | :------------------------------------ |
| **SWDIO**      | `GPIO18` | Target SWD I/O                        |
| **SWDCLK**     | `GPIO17` | Target SWD Clock                      |
| **RESET**      | `GPIO2`  | Target Reset                          |
| **ADC**        | `GPIO4`  | Reads target voltage (capped at 3.1V) |
| **WS2812**     | `GPIO48` | Status LED                            |
| **PROV RESET** | `GPIO1`  | Ground to reset provisioning          |

---

## 🛠️ Building

A helper script is provided to automatically compile the Rust and C++ components with the correct configuration, target, and CMake macros.

```bash
# Usage: ./build_all.sh [MCU] [SIZE] [RESET]
# MCU options: esp32c3 (default), esp32s3, esp32c6
# Size options: full (default), mini, zero
# Reset options: straight (default), inverted

# Example: Build for ESP32-S3 Mini
./build_all.sh esp32s3 mini

# Example: Build for ESP32-C3 Full
./build_all.sh esp32c3 full

# Example: Build for ESP32-S3 zero board with inverted NRST (via a MOSFET)
./build_all.sh esp32s3 zero inverted
```

The compiled binary will be placed in the `target/` directory as `swindle_MCU_SIZE`
(or `swindle_MCU_SIZE_inverted` when the `inverted` reset mode is selected).

## ⚡ Flashing

Flash the generated binary to your ESP32 using `espflash`. The exact flash command is printed at the end of a successful build:

```bash
espflash flash -M target/swindle_esp32s3_mini
```

---

## 📶 Provisioning & Status

Swindle uses BLE for initial WiFi credential provisioning.

1. Download the **Espressif BLE Provisioning** app on your phone.
2. Connect to the device named **`SWINDLE_ESP32`**.
3. Use the default POP (Proof of Possession): **`abcd1234`**.

### LED Status Indicators

| Color             | Status                           |
| :---------------- | :------------------------------- |
| 🔴 **Red**        | Waiting for provisioning         |
| 🌸 **Pink/White** | Successfully reset provisioning  |
| 🟡 **Yellow**     | Trying to connect to the network |
| 🟢 **Green**      | Connected to WiFi                |
| 🔵 **Blue**       | Attached to a debugger / Active  |

---

## 🐛 Using Swindle (GDB)

Once connected to WiFi, Swindle hosts a GDB server on port `8080`. Connect to it from your GDB client:

```gdb
# Connect to Swindle (replace IP with your ESP32's IP)
target extended-remote 192.168.0.149:8080

# Scan for targets
mon swdp_scan

# Attach to the first target
mon attach 1
```
