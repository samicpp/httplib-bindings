from ctypes import * # type: ignore
from typing import * # type: ignore
from typing_extensions import * # type: ignore
import ctypes, sys, os

libname = "httplib"

if sys.platform == "linux":
    name = f"lib{libname}.so"
elif sys.platform == "darwin":
    name = f"lib{libname}.dylib"
elif sys.platform == "win32":
    name = f"{libname}.dll"
else:
    raise RuntimeError(f"Unsupported platform: {sys.platform}")


httplib = CDLL(f"../httplib/target/release/{name}")

httplib.init_rt.argtypes = []
httplib.init_rt.restype = c_bool

httplib.has_init.argtypes = []
httplib.has_init.restype = c_bool


if not(httplib.has_init()) and not(httplib.init_rt()):
    raise RuntimeError("Couldn't start tokio runtime")


def def_func(name: str, res: None | type[ctypes._CDataType], args: None | list[type[ctypes._CDataType]]):
    func = getattr(httplib, name)
    func.argtypes = args
    func.restype = res
    pass


# aliases

FfiFuture: TypeAlias = c_void_p
FfiServer: TypeAlias = c_void_p
FfiSocket: TypeAlias = c_void_p
FfiReques: TypeAlias = c_void_p
FfiStream: TypeAlias = c_void_p
SockAddre: TypeAlias = c_void_p
WebSocket: TypeAlias = c_void_p
H2Session: TypeAlias = c_void_p
TlsSerCon: TypeAlias = c_void_p
TlsSniBui: TypeAlias = c_void_p


# structs

class FfiSlice(Structure):
    shouldFree: bool = False
    _fields_ = [
        ("owned", c_bool),
        ("length", c_size_t),
        ("_capacity", c_size_t),
        ("ptr", POINTER(c_ubyte)),
    ]

    def __init__(self, *args: Any, **kw: Any) -> None:
        self.shouldFree = True
        super().__init__(*args, **kw)
        pass

    def free(self):
        # print(f"free called, should free = {self.shouldFree}")

        if self.shouldFree and self.owned:
            httplib.free_slice(self)
            self.shouldFree = False
        return

    def toBytes(self) -> bytes:
        return string_at(self.ptr, self.length)
    
    @staticmethod
    def fromBytes(byt: bytes) -> FfiSlice:
        buf = (c_ubyte * len(byt))(*byt)
        ptr = ctypes.cast(buf, POINTER(c_ubyte))
        return FfiSlice(owned = False, length = len(byt), _capacity = len(byt), ptr = ptr)


    def __enter__(self): return self
    def __del__(self): self.free()
    def __exit__(self): self.free()

    pass

class HeaderPair(Structure):
    _fields_ = [
        ("name", FfiSlice),
        ("value", FfiSlice),
    ]

    @staticmethod
    def fromString(name: str, value: str) -> HeaderPair:
        nam = FfiSlice.fromBytes(name.encode())
        val = FfiSlice.fromBytes(value.encode())
        return HeaderPair(name = nam, value = val)

    pass

class FfiBundle(Structure):
    _fields_ = [
        ("stream", FfiStream),
        ("addr", SockAddre),
    ]
    pass

class HttpClient(Structure):
    _fields_ = [
        ("_owned", c_bool),
        ("valid", c_bool),
        ("headComplete", c_bool),
        ("bodyComplete", c_bool),
        ("path", FfiSlice),
        ("method", c_ubyte),
        ("version", c_ubyte),
        ("methodStr", FfiSlice),
        ("headersLen", c_size_t),
        ("_headersCap", c_size_t),
        ("headers", HeaderPair),
        ("body", FfiSlice),
        ("host", FfiSlice),
        ("scheme", FfiSlice),
    ]
    def __del__(self):
        httplib.http_free_fficlient(self)
        return
    pass

class HttpResponse(Structure):
    _fields_ = [
        ("_owned", c_bool),
        ("valid", c_bool),
        ("headComplete", c_bool),
        ("bodyComplete", c_bool),
        ("code", c_uint16),
        ("status", FfiSlice),
        ("headersLen", c_size_t),
        ("_headersCap", c_size_t),
        ("headers", HeaderPair),
        ("body", FfiSlice),
    ]
    def __del__(self):
        httplib.http_req_free_ffires(self)
        return
    pass

class DuoStream(Structure):
    _fields_ = [
        ("one", FfiStream),
        ("two", FfiStream),
    ]
    pass

class WsFrame(Structure):
    _fields_ = [
        ("fin", c_bool),
        ("rsv", c_ubyte),
        ("opcode", c_ubyte),
        ("masked", c_bool),
        ("payload", FfiSlice),
    ]
    pass

# functions

## core

def_func("ffi_future_new", FfiFuture, [CFUNCTYPE(None, c_void_p, c_void_p, use_errno=False, use_last_error=False), c_void_p])
def_func("ffi_future_state", c_uint8, [FfiFuture])
def_func("ffi_future_result", c_void_p, [FfiFuture])
def_func("ffi_future_take_result", c_void_p, [FfiFuture])
def_func("ffi_future_cancel", None, [FfiFuture])
def_func("ffi_future_complete", None, [FfiFuture, c_void_p])
def_func("ffi_future_free", None, [FfiFuture])
def_func("ffi_future_await", None, [FfiFuture])
def_func("ffi_future_get_errno", c_int, [FfiFuture])
def_func("ffi_future_get_errmsg", POINTER(FfiSlice), [FfiFuture])

def_func("free_slice", None, [FfiSlice])
def_func("add_i64", c_longlong, [c_longlong, c_longlong])
def_func("panic_test", None, [c_char_p]) # dont use this


## utils

def_func("tls_get_alpn", FfiSlice, [FfiStream])
def_func("tcp_peek", None, [FfiFuture, FfiStream, FfiSlice])
def_func("stream_read", None, [FfiFuture, FfiStream, FfiSlice])
def_func("stream_write", None, [FfiFuture, FfiStream, FfiSlice])
def_func("stream_write_all", None, [FfiFuture, FfiStream, FfiSlice])
def_func("stream_free", None, [FfiStream])


## base

### server

def_func("tcp_server_new", None, [FfiFuture, c_char_p])
def_func("tcp_server_accept", None, [FfiFuture, FfiServer])

def_func("addr_is_ipv4", c_bool, [SockAddre])
def_func("addr_is_ipv6", c_bool, [SockAddre])
def_func("get_addr_str", FfiSlice, [SockAddre])

def_func("tcp_detect_prot", None, [FfiFuture, FfiStream])
def_func("http1_new", FfiSocket, [FfiStream, c_size_t])

def_func("http_get_type", c_ubyte, [FfiSocket])

def_func("http_read_client", None, [FfiFuture, FfiSocket])
def_func("http_read_until_complete", None, [FfiFuture, FfiSocket])
def_func("http_read_until_head_complete", None, [FfiFuture, FfiSocket])

def_func("http_set_header", None, [FfiSocket, HeaderPair])
def_func("http_add_header", None, [FfiSocket, HeaderPair])
def_func("http_del_header", None, [FfiSocket, FfiSlice])

def_func("http_write", None, [FfiFuture, FfiSocket, FfiSlice])
def_func("http_close", None, [FfiFuture, FfiSocket, FfiSlice])
def_func("http_flush", None, [FfiFuture, FfiSocket])

def_func("http_get_fficlient", POINTER(HttpClient), [FfiSocket])
def_func("http_free_fficlient", None, [POINTER(HttpClient)])

def_func("http_client_get_method", c_ubyte, [FfiSocket])
def_func("http_client_get_method_str", FfiSlice, [FfiSocket])
def_func("http_client_get_path", FfiSlice, [FfiSocket])
def_func("http_client_get_version", c_ubyte, [FfiSocket])

def_func("http_client_has_header", c_bool, [FfiSocket, FfiSlice])
def_func("http_client_has_header_count", c_size_t, [FfiSocket, FfiSlice])
def_func("http_client_get_first_header", FfiSlice, [FfiSocket, FfiSlice])
def_func("http_client_get_header", FfiSlice, [FfiSocket, FfiSlice, c_size_t])

def_func("http_client_get_body", FfiSlice, [FfiSocket, FfiSlice, c_size_t])

def_func("http_free", None, [FfiSocket])

def_func("http1_websocket", None, [FfiFuture, FfiSocket])
def_func("http1_h2c", None, [FfiFuture, FfiSocket])
def_func("http1_h2_prior_knowledge", None, [FfiFuture, FfiSocket])


### client

def_func("tcp_connect", None, [FfiFuture, c_char_p])
def_func("tcp_tls_connect", None, [FfiFuture, c_char_p, c_char_p, c_char_p])
def_func("tcp_tls_connect_unverified", None, [FfiFuture, c_char_p, c_char_p, c_char_p])

def_func("http1_request_new", FfiReques, [FfiStream, c_size_t])

def_func("http_req_get_type", c_ubyte, [FfiReques])

def_func("http_req_set_header", None, [FfiReques, HeaderPair])
def_func("http_req_add_header", None, [FfiReques, HeaderPair])
def_func("http_req_del_header", None, [FfiReques, FfiSlice])

def_func("http_req_set_method_str", None, [FfiReques, FfiSlice])
def_func("http_req_set_method_byte", None, [FfiReques, c_ubyte])
def_func("http_req_set_path", None, [FfiReques, FfiSlice])

def_func("http_req_write", None, [FfiFuture, FfiReques, FfiSlice])
def_func("http_req_send", None, [FfiFuture, FfiReques, FfiSlice])
def_func("http_req_flush", None, [FfiFuture, FfiReques])

def_func("http_req_read", None, [FfiFuture, FfiReques])
def_func("http_req_read_until_complete", None, [FfiFuture, FfiReques])
def_func("http_req_read_until_head_complete", None, [FfiFuture, FfiReques])

def_func("http_response_get_status_code", c_uint16, [FfiReques])
def_func("http_response_get_status_msg", FfiSlice, [FfiReques])

def_func("http_response_has_header", c_bool, [FfiReques, FfiSlice])
def_func("http_response_has_header_count", c_size_t, [FfiReques, FfiSlice])
def_func("http_response_get_first_header", FfiSlice, [FfiReques, FfiSlice])
def_func("http_response_get_header", FfiSlice, [FfiReques, FfiSlice, c_size_t])

def_func("http_response_get_body", FfiSlice, [FfiReques, FfiSlice, c_size_t])

def_func("http_req_get_ffires", POINTER(HttpResponse), [FfiReques])
def_func("http_req_free_ffires", None, [POINTER(HttpResponse)])

def_func("http_req_free", None, [FfiReques])

def_func("http1_websocket_strict", None, [FfiFuture, FfiReques])
def_func("http1_websocket_lazy", None, [FfiFuture, FfiReques])
def_func("http1_h2c_full", None, [FfiFuture, FfiReques])


## http2

def_func("http2_new", H2Session, [FfiStream, c_size_t])
def_func("http2_new_client", H2Session, [FfiStream, c_size_t])
def_func("http2_new_server", H2Session, [FfiStream, c_size_t])
def_func("http2_with", H2Session, [FfiStream, c_size_t, c_ubyte, c_bool, FfiSlice])
def_func("http2_free", None, [H2Session])

def_func("http2_read_preface", None, [FfiFuture, H2Session])
def_func("http2_send_preface", None, [FfiFuture, H2Session])
def_func("http2_next", None, [FfiFuture, H2Session])
def_func("http2_read_raw", None, [FfiFuture, H2Session])
def_func("http2_handle_raw", None, [FfiFuture, H2Session, FfiSlice])
def_func("http2_open_stream", c_int, [H2Session])

def_func("http2_send_data", None, [FfiFuture, H2Session, c_int, c_bool, FfiSlice])
def_func("http2_send_headers", None, [FfiFuture, H2Session, c_int, c_bool, POINTER(HeaderPair), c_size_t])

def_func("http2_send_priority", None, [FfiFuture, H2Session, c_int, c_int, c_ubyte])

def_func("http2_send_rst_stream", None, [FfiFuture, H2Session, c_int, c_int])

def_func("http2_send_settings", None, [FfiFuture, H2Session, FfiSlice])
def_func("http2_send_settings_default", None, [FfiFuture, H2Session])
def_func("http2_send_settings_default_no_push", None, [FfiFuture, H2Session])
def_func("http2_send_settings_maximum", None, [FfiFuture, H2Session])

def_func("http2_send_push_promise", None, [FfiFuture, H2Session, c_int, c_int, POINTER(HeaderPair), c_size_t])

def_func("http2_send_ping", None, [FfiFuture, H2Session, c_bool, FfiSlice])

def_func("http2_send_goaway", None, [FfiFuture, H2Session, c_int, c_int, FfiSlice])

def_func("http2_client_handler", FfiReques, [H2Session, c_int])
def_func("http2_server_handler", FfiReques, [H2Session, c_int])


## tls-server

def_func("tls_config_single_cert_pem", TlsSerCon, [FfiSlice, FfiSlice, c_char_p])
def_func("tls_config_sni_builder", TlsSniBui, [])
def_func("tls_config_sni_builder_with_pem", TlsSniBui, [FfiSlice, FfiSlice])
def_func("tls_config_sni_add_pem", c_bool, [TlsSniBui, c_char_p, FfiSlice, FfiSlice])
def_func("tls_config_sni_builder_build", TlsSerCon, [TlsSniBui, c_char_p])
def_func("tls_config_free", None, [TlsSerCon])
def_func("tcp_upgrade_tls", None, [FfiFuture, FfiStream, TlsSerCon])


## websocket

def_func("websocket_read_frame", None, [FfiFuture, WebSocket])
def_func("websocket_free_frame", None, [WsFrame])
def_func("websocket_flush", None, [FfiFuture, WebSocket])
def_func("websocket_free", None, [WebSocket])


def_func("websocket_send_continuation", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_continuation_masked", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_continuation_frag", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_continuation_masked_frag", None, [FfiFuture, WebSocket, FfiSlice])

def_func("websocket_send_text", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_text_masked", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_text_frag", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_text_masked_frag", None, [FfiFuture, WebSocket, FfiSlice])

def_func("websocket_send_binary", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_binary_masked", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_binary_frag", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_binary_masked_frag", None, [FfiFuture, WebSocket, FfiSlice])

def_func("websocket_send_close", None, [FfiFuture, WebSocket, c_uint16, FfiSlice])
def_func("websocket_send_close_masked", None, [FfiFuture, WebSocket, c_uint16, FfiSlice])

def_func("websocket_send_ping", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_ping_masked", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_pong", None, [FfiFuture, WebSocket, FfiSlice])
def_func("websocket_send_pong_masked", None, [FfiFuture, WebSocket, FfiSlice])
