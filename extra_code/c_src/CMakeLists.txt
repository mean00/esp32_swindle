
cmake_minimum_required(VERSION 3.13)
project(extra_code)

include($ENV{IDF_PATH}/tools/cmake/idf.cmake)


add_library(esp32_ws2812 STATIC esp32_ws2812.cpp)
target_include_directories( esp32_ws2812 PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}
    #${IDF_PATH}/components/esp_driver_rmt/include/
    #${IDF_PATH}/components/esp_common/include/
    #${IDF_PATH}/components/log/include/
    #${IDF_PATH}/components/esp_rom/include/
    #${IDF_PATH}/components/hal/include/
    #${IDF_PATH}/components/soc/esp32s3/include/
)
if(LN_ESP_MCU STREQUAL "ESP32C6") # ESP32C6
  set(SOC esp32c6)
elseif(LN_ESP_MCU STREQUAL "ESP32S3") # ESP32S3
  set(SOC esp32s3)
else()
  message(FATAL_ERROR "Unsupported SOC")
endif()
foreach(comp esp_driver_rmt esp_common log esp_rom hal soc/esp32s3 esp_hw_support)
  target_include_directories( esp32_ws2812 PRIVATE
    ${IDF_PATH}/components/${comp}/include/)
endforeach()
target_include_directories( esp32_ws2812 PRIVATE ${IDF_PATH}/components/esp_driver_gpio/include)

install(TARGETS  esp32_ws2812
        DESTINATION lib )
