from ffi import httplib, FfiFuture, FfiSlice
from ctypes import * # type: ignore
import asyncio, ctypes, types, typing_extensions

_callback = CFUNCTYPE(None, c_void_p, c_void_p, use_errno=False, use_last_error=False)


class FfiException(Exception):
    errno: int
    errmsg: str
    def __init__(self, errno: c_int, errmsg: FfiSlice) -> None:
        self.errno = errno.value
        self.errmsg = errmsg.toBytes().decode("utf-8")
        super().__init__(f"{self.errno}: {self.errmsg}")
        pass
    pass

class Future:
    _loop: asyncio.AbstractEventLoop
    fut: asyncio.Future
    rsfut: FfiFuture
    __c_cb: typing_extensions.Any

    def _cb(self, userdata: c_void_p, result: c_void_p) -> None:
        state = httplib.ffi_future_state(self.rsfut)
        
        if state == 1:
            self._loop.call_soon_threadsafe(self.fut.set_result, result)
            pass
        else:
            errno = httplib.ffi_future_get_errno(self.rsfut)
            errmsg = httplib.ffi_future_get_errmsg(self.rsfut)
            
            self._loop.call_soon_threadsafe(self.fut.set_exception, FfiException(errno, errmsg))
            
            pass

        httplib.ffi_future_free(self.rsfut)
        pass

    def __init__(self) -> None:
        self._loop = asyncio.get_running_loop()
        self.__c_cb = _callback(self._cb)
        self.fut = self._loop.create_future()
        self.rsfut = httplib.ffi_future_new(self.__c_cb)
        pass
    pass
