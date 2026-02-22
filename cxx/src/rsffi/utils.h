#pragma once
#include<cstdint>
#include"core.h"

#ifdef __cplusplus
extern "C"{
#endif


typedef struct {
    FfiStream one;
    FfiStream two;
} DuoStream;

DuoStream create_duplex(size_t bufsize);

void tcp_seek(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_read(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_write(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_write_all(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_free(FfiStream stream);


#ifdef __cplusplus
}
#endif