#pragma once
#include<cstdint>
#include"core.h"

#ifdef __cplusplus
extern "C"{
#endif


typedef void* Http2Sess;


Http2Sess http2_new(FfiStream stream, size_t bufsize);
Http2Sess http2_new_client(FfiStream stream, size_t bufsize);
Http2Sess http2_new_server(FfiStream stream, size_t bufsize);
Http2Sess http2_with(FfiStream stream, size_t bufsize, uint8_t mode, bool strict, FfiSlice settings);
void http2_free(Http2Sess session);


void http2_read_preface(FfiFuture fut, Http2Sess session);               // resolves in bool
void http2_send_preface(FfiFuture fut, Http2Sess session);               // resolves in nothing
void http2_next(FfiFuture fut, Http2Sess session);                       // resolves in u32, or null
void http2_read_raw(FfiFuture fut, Http2Sess session);                   // resolves in FfiSlice
void http2_handle_raw(FfiFuture fut, Http2Sess session, FfiSlice frame); // resolves in u32, or null
int  http2_open_stream(Http2Sess session);


void http2_send_data(FfiFuture fut, Http2Sess session, int stream_id, bool end, FfiSlice buf);
void http2_send_headers(FfiFuture fut, Http2Sess session, int stream_id, bool end, HeaderPair* headers, size_t length);

void http2_send_priority(FfiFuture fut, Http2Sess session, int stream_id, int dependency, uint8_t weight);

void http2_send_rst_stream(FfiFuture fut, Http2Sess session, int stream_id, int code);

void http2_send_settings(FfiFuture fut, Http2Sess session, FfiSlice settings);
void http2_send_settings_default(FfiFuture fut, Http2Sess session);
void http2_send_settings_default_no_push(FfiFuture fut, Http2Sess session);
void http2_send_settings_maximum(FfiFuture fut, Http2Sess session);

void http2_send_push_promise(FfiFuture fut, Http2Sess session, int associate_id, int promise_id, bool end, HeaderPair* headers, size_t length);

void http2_send_ping(FfiFuture fut, Http2Sess session, bool ack, FfiSlice buf);

void http2_send_goaway(FfiFuture fut, Http2Sess session, int stream_id, int code, FfiSlice message);

FfiReques http2_client_handler(Http2Sess session, int stream_id);
FfiSocket http2_server_handler(Http2Sess session, int stream_id);

#ifdef __cplusplus
}
#endif