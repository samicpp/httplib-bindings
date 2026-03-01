from ffi import httplib, FfiFuture, FfiSlice
from ctypes import * # type: ignore
import asyncio, ctypes, types, typing_extensions
from typing_extensions import Any

_callback = CFUNCTYPE(None, c_void_p, c_void_p, use_errno=False, use_last_error=False)


class FfiException(Exception):
    errno: int
    errmsg: str
    def __init__(self, errno: int, errmsg: str) -> None:
        self.errno = errno
        self.errmsg = errmsg
        super().__init__(f"{self.errno}: {self.errmsg}")
        pass
    pass

class Future:
    _loop: asyncio.AbstractEventLoop
    fut: asyncio.Future[c_void_p]
    rsfut: FfiFuture
    __c_cb: typing_extensions.Any
    result: c_void_p

    
    def _cb(self, userdata: c_void_p, result: c_void_p) -> None:
        state = httplib.ffi_future_state(self.rsfut)

        # print(f"result = {result}, state = {state}")
        
        if state == 1:
            self._loop.call_soon_threadsafe(self.fut.set_result, result)
            pass
        else:
            errno = httplib.ffi_future_get_errno(self.rsfut)
            errmsg = httplib.ffi_future_get_errmsg(self.rsfut)
            errmsg.contents.shouldFree = False
            errmsg = errmsg.contents.toBytes().decode("utf-8")
            
            # print(f"{errno} {errmsg}")

            self._loop.call_soon_threadsafe(self.fut.set_exception, FfiException(errno, errmsg))
            

            pass

        # self.result = result
        self.result = httplib.ffi_future_take_result(self.rsfut)
        # httplib.ffi_future_free(self.rsfut)
        pass

    def __init__(self, create: bool = False) -> None:
        self._loop = asyncio.new_event_loop() if create else asyncio.get_running_loop()
        self.__c_cb = _callback(self._cb)
        self.fut = self._loop.create_future()
        self.rsfut = httplib.ffi_future_new(self.__c_cb, None)
        pass

    def waitSync(self, timeout: float | None = None) -> c_void_p:
        self._loop.run_until_complete(asyncio.wait_for(self.fut, timeout))
        return self.result
    pass

