#pragma once
#include<cstdint>

#ifdef __cplusplus
extern "C"{
#endif

bool init_rt();  // init rust tokio runtime
bool has_init();

typedef void* FfiFuture;
typedef void* FfiServer;

typedef void* FfiSocket;
typedef void* FfiReques;

typedef void* FfiStream;
typedef void* SockAddre;

typedef void* WebSocket;

typedef void* H2Session;

typedef struct {
    bool owned;
    size_t len;
    size_t cap;
    uint8_t* ptr;
} FfiSlice;

typedef struct {
    FfiSlice name;
    FfiSlice value;
} HeaderPair;

typedef struct {
    FfiStream stream;
    SockAddre addr;
} FfiBundle;



#ifdef __cplusplus
}
#endif