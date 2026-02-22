#include"rsffi/base.h"
#include<string.h>


static FfiSlice sliceFromCstr(const char* s) {
    FfiSlice slice;
    slice.owned = false;
    slice.len = strlen(s);
    slice.cap = slice.len;
    slice.ptr = (uint8_t*)s;
    return slice;
}
