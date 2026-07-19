use core::slice;
use std::ffi::{c_char, c_void};
use std::marker::PhantomData;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};

#[repr(C)] pub struct FfiFuture<T = c_void, U = c_void> { _unused: [u8; 0], _phantom: PhantomData<T>, _phantom2: PhantomData<U> }
#[repr(C)] pub struct DynStream { _unused: [u8; 0], _phantom: PhantomData<c_void> }
#[repr(C)] pub struct DynHttpRequest { _unused: [u8; 0], _phantom: PhantomData<c_void> }
#[repr(C)] pub struct DynHttpSocket { _unused: [u8; 0], _phantom: PhantomData<c_void> }
#[repr(C)] pub struct DynH2Sess { _unused: [u8; 0], _phantom: PhantomData<c_void> }
#[repr(C)] pub struct DynWebSocket { _unused: [u8; 0], _phantom: PhantomData<c_void> }
#[repr(C)] pub struct TcpListener { _unused: [u8; 0], _phantom: PhantomData<c_void> }
#[repr(C)] pub struct SocketAddr { _unused: [u8; 0], _phantom: PhantomData<c_void> }

#[repr(C)]
#[derive(Debug)]
pub struct FfiSlice{
    pub owned: u16,
    pub len: usize,
    pub cap: usize,
    pub ptr: *const u8,
}

impl FfiSlice{
    #[inline]
    pub const fn ro_mem(&self) -> bool { self.owned == 0 }
    pub const fn host_owns(&self) -> bool { self.owned == 1 }
    pub const fn is_owned(&self) -> bool { self.owned == 2 }

    pub fn from_string(string: String) -> Self{
        let bytes = string.into_bytes();
        let ptr = bytes.as_ptr();
        let len = bytes.len();
        let cap = bytes.capacity();
        std::mem::forget(bytes);

        Self {
            owned: 2,
            len,
            cap,
            ptr,
        }
    }
    pub fn from_vec(vec: Vec<u8>) -> Self{
        let ptr = vec.as_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        std::mem::forget(vec);

        Self {
            owned: 2,
            len,
            cap,
            ptr,
        }
    }
    pub const fn from_str(str_slice: &str) -> Self{
        let ptr = str_slice.as_ptr();
        let len = str_slice.len();

        Self {
            owned: 0,
            len,
            ptr,
            cap: len,
        }
    }
    pub const fn from_buf(slice: &[u8]) -> Self{
        let ptr = slice.as_ptr();
        let len = slice.len();

        Self {
            owned: 0,
            len,
            ptr,
            cap: len,
        }
    }

    pub const fn empty() -> Self{
        Self { len: 0, cap: 0, ptr: ptr::null(), owned: 0 }
    }

    pub fn free(self) {
        if self.is_owned() && self.ptr != ptr::null(){
            drop(self.to_vec());
        } 
        else if self.host_owns() && self.ptr != ptr::null() {
            unsafe { free_slice(self); }
        }
    }
    pub fn to_string(self) -> Option<String>{
        if !self.is_owned() {
            None
        }
        else {
            unsafe { Some(String::from_raw_parts(self.ptr as *mut u8, self.len, self.cap)) }
        }
    }
    pub fn to_vec(self) -> Option<Vec<u8>>{
        if !self.is_owned() { None }
        else{
            unsafe { Some(Vec::from_raw_parts(self.ptr as *mut u8, self.len, self.cap)) }
        }
    }
    pub fn to_owned(self) -> Self {
        if self.is_owned() { self }
        else {
            Self::from_vec(self.as_bytes().to_vec())
        }
    }
    pub const fn as_bytes(&self) -> &[u8]{
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
    pub const fn as_bytes_mut(&self) -> &mut [u8]{
        unsafe { slice::from_raw_parts_mut(self.ptr as *mut u8, self.len) }
    }
    pub const fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        str::from_utf8(self.as_bytes())
    }
    pub fn as_str_lossy(&self) -> std::borrow::Cow<'_, str>{
        String::from_utf8_lossy(self.as_bytes())
    }
    pub const unsafe fn as_bytes_static(&self) -> &'static [u8]{
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
    pub const unsafe fn as_bytes_mut_static(&self) -> &'static mut [u8]{
        unsafe { slice::from_raw_parts_mut(self.ptr as *mut u8, self.len) }
    }
}

unsafe impl Sync for FfiSlice{}
unsafe impl Send for FfiSlice{}

impl From<String> for FfiSlice{
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}
impl From<Vec<u8>> for FfiSlice{
    fn from(value: Vec<u8>) -> Self {
        Self::from_vec(value)
    }
}
impl From<&str> for FfiSlice{
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}
impl From<&[u8]> for FfiSlice{
    fn from(value: &[u8]) -> Self {
        Self::from_buf(value)
    }
}
impl From<&String> for FfiSlice{
    fn from(value: &String) -> Self {
        Self::from_str(value)
    }
}
impl From<&Vec<u8>> for FfiSlice{
    fn from(value: &Vec<u8>) -> Self {
        Self::from_buf(value)
    }
}
impl Drop for FfiSlice{
    fn drop(&mut self) {
        unsafe {
            if self.is_owned() {
                drop(Vec::from_raw_parts(self.ptr as *mut u8, self.len, self.cap));
            }
        }
    }
}

pub trait ToFfiSlice {
    fn to_ffi_slice(self) -> FfiSlice;
}
pub trait AsFfiSlice {
    fn as_ffi_slice(&self) -> FfiSlice;
}
impl<I: Into<FfiSlice>> ToFfiSlice for I {
    fn to_ffi_slice(self) -> FfiSlice {
        self.into()
    }
}
impl<I: AsRef<[u8]>> AsFfiSlice for I {
    fn as_ffi_slice(&self) -> FfiSlice {
        self.as_ref().into()
    }
}


#[repr(C)]
#[derive(Debug)]
pub struct FfiHeaderPair{
    pub nam: FfiSlice,
    pub val: FfiSlice,
}
impl FfiHeaderPair{
    pub const fn new(name: &str, value: &str) -> Self {
        Self { nam: FfiSlice::from_str(name), val: FfiSlice::from_str(value) }
    }
    pub fn new_owned(name: String, value: String) -> Self {
        Self { nam: FfiSlice::from_string(name), val: FfiSlice::from_string(value) }
    }
}
impl From<(&str, &str)> for FfiHeaderPair {
    fn from(value: (&str, &str)) -> Self {
        FfiHeaderPair::new(value.0, value.1)
    }
}
impl From<(String, String)> for FfiHeaderPair {
    fn from(value: (String, String)) -> Self {
        FfiHeaderPair::new_owned(value.0, value.1)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FfiResponse{
    pub owned: bool,
    pub valid: bool,

    pub head_complete: bool,
    pub body_complete: bool,

    pub code: u16,
    pub status: FfiSlice,

    pub headers_len: usize,
    pub headers_cap: usize,
    pub headers: *const FfiHeaderPair,
    pub body: FfiSlice,
}

#[repr(C)]
#[derive(Debug)]
pub struct FfiBundle{
    pub sock: *mut DynStream,
    pub addr: *const SocketAddr,
}

#[repr(C)]
#[derive(Debug)]
pub struct FfiClient{
    pub owned: bool,
    pub valid: bool,

    pub head_complete: bool,
    pub body_complete: bool,
    
    pub path: FfiSlice,
    pub method: u8,
    pub version: u8,
    pub method_str: FfiSlice,

    pub headers_len: usize,
    pub headers_cap: usize,
    pub headers: *const FfiHeaderPair,
    pub body: FfiSlice,

    pub host: FfiSlice,
    pub scheme: FfiSlice,
}

#[repr(C)]
#[derive(Debug)]
pub struct FfiDuoStream {
    pub one: *mut DynStream, // idk
    pub two: *mut DynStream, // 
}

#[repr(C)]
#[derive(Debug)]
pub struct FfiWsFrame{
    pub fin: bool,
    pub rsv: u8,
    pub opcode: u8,
    pub masked: bool,
    pub payload: FfiSlice,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiFutureStatus {
    Pending = 0,
    Success = 1,
    Error = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Methods {
    Unknown = 0,
    Get = 1,
    Head = 2,
    Post = 3,
    Put = 4,
    Delete = 5,
    Connect = 6,
    Options = 7,
    Trace = 8,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Versions{
    Unknown = 0,
    Debug = 1,
    HTTP09 = 2,
    HTTP10 = 3,
    HTTP11 = 4,
    HTTP2 = 5,
    HTTP3 = 6,
}


unsafe extern "C" {
    pub unsafe fn tcp_connect(fut: *const FfiFuture<DynStream>, addr: *mut i8);
    pub unsafe fn tcp_tls_connect(fut: *const FfiFuture<DynStream>, addr: *mut i8, domain: *mut i8, alpns: *mut i8);
    pub unsafe fn tcp_tls_connect_unverified(fut: *const FfiFuture<DynStream>, addr: *mut i8, domain: *mut i8, alpns: *mut i8);

    pub unsafe fn http1_request_new(stream: *mut DynStream, bufsize: usize) -> *mut DynHttpRequest;
    pub unsafe fn http_req_get_type(http: *mut DynHttpRequest) -> u8;
    pub unsafe fn http_req_set_header(req: *mut DynHttpRequest, pair: FfiHeaderPair);
    pub unsafe fn http_req_add_header(req: *mut DynHttpRequest, pair: FfiHeaderPair);
    pub unsafe fn http_req_del_header(req: *mut DynHttpRequest, name: FfiSlice);
    pub unsafe fn http_req_set_method_str(req: *mut DynHttpRequest, method: FfiSlice);
    pub unsafe fn http_req_set_method_byte(req: *mut DynHttpRequest, method: u8);
    pub unsafe fn http_req_set_path(req: *mut DynHttpRequest, path: FfiSlice);
    pub unsafe fn http_req_write(fut: *const FfiFuture<c_void>, req: *mut DynHttpRequest, buf: FfiSlice);
    pub unsafe fn http_req_send(fut: *const FfiFuture<c_void>, req: *mut DynHttpRequest, buf: FfiSlice);
    pub unsafe fn http_req_flush(fut: *const FfiFuture<c_void>, req: *mut DynHttpRequest);
    pub unsafe fn http_req_read(fut: *const FfiFuture<c_void>, req: *mut DynHttpRequest);
    pub unsafe fn http_req_read_until_complete(fut: *const FfiFuture<c_void>, req: *mut DynHttpRequest);
    pub unsafe fn http_req_read_until_head_complete(fut: *const FfiFuture<c_void>, req: *mut DynHttpRequest);
    pub unsafe fn http_req_free(req: *mut DynHttpRequest);

    pub unsafe fn http_response_get_status_code(req: *mut DynHttpRequest) -> u16;
    pub unsafe fn http_response_get_status_msg(req: *mut DynHttpRequest) -> FfiSlice;
    pub unsafe fn http_response_has_header(req: *mut DynHttpRequest, name: FfiSlice) -> bool;
    pub unsafe fn http_response_has_header_count(req: *mut DynHttpRequest, name: FfiSlice) -> usize;
    pub unsafe fn http_response_get_first_header(req: *mut DynHttpRequest, name: FfiSlice) -> FfiSlice;
    pub unsafe fn http_response_get_header(req: *mut DynHttpRequest, name: FfiSlice, index: usize) -> FfiSlice;
    pub unsafe fn http_response_get_body(req: *mut DynHttpRequest) -> FfiSlice;
    pub unsafe fn http_req_get_ffires(req: *mut DynHttpRequest) -> *const FfiResponse;
    pub unsafe fn http_req_free_ffires(res: *const FfiResponse);

    pub unsafe fn http1_websocket_strict(fut: *const FfiFuture<DynWebSocket>, http: *mut DynHttpRequest);
    pub unsafe fn http1_websocket_lazy(fut: *const FfiFuture<DynWebSocket>, http: *mut DynHttpRequest);
    pub unsafe fn http1_h2c_full(fut: *const FfiFuture<DynH2Sess>, http: *mut DynHttpRequest);

    pub unsafe fn http2_new(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess;
    pub unsafe fn http2_new_client(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess;
    pub unsafe fn http2_new_server(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess;
    pub unsafe fn http2_with(stream: *mut DynStream, bufsize: usize, mode: u8, strict: bool, settings: FfiSlice) -> *const DynH2Sess;
    pub unsafe fn http2_free(session: *const DynH2Sess);

    pub unsafe fn http2_read_preface(fut: *const FfiFuture<bool>, session: *const DynH2Sess);
    pub unsafe fn http2_send_preface(fut: *const FfiFuture, session: *const DynH2Sess);
    pub unsafe fn http2_next(fut: *const FfiFuture<u32>, session: *const DynH2Sess);
    pub unsafe fn http2_read_raw(fut: *const FfiFuture<FfiSlice>, session: *const DynH2Sess);
    pub unsafe fn http2_handle_raw(fut: *const FfiFuture<u32>, session: *const DynH2Sess, frame: FfiSlice);
    pub unsafe fn http2_open_stream(session: *const DynH2Sess) -> u32;

    pub unsafe fn http2_send_data(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, end: bool, buf: FfiSlice);
    pub unsafe fn http2_send_headers(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, end: bool, headers: *const FfiHeaderPair, length: usize);
    pub unsafe fn http2_send_priority(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, dependency: u32, weight: u8);
    pub unsafe fn http2_send_rst_stream(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, code: u32);
    pub unsafe fn http2_send_settings(fut: *const FfiFuture, session: *const DynH2Sess, settings: FfiSlice);
    pub unsafe fn http2_send_settings_default(fut: *const FfiFuture, session: *const DynH2Sess);
    pub unsafe fn http2_send_settings_default_no_push(fut: *const FfiFuture, session: *const DynH2Sess);
    pub unsafe fn http2_send_settings_maximum(fut: *const FfiFuture, session: *const DynH2Sess);
    pub unsafe fn http2_send_push_promise(fut: *const FfiFuture, session: *const DynH2Sess, associate_id: u32, promise_id: u32, headers: *const FfiHeaderPair, length: usize);
    pub unsafe fn http2_send_ping(fut: *const FfiFuture, session: *const DynH2Sess, ack: bool, buf: FfiSlice);
    pub unsafe fn http2_send_goaway(fut: *const FfiFuture, session: *const DynH2Sess, stream_id: u32, code: u32, buf: FfiSlice);

    pub unsafe fn http2_client_handler(session: *const DynH2Sess, stream_id: u32) -> *mut DynHttpRequest;
    pub unsafe fn http2_server_handler(session: *const DynH2Sess, stream_id: u32) -> *mut DynHttpSocket;

    pub unsafe fn tcp_server_new(fut: *const FfiFuture<TcpListener>, string: *mut c_char);
    pub unsafe fn tcp_server_from_fd(fd: i32) -> *mut TcpListener;
    pub unsafe fn tcp_server_free(listener: *mut TcpListener);
    pub unsafe fn tcp_server_accept(fut: *const FfiFuture<FfiBundle>, server: *mut TcpListener);

    pub unsafe fn addr_is_ipv4(addr: *const SocketAddr) -> bool;
    pub unsafe fn addr_is_ipv6(addr: *const SocketAddr) -> bool;
    pub unsafe fn get_addr_str(addr: *const SocketAddr) -> FfiSlice;
    pub unsafe fn tcp_detect_prot(fut: *const FfiFuture<u8>, stream: *mut DynStream);

    pub unsafe fn http1_new(ffi: *mut DynStream, bufsize: usize) -> *mut DynHttpSocket;
    pub unsafe fn http_get_type(http: *mut DynHttpSocket) -> u8;
    pub unsafe fn http_read_client(fut: *const FfiFuture, http: *mut DynHttpSocket);
    pub unsafe fn http_read_until_complete(fut: *const FfiFuture, http: *mut DynHttpSocket);
    pub unsafe fn http_read_until_head_complete(fut: *const FfiFuture, http: *mut DynHttpSocket);
    pub unsafe fn http_set_header(http: *mut DynHttpSocket, pair: FfiHeaderPair);
    pub unsafe fn http_add_header(http: *mut DynHttpSocket, pair: FfiHeaderPair);
    pub unsafe fn http_del_header(http: *mut DynHttpSocket, name: FfiSlice);
    pub unsafe fn http_write(fut: *const FfiFuture, http: *mut DynHttpSocket, buf: FfiSlice);
    pub unsafe fn http_close(fut: *const FfiFuture, http: *mut DynHttpSocket, buf: FfiSlice);
    pub unsafe fn http_flush(fut: *const FfiFuture, http: *mut DynHttpSocket);
    pub unsafe fn http_free(http: *mut DynHttpSocket);

    pub unsafe fn http_get_fficlient(http: *mut DynHttpSocket) -> *mut FfiClient;
    pub unsafe fn http_free_fficlient(client: *mut FfiClient);
    pub unsafe fn http_client_get_method(http: *mut DynHttpSocket) -> u8;
    pub unsafe fn http_client_get_method_str(http: *mut DynHttpSocket) -> FfiSlice;
    pub unsafe fn http_client_get_path(http: *mut DynHttpSocket) -> FfiSlice;
    pub unsafe fn http_client_get_version(http: *mut DynHttpSocket) -> u8;
    pub unsafe fn http_client_has_header(http: *mut DynHttpSocket, name: FfiSlice) -> bool;
    pub unsafe fn http_client_has_header_count(http: *mut DynHttpSocket, name: FfiSlice) -> usize;
    pub unsafe fn http_client_get_first_header(http: *mut DynHttpSocket, name: FfiSlice) -> FfiSlice;
    pub unsafe fn http_client_get_header(http: *mut DynHttpSocket, name: FfiSlice, index: usize) -> FfiSlice;
    pub unsafe fn http_client_get_body(http: *mut DynHttpSocket) -> FfiSlice;

    pub unsafe fn http1_direct_write(fut: *const FfiFuture, http: *mut DynHttpSocket, buf: FfiSlice);
    pub unsafe fn http1_websocket(fut: *const FfiFuture<DynWebSocket>, http: *mut DynHttpSocket);
    pub unsafe fn http1_h2c(fut: *const FfiFuture<DynH2Sess>, http: *mut DynHttpSocket);
    pub unsafe fn http1_h2_prior_knowledge(fut: *const FfiFuture<DynH2Sess>, http: *mut DynHttpSocket);

    pub unsafe fn tls_config_single_cert_pem(certs: FfiSlice, key: FfiSlice, alpns: *mut c_char) -> *const c_void; // ServerConfig
    pub unsafe fn tls_config_sni_builder() -> *const c_void; // TlsCertSelector
    pub unsafe fn tls_config_sni_builder_with_pem(def_certs: FfiSlice, def_key: FfiSlice) -> *const c_void; // TlsCertSelector
    pub unsafe fn tls_config_sni_add_pem(sni_build: *const c_void, domain: *mut c_char, certs: FfiSlice, key: FfiSlice) -> bool;
    pub unsafe fn tls_config_sni_builder_build(sni_build: *const c_void, alpns: *mut c_char) -> *const c_void; // ServerConfig
    pub unsafe fn tls_config_free(conf: *const c_void);
    pub unsafe fn tcp_upgrade_tls(fut: *const FfiFuture<DynStream>, stream: *mut DynStream, conf: *const c_void);

    pub unsafe fn create_duplex(bufsize: usize) -> FfiDuoStream;
    pub unsafe fn tcp_from_fd(fd: i32) -> *mut DynStream;
    pub unsafe fn unix_from_fd(fd: i32) -> *mut DynStream;
    pub unsafe fn tcp_to_fd(fd: *mut DynStream) -> *mut i32;
    pub unsafe fn unix_to_fd(fd: *mut DynStream) -> *mut i32;

    pub unsafe fn tcp_peek(fut: *const FfiFuture<usize>, ffi: *mut DynStream, buf: *mut FfiSlice);
    pub unsafe fn tls_get_alpn(stream: *mut DynStream) -> FfiSlice;
    pub unsafe fn stream_get_type(stream: *mut DynStream) -> u8;
    pub unsafe fn stream_read(fut: *const FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice);
    pub unsafe fn stream_read_exact(fut: *const FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice);
    pub unsafe fn stream_write(fut: *const FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice);
    pub unsafe fn stream_write_all(fut: *const FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice);
    pub unsafe fn stream_flush(fut: *const FfiFuture, stream: *mut DynStream);
    pub unsafe fn stream_shutdown(fut: *const FfiFuture, stream: *mut DynStream);
    pub unsafe fn stream_free(stream: *mut DynStream);

    pub unsafe fn websocket_read_frame(fut: *const FfiFuture<FfiWsFrame>, ws: *mut DynWebSocket);
    pub unsafe fn websocket_free_frame(frame: *mut FfiWsFrame);
    pub unsafe fn websocket_flush(fut: *const FfiFuture, ws: *mut DynWebSocket);
    pub unsafe fn websocket_free(ws: *mut DynWebSocket);
    pub unsafe fn websocket_send_continuation(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_continuation_masked(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_continuation_frag(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_continuation_masked_frag(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_text(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_text_masked(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_text_frag(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_text_masked_frag(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_binary(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_binary_masked(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_binary_frag(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_binary_masked_frag(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_close(fut: *const FfiFuture, ws: *mut DynWebSocket, code: u16, buf: FfiSlice);
    pub unsafe fn websocket_send_close_masked(fut: *const FfiFuture, ws: *mut DynWebSocket, code: u16, buf: FfiSlice);
    pub unsafe fn websocket_send_ping(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_ping_masked(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_pong(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);
    pub unsafe fn websocket_send_pong_masked(fut: *const FfiFuture, ws: *mut DynWebSocket, buf: FfiSlice);

    pub unsafe fn init_rt() -> bool;
    pub unsafe fn has_init() -> bool;
    pub unsafe fn ffi_future_new(cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, userdata: *mut c_void) -> *const FfiFuture<c_void>;
    pub unsafe fn ffi_future_state(fut: *const FfiFuture) -> u8;
    pub unsafe fn ffi_future_result(fut: *const FfiFuture<c_void>) -> *mut c_void;
    pub unsafe fn ffi_future_take_result(fut: *const FfiFuture<c_void>) -> *mut c_void;
    pub unsafe fn ffi_future_cancel(fut: *const FfiFuture);
    pub unsafe fn ffi_future_cancel_with_err(fut: *const FfiFuture, code: i32, msg: FfiSlice);
    pub unsafe fn ffi_future_complete(fut: *const FfiFuture<c_void>, result: *mut c_void);
    pub unsafe fn ffi_future_free(fut: *const FfiFuture);
    pub unsafe fn ffi_future_await(fut: *const FfiFuture);
    pub unsafe fn ffi_future_get_errno(fut: *const FfiFuture) -> i32;
    pub unsafe fn ffi_future_get_errmsg(fut: *const FfiFuture) -> *const FfiSlice;
    pub unsafe fn ffi_future_reset(fut: *const FfiFuture);
    pub unsafe fn ffi_future_get_userdata(fut: *const FfiFuture<c_void>) -> *mut c_void;
    pub unsafe fn ffi_future_set_userdata(fut: *const FfiFuture<c_void>, userdata: *mut c_void);
    pub unsafe fn rt_spawn_async_ffi_future(fut: async_ffi::FfiFuture<()>);
    
    pub unsafe fn free_slice(slice: FfiSlice);
    pub unsafe fn add_i64(x: i64, y: i64) -> i64;
    pub unsafe fn panic_test(message: *const c_char) -> !;
}


impl<T, U> FfiFuture<T, U> {
    pub fn new_raw(cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, userdata: *mut U) -> *const Self {
        unsafe {
            ffi_future_new(cb, userdata as *mut c_void) as *const Self
        }
    }
    pub fn new_no_cb(userdata: *mut U) -> *const Self {
        unsafe {
            ffi_future_new(None, userdata as *mut c_void) as *const Self
        }
    }
    pub fn new_cb(cb: extern "C" fn(*mut T, *mut U), userdata: *mut U) -> *const Self {
        unsafe {
            ffi_future_new(Some(std::mem::transmute(cb)), userdata as *mut c_void) as *const Self
        }
    }

    pub fn get_state(&self) -> FfiFutureStatus {
        match unsafe { ffi_future_state(self as *const _ as *const FfiFuture<c_void>) } {
            0 => FfiFutureStatus::Pending,
            1 => FfiFutureStatus::Success,
            2 => FfiFutureStatus::Error,
            _ => unreachable!(),
        }
    }
    pub fn get_result(&self) -> *mut T {
        unsafe { 
            ffi_future_result(self as *const _ as *const FfiFuture<c_void>) as *mut T 
        }
    }
    pub fn take_result(&self) -> *mut T {
        unsafe { 
            ffi_future_take_result(self as *const _ as *const FfiFuture<c_void>) as *mut T 
        }
    }
    pub fn cancel(&self) {
        unsafe { 
            ffi_future_cancel(self as *const _ as *const FfiFuture<c_void>)
        }
    }
    pub fn cancel_with_err(&self, code: i32, msg: FfiSlice) {
        unsafe { 
            ffi_future_cancel_with_err(self as *const _ as *const FfiFuture<c_void>, code, msg)
        }
    }
    pub fn complete(&self, result: *mut T) {
        unsafe { 
            ffi_future_complete(self as *const _ as *const FfiFuture<c_void>, result as *mut c_void)
        }
    }
    pub fn free(&self) {
        unsafe { 
            ffi_future_free(self as *const _ as *const FfiFuture<c_void>)
        }
    }
    pub fn blocking_await(&self) {
        unsafe { 
            ffi_future_await(self as *const _ as *const FfiFuture<c_void>)
        }
    }
    pub fn get_errno(&self) -> i32 {
        unsafe { 
            ffi_future_get_errno(self as *const _ as *const FfiFuture<c_void>)
        }
    }
    pub fn get_errmsg(&self) -> *const FfiSlice {
        unsafe { 
            ffi_future_get_errmsg(self as *const _ as *const FfiFuture<c_void>)
        }
    }
    pub fn reset(&self) {
        unsafe { 
            ffi_future_reset(self as *const _ as *const FfiFuture<c_void>) 
        }
    }
    pub fn get_userdata(&self) -> *mut U {
        unsafe { 
            ffi_future_get_userdata(self as *const _ as *const FfiFuture<c_void>) as *mut U
        }
    }
    pub fn set_result(&self, userdata: *mut U) {
        unsafe { 
            ffi_future_set_userdata(self as *const _ as *const FfiFuture<c_void>, userdata as *mut c_void)
        }
    }
}
impl<T, U> Drop for FfiFuture<T, U> {
    fn drop(&mut self) {
        self.free();
    }
}

impl<T> FfiFuture<T, Mutex<Option<Waker>>> {
    pub fn new_async() -> *const Self {
        unsafe {
            ffi_future_new(Some(ffi_wake_callback), Arc::into_raw(Arc::new(Mutex::new(Option::<Waker>::None))) as *mut c_void) as *const Self
        }
    }
}
impl<T> Future for FfiFuture<T, Mutex<Option<Waker>>> {
    type Output = Result<*mut T, i32>;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unsafe {
            match self.get_state() {
                FfiFutureStatus::Success => {
                    Poll::Ready(Ok(self.take_result()))
                }
                FfiFutureStatus::Error => {
                    Poll::Ready(Err(self.get_errno()))
                }
                FfiFutureStatus::Pending => {
                    let ud = self.get_userdata() as *const Mutex<Option<Waker>>;
                    if !ud.is_null(){
                        let arc = &*ud;
                        let mut guard = arc.lock().unwrap();
                        *guard = Some(cx.waker().clone());
                    }
                    Poll::Pending
                }
            }
        }
    }
}

extern "C" fn ffi_wake_callback(userdata: *mut c_void, _result: *mut c_void) {
    if userdata.is_null() { return; }
    unsafe {
        let arc = Arc::from_raw(userdata as *const Mutex<Option<Waker>>);
        if let Some(waker) = arc.lock().unwrap().take() {
            waker.wake();
        }
    }
}