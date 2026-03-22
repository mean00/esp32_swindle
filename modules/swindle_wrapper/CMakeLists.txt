cmake_minimum_required(VERSION 3.13)
project(swindle_wrapper)

include($ENV{IDF_PATH}/tools/cmake/idf.cmake)

set(LN_ENABLED_BOARDS   CH32F1 RP2040 RP2350 STM32F1  NRF51 NRF91 LPC11XX LPC15XX LPC17XX STM32F4 STM32G0 STM32H7 STM32H5 STM32L4 CACHE INTERNAL "")
set(LN_MCU              "ESP32")
set(LN_ARCH             "RISCV")
set(LN_MCU_RAM_SIZE     256)
set(LN_MCU_FLASH_SIZE   2048)
set(LN_EXTERNAL_RUST    True)
set(LN_EXTERNAL_FREERTOS True)
set(LN_EXTERNAL_TAP True)
set(LN_EXTERNAL_MALLOC  True)
set(LN_EXTERNAL_GENERATE True)
set(LN_SWINDLE_AS_EXTERNAL True)
set(LN_EXT              "riscv_esp32")
SET(USE_64PIN_PACKAGE    True )
set(USE_NO_DEFAULT       True)
set(SWINDLE_USE_USB      False)
set(SWINDLE_USE_NETWORK  OFF)
set(LN_ENABLE_RUST      OFF)
set(LN_DISABLE_SWINDLE_LNIO      ON)
#set(LN_ESP_MCU "ESP32S3")
set(swindle_root ${CMAKE_CURRENT_LIST_DIR}/../swindle)
set(BMP ${swindle_root}/blackmagic)
add_definitions("-DLN_ARCH=1") # riscv
add_definitions("-DportTICK_RATE_MS=portTICK_PERIOD_MS")
set(ESPRIT_WRAPPER  ${CMAKE_CURRENT_LIST_DIR}/../swindle/esprit/)
set(ESPRIT_ROOT  ${ESPRIT_WRAPPER})
message(STATUS "Esprit root is  ${ESPRIT_ROOT}")
message(STATUS "Freertos ipaths are $ENV{IDF_PATH}/components/freertos/FreeRTOS-Kernel/include:$ENV{IDF_PATH}/components/freertos/esp_additions/include/freertos")
message(STATUS "FreeRTOS config is  ${CMAKE_BINARY_DIR}/config")
include( ${ESPRIT_ROOT}/esprit.cmake)
include_directories( ${CMAKE_CURRENT_BINARY_DIR})
include_directories( ${CMAKE_CURRENT_SOURCE_DIR}/include)
include_directories( ${CMAKE_CURRENT_LIST_DIR}/)
include_directories( ${ESPRIT_WRAPPER}/include)
include_directories("$ENV{IDF_PATH}/components/freertos/FreeRTOS-Kernel/include"
"$ENV{IDF_PATH}/components/freertos/esp_additions/include/freertos"
)
#include_directories( ${ESPRIT_WRAPPER}/mcus/riscv_esp32/include)
#"${CMAKE_BINARY_DIR}/config"


add_subdirectory(../swindle/ swindle_binary)
GENERATE_GD32_FIRMWARE()

add_library(swindle_single STATIC
$<TARGET_OBJECTS:swindle_interface>
$<TARGET_OBJECTS:swindle>
$<TARGET_OBJECTS:libswindle>
$<TARGET_OBJECTS:swindleio_impl>
)

add_dependencies(swindle_single swindle_interface swindle libswindle swindleio_impl  )
install(TARGETS  swindle_single
        DESTINATION lib )
#install(TARGETS  swindle_single esprit_single_lib
#DESTINATION lib )

#${CMAKE_BINARY_DIR}/../foo)
