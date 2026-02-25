#pragma once
#include"core.h"
#include<cstdint>
#include <cstddef>

#ifdef __cplusplus
extern "C"{
#endif


typedef struct {
    FfiStream one;
    FfiStream two;
} DuoStream;

DuoStream create_duplex(size_t bufsize);
FfiStream tcp_from_fd(int fd);

FfiSlice tls_get_alpn(FfiStream stream);
void tcp_seek(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_read(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_write(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_write_all(FfiFuture fut, FfiStream stream, FfiSlice buf);
void stream_free(FfiStream stream);


#ifdef __cplusplus
}
#endif