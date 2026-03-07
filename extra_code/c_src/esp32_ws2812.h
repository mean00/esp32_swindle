#pragma once
#include <cstdint>

// Initialize the driver on the given GPIO (call once)
void ws2812_init(uint8_t pinno);

// Set color (0xRRGGBB format). Blocking until sent.
void ws2812_set_color(uint32_t color);
