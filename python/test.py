import asyncio
from future import Future
from ffi import httplib, FfiBundle, HeaderPair, FfiSlice
from ctypes import cast, POINTER


async def serveTest(addr: str, create: bool = False):
    print("1. listening on " + addr)
    fut = Future(create)
    httplib.tcp_server_new(fut.rsfut, addr.encode())
    server = await fut.fut

    print("2. accepting incoming...")
    fut = Future(create)
    httplib.tcp_server_accept(fut.rsfut, server)
    bundlep = await fut.fut

    print("3. done")
    bundle = cast(bundlep, POINTER(FfiBundle))
    # bundle = bundle.contents

    print("4. creating socket")
    caddr = httplib.get_addr_str(bundle.contents.addr)
    print(f"{caddr.toBytes().decode()}")
    http = httplib.http1_new(bundle.contents.stream, 8 * 1024)

    print("5. reading..")
    fut = Future(create)
    httplib.http_read_until_head_complete(fut.rsfut, http)
    await fut.fut

    print("6. setting headers")
    httplib.http_set_header(http, HeaderPair.fromString("Content-Type", "text/plain"))
    httplib.http_set_header(http, HeaderPair.fromString("Connection", "close"))

    print("7. closing..")
    fut = Future(create)
    httplib.http_close(fut.rsfut, http, FfiSlice.fromBytes(b"pello, Hut brom fython"))
    await fut.fut

    print("8. free")
    httplib.http_free(http)
    pass


routine = serveTest("0.0.0.0:2001", False)
asyncio.run(routine)
