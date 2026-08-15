
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
elseif(LN_ESP_MCU STREQUAL "ESP32C3") # ESP32C3
  set(SOC esp32c3)
elseif(LN_ESP_MCU STREQUAL "ESP32S3") # ESP32S3
  set(SOC esp32s3)
  # Xtensa: without -mlongcalls the final link fails with
  # "dangerous relocation: call8: call target out of range: esp_log"
  target_compile_options(esp32_ws2812 PRIVATE -mlongcalls)
else()
  message(FATAL_ERROR "Unsupported SOC")
endif()
foreach(comp esp_driver_rmt esp_common log esp_rom hal soc/${SOC} esp_hw_support)
  target_include_directories( esp32_ws2812 PRIVATE
    ${IDF_PATH}/components/${comp}/include/)
endforeach()
# common soc include (soc_caps_eval.h, defines SOC_HAS)
target_include_directories( esp32_ws2812 PRIVATE ${IDF_PATH}/components/soc/include )
# IDF 6.0 split the hal headers into per-peripheral esp_hal_* components
file(GLOB ESP_HAL_COMP_DIRS "${IDF_PATH}/components/esp_hal_*")
foreach(comp ${ESP_HAL_COMP_DIRS})
  if(EXISTS "${comp}/include")
    target_include_directories( esp32_ws2812 PRIVATE ${comp}/include )
  endif()
endforeach()
# IDF 6.0 moved soc/gpio_num.h into the chip-specific esp_hal_gpio include
if(EXISTS "${IDF_PATH}/components/esp_hal_gpio/${SOC}/include")
  target_include_directories( esp32_ws2812 PRIVATE ${IDF_PATH}/components/esp_hal_gpio/${SOC}/include )
endif()
# IDF 6.0 moved esp_etm.h under esp_hw_support/etm
if(EXISTS "${IDF_PATH}/components/esp_hw_support/etm/include")
  target_include_directories( esp32_ws2812 PRIVATE ${IDF_PATH}/components/esp_hw_support/etm/include )
endif()
target_include_directories( esp32_ws2812 PRIVATE ${IDF_PATH}/components/esp_driver_gpio/include)

install(TARGETS  esp32_ws2812
        DESTINATION lib )
