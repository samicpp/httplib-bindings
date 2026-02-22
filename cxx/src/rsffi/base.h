#pragma once
#include<cstdint>
#include"core.h"

#ifdef __cplusplus
extern "C"{
#endif


typedef struct {
    bool owned;
    bool valid;

    bool headComplete;
    bool bodyComplete;

    FfiSlice path;
    uint8_t method;
    uint8_t version;
    FfiSlice methodStr;

    size_t headersLen;
    size_t headersCap;
    HeaderPair* headers;
    FfiSlice body;

    FfiSlice host;
    FfiSlice scheme;
} HttpClient;


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


// server

void tcp_server_new(FfiFuture fut, char* addr_cstr); // resolves in FfiServer
void tcp_server_accept(FfiFuture fut, FfiServer server); // resolves in FfiBundle
// void server_loop(FfiFuture fut, FfiServer server, void (*cb)(FfiBundle));

bool addr_is_ipv4(SockAddre bundle);
bool addr_is_ipv6(SockAddre bundle);
FfiSlice get_addr_str(SockAddre bundle);  // manual free

void tcp_detect_prot(FfiFuture fut, FfiStream bundle); // detect wether tls, http2 preface, or rest
FfiSocket http1_new(FfiStream bundle, size_t bufsize);

uint8_t http_get_type(FfiSocket http); // http type

void http_read_client(FfiFuture fut, FfiSocket http);              // resolves in nothing
void http_read_until_complete(FfiFuture fut, FfiSocket http);      // nothing
void http_read_until_head_complete(FfiFuture fut, FfiSocket http); // nothing

void http_set_header(FfiSocket http, HeaderPair pair);
void http_add_header(FfiSocket http, HeaderPair pair);
void http_del_header(FfiSocket http, HeaderPair pair);

void http_write(FfiFuture fut, FfiSocket http, FfiSlice bytes); // resolves in nothing
void http_close(FfiFuture fut, FfiSocket http, FfiSlice bytes); // nothing
void http_flush(FfiFuture fut, FfiSocket http);                 // nothing

HttpClient* http_get_fficlient(FfiSocket http); // manual free
void http_free_fficlient(HttpClient* client);

uint8_t http_client_get_method(FfiSocket http);
FfiSlice http_client_get_method_str(FfiSocket http);
FfiSlice http_client_get_path(FfiSocket http);
uint8_t http_client_get_version(FfiSocket http); // client version (like http type but includes 0.9, 1.0, unknown (type 1 only), and debug)

bool http_client_has_header(FfiSocket http, FfiSlice name);
size_t http_client_has_header_count(FfiSocket http, FfiSlice name);
FfiSlice http_client_get_first_header(FfiSocket http, FfiSlice name);
FfiSlice http_client_get_header(FfiSocket http, FfiSlice name, size_t index);

FfiSlice http_client_get_body(FfiSocket http);

void http_free(FfiSocket http);

void http1_direct_write(FfiFuture fut, FfiSocket http, FfiSlice bytes); // resolves in nothing

void http1_websocket(FfiFuture fut, FfiSocket http);          // resolves in WebSocket
void http1_h2c(FfiFuture fut, FfiSocket http);                // resolves in Http2Session
void http1_h2_prior_knowledge(FfiFuture fut, FfiSocket http); // resolves in Http2Session




typedef struct {
    bool owned;
    bool valid;

    bool headComplete;
    bool bodyComplete;

    uint16_t code;
    FfiSlice status;

    size_t headersLen;
    size_t headersCap;
    HeaderPair* headers;
    FfiSlice body;
} HttpResponse;


void tcp_connect(FfiFuture fut, char* addr); // resolves in FfiStream

void tcp_tls_connect(FfiFuture fut, char* addr, char* domain, size_t len, FfiSlice* alpns);            // resolves in FfiStream
void tcp_tls_connect_unverified(FfiFuture fut, char* addr, char* domain, size_t len, FfiSlice* alpns); // resolves in FfiStream

FfiReques http1_request_new(FfiStream stream, size_t bufsize);

uint8_t http_req_get_type(FfiReques req);

void http_req_set_header(FfiReques req, HeaderPair pair);
void http_req_add_header(FfiReques req, HeaderPair pair);
void http_req_del_header(FfiReques req, HeaderPair pair);

void http_req_set_method_str(FfiReques req, FfiSlice method);
void http_req_set_method_byte(FfiReques req, uint8_t method);
void http_req_set_path(FfiReques req, FfiSlice method);

void http_req_write(FfiFuture fut, FfiReques req, FfiSlice bytes);    // resolves in nothing
void http_req_send(FfiFuture fut, FfiReques req, FfiSlice bytes);     // nothing
void http_req_flush(FfiFuture fut, FfiReques req);                    // nothing

void http_req_read_client(FfiFuture fut, FfiReques req);              // nothing
void http_req_read_until_complete(FfiFuture fut, FfiReques req);      // nothing
void http_req_read_until_head_complete(FfiFuture fut, FfiReques req); // nothing

uint16_t http_response_get_status_code(FfiReques req);
FfiSlice http_response_get_status_msg(FfiReques req);

bool http_req_response_has_header(FfiReques req, FfiSlice name);
size_t http_req_response_has_header_count(FfiReques req, FfiSlice name);
FfiSlice http_req_response_get_first_header(FfiReques req, FfiSlice name);
FfiSlice http_req_response_get_header(FfiReques req, FfiSlice name, size_t index);

FfiSlice http_req_response_get_body(FfiReques req);

HttpResponse* http_req_get_ffires(FfiReques req);
void http_req_free_ffires(HttpResponse* res);

void http_req_free(FfiReques req);

void http1_websocket_strict(FfiFuture fut, FfiSocket http);          // resolves in WebSocket
void http1_websocket_lazy(FfiFuture fut, FfiSocket http);            // resolves in WebSocket
void http1_h2c_full(FfiFuture fut, FfiSocket http);                  // resolves in Http2Session


#ifdef __cplusplus
}
#endif