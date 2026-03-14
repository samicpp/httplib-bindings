from ffi import httplib, FfiSocket, FfiSlice, HeaderPair, HttpClient
from future import Future
# from ctypes import * # type: ignore
from typing import * # type: ignore
from typing_extensions import * # type: ignore
import ctypes, sys, os

class Socket:
    _ptr: FfiSocket

    def __init__(self, socket: FfiSocket) -> None:
        self._ptr = socket
        pass

    def __del__(self) -> None:
        httplib.http_free(self._ptr)
        pass

    
    @property
    def type(self) -> int:
        return httplib.http_get_type(self._ptr)
    
    
    async def readClient(self) -> None:
        fut = Future()
        httplib.http_read_client(fut.rsfut, self._ptr)
        await fut.fut
        pass

    async def readUntilComplete(self) -> None:
        fut = Future()
        httplib.http_read_until_complete(fut.rsfut, self._ptr)
        await fut.fut
        pass

    async def readUntilHeadComplete(self) -> None:
        fut = Future()
        httplib.http_read_until_head_complete(fut.rsfut, self._ptr)
        await fut.fut
        pass


    def setHeader(self, name: str, value: str) -> None:
        nam = FfiSlice.fromBytes(name.encode())
        val = FfiSlice.fromBytes(value.encode())
        pair = HeaderPair(name = nam, value = val)

        httplib.http_set_header(self._ptr, pair)
        pass

    def addHeader(self, name: str, value: str) -> None:
        nam = FfiSlice.fromBytes(name.encode())
        val = FfiSlice.fromBytes(value.encode())
        pair = HeaderPair(name = nam, value = val)

        httplib.http_add_header(self._ptr, pair)
        pass

    def delHeader(self, name: str) -> None:
        nam = FfiSlice.fromBytes(name.encode())
        httplib.http_del_header(self._ptr, nam)
        pass

    
    async def write(self, body: bytes) -> None:
        fut = Future()

        slice: FfiSlice
        if type(body) == bytes:
            slice = FfiSlice.fromBytes(body)
            pass
        else:
            slice = FfiSlice.fromBytes(str(body).encode())
            pass

        httplib.http_write(fut.rsfut, self._ptr, slice)
        await fut.fut
        pass

    async def close(self, body: bytes | str) -> None:
        fut = Future()

        slice: FfiSlice
        if type(body) == bytes:
            slice = FfiSlice.fromBytes(body)
            pass
        else:
            slice = FfiSlice.fromBytes(str(body).encode())
            pass

        httplib.http_close(fut.rsfut, self._ptr, slice)
        await fut.fut
        pass

    async def flush(self) -> None:
        fut = Future()
        httplib.http_flush(fut.rsfut, self._ptr)
        await fut.fut
        pass

    def getClient(self) -> HttpClient:
        return httplib.http_get_fficlient(self._ptr)
    pass