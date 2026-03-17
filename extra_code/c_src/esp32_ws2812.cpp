extern "C"
{
#include "driver/gpio.h"
#include "driver/rmt_tx.h"
#include "esp_err.h"
#include "esp_log.h"
//
#include "esp32_ws2812.h"
}
//
static const char *TAG = "WS2812";

static rmt_channel_handle_t s_chan = nullptr;
static rmt_encoder_handle_t s_encoder = nullptr;
extern uint8_t ln_get_ws2812_pin();
/*
 *
 */
extern "C" void ws2812_init()
{
    uint8_t pinno = ln_get_ws2812_pin();

    if (s_chan != nullptr)
    {
        ESP_LOGW(TAG, "Already initialized");
        return;
    }

    // ── RMT TX channel (10 MHz resolution = 0.1 µs/tick) ─────────────────────
    rmt_tx_channel_config_t tx_config = {
        .gpio_num = (gpio_num_t)pinno,
        .clk_src = RMT_CLK_SRC_DEFAULT,
        .resolution_hz = 10 * 1000 * 1000,
        .mem_block_symbols = 48, // plenty for 1 LED (24 bits)
        .trans_queue_depth = 4,
        .flags =
            {
                .invert_out = false,
                .with_dma = false,
            },
    };
    ESP_ERROR_CHECK(rmt_new_tx_channel(&tx_config, &s_chan));
    ESP_ERROR_CHECK(rmt_enable(s_chan));

    // ── Bytes encoder for WS2812 timing (GRB, MSB first) ─────────────────────
    rmt_bytes_encoder_config_t encoder_config = {.bit0 =
                                                     {
                                                         .duration0 = 4, // 0.4 µs high
                                                         .level0 = 1,
                                                         .duration1 = 9, // 0.9 µs low  → total ~1.3 µs
                                                         .level1 = 0,
                                                     },
                                                 .bit1 =
                                                     {
                                                         .duration0 = 8, // 0.8 µs high
                                                         .level0 = 1,
                                                         .duration1 = 5, // 0.5 µs low  → total ~1.3 µs
                                                         .level1 = 0,
                                                     },
                                                 .flags = {
                                                     .msb_first = true,
                                                 }};
    ESP_ERROR_CHECK(rmt_new_bytes_encoder(&encoder_config, &s_encoder));

    // Start off
    ws2812_set_color(0x000000);
    ESP_LOGI(TAG, "WS2812 ready on GPIO %d", pinno);
}
/*
 *
 *
 */
extern "C" void ws2812_set_color(uint32_t color)
{
    if (s_chan == nullptr || s_encoder == nullptr)
    {
        ESP_LOGE(TAG, "Call ws2812_init() first!");
        return;
    }

    // WS2812 expects GRB order
    uint8_t pixel[3] = {
        (uint8_t)(color >> 8),  // G
        (uint8_t)(color >> 16), // R
        (uint8_t)(color >> 0)   // B
    };

    rmt_transmit_config_t tx_config = {
        .loop_count = 0, .flags = {.eot_level = 0} // end low → reset pulse
    };

    ESP_ERROR_CHECK(rmt_transmit(s_chan, s_encoder, pixel, sizeof(pixel), &tx_config));

    // Block until the whole frame is sent (very fast for 1 LED)
#define pdMS_TO_TICKS(x) x
    ESP_ERROR_CHECK(rmt_tx_wait_all_done(s_chan, pdMS_TO_TICKS(50)));
}
