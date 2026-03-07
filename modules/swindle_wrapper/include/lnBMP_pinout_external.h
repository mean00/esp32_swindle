
// mapping of BMP gpio to the GPIO we use
const lnPin _mapping[10] = {
    (lnPin)0, // 0 TMS_PIN
    (lnPin)0, // 1 TDI_PIN
    (lnPin)0, // 2 TDO_PIN
    (lnPin)0, // 3 TCK_PIN
    (lnPin)0, // 4 TRACESWO_PIN

    (lnPin)GPIO18, // 5 SWDIO_PIN
    (lnPin)GPIO17, // 6 SWCLK_PIN

    (lnPin)GPIO1,  // 7 RST
    (lnPin)GPIO2,  // 8 direction
    (lnPin)GPIO16, // 9 SWDIO2
                   //
};

#define PIN_ADC_NRESET_DIV_BY_TWO GPIO3 // this pins is connected to NRST/2
#define PIN_ADC_NRESET_MULTIPLIER 1.    // 2.0 if divided by 2 , 1.0 if not divided

#define LN_USB_INSTANCE 1
#define LN_SERIAL_INSTANCE 2
#define LN_LOGGER_INSTANCE 2

#define EXTRA_SETUP()                                                                                                  \
    {                                                                                                                  \
    }
