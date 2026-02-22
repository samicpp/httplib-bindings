#pragma once
#include<cstdint>
#include"base.h"

#ifdef __cplusplus
extern "C"{
#endif


typedef struct {
    bool fin;
    uint8_t rsv;
    uint8_t opcode;
    bool masked;
    FfiSlice payload;
} WsFrame;


void websocket_read_frame(FfiFuture fut, WebSocket ws); // resolves in WsFrame*
void websocket_free_frame(WsFrame* frame);
void websocket_flush(FfiFuture fut);                    // resolves in nothing
void websocket_free(WebSocket ws);


void websocket_send_continuation(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_continuation_masked(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_continuation_frag(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_continuation_masked_frag(FfiFuture fut, WebSocket ws, FfiSlice buf);

void websocket_send_text(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_text_masked(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_text_frag(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_text_masked_frag(FfiFuture fut, WebSocket ws, FfiSlice buf);

void websocket_send_binary(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_binary_masked(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_binary_frag(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_binary_masked_frag(FfiFuture fut, WebSocket ws, FfiSlice buf);

void websocket_send_close(FfiFuture fut, WebSocket ws, uint16_t code, FfiSlice buf);
void websocket_send_close_masked(FfiFuture fut, WebSocket ws, uint16_t code, FfiSlice buf);

void websocket_send_ping(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_ping_masked(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_pong(FfiFuture fut, WebSocket ws, FfiSlice buf);
void websocket_send_pong_masked(FfiFuture fut, WebSocket ws, FfiSlice buf);

#ifdef __cplusplus
}
#endif