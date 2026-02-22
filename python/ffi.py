from ctypes import * # type: ignore
from typing import * # type: ignore
from typing_extensions import * # type: ignore
import ctypes, sys, os


if sys.platform == "linux":
    name = "libhttplib.so"
elif sys.platform == "darwin":
    name = "libhttplib.dylib"
elif sys.platform == "win32":
    name = "httplib.dll"
else:
    raise RuntimeError(f"Unsupported platform: {sys.platform}")


httplib = CDLL(f"./httplib/target/debug/{name}")

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


# structs

class FfiSlice(Structure):
    _fields_ = [
        ("owned", c_bool),
        ("length", c_size_t),
        ("_capacity", c_size_t),
        ("ptr", POINTER(c_ubyte)),
    ]

    def __del__(self):
        if self.owned:
            httplib.free_slice(self)
        return

    def toBytes(self) -> bytes:
        return string_at(self.ptr, self.length)
    
    @staticmethod
    def fromBytes(byt: bytes) -> FfiSlice:
        buf = (c_ubyte * len(byt))(*byt)
        ptr = ctypes.cast(buf, POINTER(c_ubyte))
        return FfiSlice(owned = False, length = len(byt), _capacity = len(byt), ptr = ptr)

    pass

class HeaderPair(Structure):
    _fields_ = [
        ("name", FfiSlice),
        ("value", FfiSlice),
    ]
    pass

class FfiBundle(Structure):
    _fields_ = [
        ("stream", FfiStream),
        ("addr", SockAddre),
    ]
    pass


# functions

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
