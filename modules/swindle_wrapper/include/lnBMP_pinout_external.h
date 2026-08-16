#include "sdkconfig.h"

#if defined(LN_BOARD_SIZE_FULL)
#include "lnBMP_pinout_external_full.h"
#elif defined(LN_BOARD_SIZE_MINI)
#include "lnBMP_pinout_external_zero.h"
#else
#error "Unsupported target or board size combination! Please define the appropriate pinout file."
#endif
