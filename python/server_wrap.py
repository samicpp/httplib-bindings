from ffi import httplib, FfiStream, FfiServer, FfiBundle
from future import Future
from ctypes import c_ubyte, POINTER, c_void_p, cast
from typing import * # type: ignore
from typing_extensions import * # type: ignore
import ctypes, sys, os

class TcpServer():
    _ptr: FfiServer

    def __init__(self, ptr: FfiServer) -> None:
        self._ptr = ptr
        pass

    @staticmethod
    async def listen(addr: str) -> TcpServer:
        addr = str(addr)
        fut = Future()
        httplib.tcp_server_new(fut.rsfut, addr.encode())
        ptr = await fut.fut
        return TcpServer(ptr)
    
    async def accept(self) -> Stream:
        fut = Future()
        httplib.tcp_server_accept(fut.rsfut, self._ptr)
        bundlep: c_void_p = await fut.fut
        bundle = ctypes.cast(bundlep, POINTER(FfiBundle))
        return Stream(bundle.contents.stream)

    pass

class Stream():
    ptr: FfiStream

    def __init__(self, ptr: FfiStream) -> None:
        self.ptr = ptr
        pass

    @property
    def type(self) -> c_ubyte:
        return httplib.stream_get_type(self.ptr)
    

    pass
