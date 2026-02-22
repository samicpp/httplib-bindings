#pragma once
#include<cstdint>
#include <cstddef>


#ifdef __cplusplus
extern "C"{
#endif

bool init_rt();  // init rust tokio runtime
bool has_init();

typedef void* FfiFuture;
typedef void* FfiServer;
typedef void* FfiStream;
typedef void* SockAddre;

typedef void* FfiSocket;
typedef void* FfiReques;

typedef void* WebSocket;
typedef void* H2Session;

typedef void* TlsSerCon;
typedef void* TlsSniBui;


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


// futures are manual free
FfiFuture ffi_future_new(void (*cb)(void* userdata, void* result), void* userdata);
uint8_t ffi_future_state(FfiFuture fut);
void* ffi_future_result(FfiFuture fut);
void* ffi_future_take_result(FfiFuture fut);
void ffi_future_cancel(FfiFuture fut);
void ffi_future_complete(FfiFuture fut, void* result);
void ffi_future_free(FfiFuture fut);
void ffi_future_await(FfiFuture fut);
int ffi_future_get_errno(FfiFuture fut);
FfiSlice* ffi_future_get_errmsg(FfiFuture fut);

void free_slice(FfiSlice slice); // only when owned == true

long long add_i64(long long x, long long y); // test func


#ifdef __cplusplus
}
#endif