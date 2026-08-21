#include "sdkconfig.h"

#if defined(LN_BOARD_SIZE_DEV)
#include "lnBMP_pinout_external_dev.h"
#elif defined(LN_BOARD_SIZE_ZERO) || defined(LN_BOARD_SIZE_MINI)
#include "lnBMP_pinout_external_zero.h"
#elif defined(LN_BOARD_SIZE_ALTERNATEZERO)
#include "lnBMP_pinout_external_alternatezero.h"
#else
#error "Unsupported target or board size combination! Please define the appropriate pinout file."
#endif
