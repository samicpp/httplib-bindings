use std::{ffi::{CStr, c_void}, ptr};

use http::{http1::client::Http1Request, http2::session::Http2Session, shared::{HttpMethod, HttpRequest, HttpResponse, HttpType}, websocket::socket::WebSocket};
use httprs_core::ffi::{futures::FfiFuture, slice::FfiSlice};
use tokio::io::{BufReader, ReadHalf, WriteHalf};

use crate::{DynStream, clients::{DynHttpRequest, tcp_connect as ntcpconn, tls_upgrade, tls_upgrade_no_verification}, errno::TYPE_ERR, ffi::{const_enums::methods, server::FfiHeaderPair, utils::heap_ptr}, spawn_task_with};

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
impl FfiResponse{
    pub fn from(response: &HttpResponse) -> Self {
        let mut pairs = Vec::new();
        response.headers.iter().for_each(|(h,vs)|vs.into_iter().for_each(|v| pairs.push(FfiHeaderPair { nam: FfiSlice::from_str(h), val: FfiSlice::from_str(v) })));
        let pair_ptr = pairs.as_ptr();
        let pairs_len = pairs.len();
        let pairs_cap = pairs.capacity();
        std::mem::forget(pairs);

        Self { 
            owned: false,
            valid: response.valid,
            head_complete: response.head_complete,
            body_complete: response.body_complete,
            code: response.code,
            status: response.status.as_str().into(),
            headers_len: pairs_len,
            headers_cap: pairs_cap,
            headers: pair_ptr,
            body: FfiSlice::from_buf(&response.body),
        }
    }
    pub fn from_owned(response: HttpResponse) -> Self {
        let mut pairs = Vec::new();
        response.headers.into_iter().for_each(|(h,vs)|vs.into_iter().for_each(|v| pairs.push(FfiHeaderPair { nam: FfiSlice::from_string(h.clone()), val: FfiSlice::from_string(v) })));
        let pair_ptr = pairs.as_ptr();
        let pairs_len = pairs.len();
        let pairs_cap = pairs.capacity();
        std::mem::forget(pairs);

        Self { 
            owned: true,
            valid: response.valid,
            head_complete: response.head_complete,
            body_complete: response.body_complete,
            code: response.code,
            status: response.status.into(),
            headers_len: pairs_len,
            headers_cap: pairs_cap,
            headers: pair_ptr,
            body: response.body.into(),
        }
    }
    
    pub fn free(self){
        let pairs = unsafe { Vec::from_raw_parts(self.headers as *mut FfiHeaderPair, self.headers_len, self.headers_cap) };
        
        if self.owned{
            self.status.free();
            self.body.free();

            for h in pairs {
                h.nam.free();
                h.val.free();
            }
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn tcp_connect(fut: *mut FfiFuture<DynStream>, addr: *mut i8){
    unsafe{
        let addr = CStr::from_ptr(addr).to_string_lossy().to_string();
        let fut = &*fut;

        spawn_task_with(fut, async move {
            let tcp = ntcpconn(addr).await?;
            let ptr = heap_ptr(DynStream::from(tcp));
            Ok(ptr)
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_tls_connect(fut: *mut FfiFuture<DynStream>, addr: *mut i8, domain: *mut i8, alpns: *mut i8){
    unsafe{
        let addr = CStr::from_ptr(addr).to_string_lossy().to_string();
        let domain = CStr::from_ptr(domain).to_string_lossy().to_string();
        let alpns = CStr::from_ptr(alpns).to_string_lossy().to_string();
        let alpns = alpns.split(',').map(|s|s.as_bytes().to_vec()).collect();
        let fut = &*fut;

        spawn_task_with(fut, async move {
            let tcp = ntcpconn(addr).await?;
            let tls = tls_upgrade(tcp, domain, alpns).await?;
            let ptr = heap_ptr(DynStream::from(tls));
            Ok(ptr)
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn tcp_tls_connect_unverified(fut: *mut FfiFuture<DynStream>, addr: *mut i8, domain: *mut i8, alpns: *mut i8){
    unsafe{
        let addr = CStr::from_ptr(addr).to_string_lossy().to_string();
        let domain = CStr::from_ptr(domain).to_string_lossy().to_string();
        let alpns = CStr::from_ptr(alpns).to_string_lossy().to_string();
        let alpns = alpns.split(',').map(|s|s.as_bytes().to_vec()).collect();
        let fut = &*fut;

        spawn_task_with(fut, async move {
            let tcp = ntcpconn(addr).await?;
            let tls = tls_upgrade_no_verification(tcp, domain, alpns).await?;
            let ptr = heap_ptr(DynStream::from(tls));
            Ok(ptr)
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_request_new(stream: *mut DynStream, bufsize: usize) -> *mut DynHttpRequest{
    unsafe{
        let stream = *Box::from_raw(stream);
        let dreq = Http1Request::new(stream, bufsize).into();
        heap_ptr(dreq)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_get_type(http: *mut DynHttpRequest) -> u8{
    unsafe {
        match (*http).get_type() {
            HttpType::Http1 => 1,
            HttpType::Http2 => 2,
            HttpType::Http3 => 3,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_header(req: *mut DynHttpRequest, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str_lossy();
        let value = pair.val.as_str_lossy();

        (*req).set_header(&name, value.into_owned());
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_add_header(req: *mut DynHttpRequest, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str_lossy();
        let value = pair.val.as_str_lossy();

        (*req).add_header(&name, value.into_owned());
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_del_header(req: *mut DynHttpRequest, name: FfiSlice){
    unsafe{
        let name = name.as_str_lossy();
        let _ = (*req).del_header(&name);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_method_str(req: *mut DynHttpRequest, method: FfiSlice){
    unsafe{
        let meth = method.as_str_lossy().as_ref().into();
        (*req).set_method(meth);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_method_byte(req: *mut DynHttpRequest, method: u8){
    unsafe{
        let meth = match method{
            methods::GET => HttpMethod::Get,
            methods::HEAD => HttpMethod::Head,
            methods::POST => HttpMethod::Post,
            methods::PUT => HttpMethod::Put,
            methods::DELETE => HttpMethod::Delete,
            methods::CONNECT => HttpMethod::Connect,
            methods::OPTIONS => HttpMethod::Options,
            methods::TRACE => HttpMethod::Trace,
            _ => HttpMethod::Unknown(None),
        };
        (*req).set_method(meth);
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_set_path(req: *mut DynHttpRequest, path: FfiSlice){
    unsafe{
        let path = path.as_str_lossy().to_string();
        (*req).set_path(path);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_write(fut: *mut FfiFuture<c_void>, req: *mut DynHttpRequest, buf: FfiSlice){
    unsafe{
        let req = &mut *req;
        let fut = &*fut;
        spawn_task_with(fut, async move{
            req.write(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_send(fut: *mut FfiFuture<c_void>, req: *mut DynHttpRequest, buf: FfiSlice){
    unsafe{
        let req = &mut *req;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            req.send(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_flush(fut: *mut FfiFuture<c_void>, req: *mut DynHttpRequest){
    unsafe{
        let fut = &*fut;
        let req = &mut *req;
        spawn_task_with(fut, async move{
            req.flush().await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_read(fut: *mut FfiFuture<c_void>, req: *mut DynHttpRequest){
    unsafe{
        let req = &mut *req;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            req.read_response().await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_read_until_complete(fut: *mut FfiFuture<c_void>, req: *mut DynHttpRequest){
    unsafe{
        let req = &mut *req;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            req.read_until_complete().await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_read_until_head_complete(fut: *mut FfiFuture<c_void>, req: *mut DynHttpRequest){
    unsafe{
        let req = &mut *req;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            req.read_until_head_complete().await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_status_code(req: *mut DynHttpRequest) -> u16 {
    unsafe {
        (*req).get_response().code
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_status_msg(req: *mut DynHttpRequest) -> FfiSlice {
    unsafe {
        (&(*req).get_response().status).into()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_has_header(req: *mut DynHttpRequest, name: FfiSlice) -> bool {
    unsafe{
        (*req).get_response().headers.contains_key(name.as_str_lossy().as_ref())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_has_header_count(req: *mut DynHttpRequest, name: FfiSlice) -> usize {
    unsafe{
        (*req).get_response().headers.get(name.as_str_lossy().as_ref()).and_then(|h|Some(h.len())).unwrap_or(0)
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_first_header(req: *mut DynHttpRequest, name: FfiSlice) -> FfiSlice {
    unsafe{
        (*req).get_response().headers.get(name.as_str_lossy().as_ref()).and_then(|h|Some(FfiSlice::from_string(h[0].clone()))).unwrap_or(FfiSlice::empty())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_header(req: *mut DynHttpRequest, name: FfiSlice, index: usize) -> FfiSlice {
    unsafe{
        (*req).get_response().headers.get(name.as_str_lossy().as_ref()).and_then(
            |h|h.get(index)
            .and_then(|h|Some(FfiSlice::from_string(h.clone())))
        ).unwrap_or(FfiSlice::empty())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_response_get_body(req: *mut DynHttpRequest) -> FfiSlice {
    unsafe {
        (&(*req).get_response().body).into()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_get_ffires(req: *mut DynHttpRequest) -> *const FfiResponse {
    unsafe {
        heap_ptr(FfiResponse::from(&(*req).get_response()))
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_req_free_ffires(res: *mut FfiResponse) {
    unsafe {
        drop(Box::from_raw(res))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_req_free(req: *mut DynHttpRequest){
    unsafe{
        drop(Box::from_raw(req));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_websocket_strict(fut: *mut FfiFuture<WebSocket<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>>, http: *mut DynHttpRequest){
    unsafe{
        let http = *Box::from_raw(http);
        let fut = &*fut;
        
        match http {
            DynHttpRequest::Http1(one) => {
                spawn_task_with(fut, async move {
                    let ws = one.websocket_strict().await?;
                    Ok(heap_ptr(ws))
                })
            }
            _ => fut.cancel_with_err(TYPE_ERR, "not http1".into()),
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http1_websocket_lazy(fut: *mut FfiFuture<WebSocket<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>>, http: *mut DynHttpRequest){
    unsafe{
        let http = *Box::from_raw(http);
        let fut = &*fut;

        match http {
            DynHttpRequest::Http1(one) => {
                spawn_task_with(fut, async move {
                    let ws = one.websocket_lazy().await?;
                    Ok(heap_ptr(ws))
                })
            }
            _ => fut.cancel_with_err(TYPE_ERR, "not http1".into()),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_h2c_full(fut: *mut FfiFuture<Http2Session<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>>, http: *mut DynHttpRequest){
    unsafe{
        let http = *Box::from_raw(http);
        let fut = &*fut;
        
        match http {
            DynHttpRequest::Http1(one) => {
                spawn_task_with(fut, async move {
                    let h2 = one.h2c_full(None).await?;
                    Ok(heap_ptr(h2))
                })
            }
            _ => fut.cancel_with_err(TYPE_ERR, "not http1".into()),
        }
    }
}
pub mod methods{
    pub const UNKNOWN: u8 = 0;
    pub const GET: u8 = 1;
    pub const HEAD: u8 = 2;
    pub const POST: u8 = 3;
    pub const PUT: u8 = 4;
    pub const DELETE: u8 = 5;
    pub const CONNECT: u8 = 6;
    pub const OPTIONS: u8 = 7;
    pub const TRACE: u8 = 8;
}
pub mod versions{
    pub const UNKNOWN: u8 = 0;
    pub const DEBUG: u8 = 1;
    pub const HTTP09: u8 = 2;
    pub const HTTP10: u8 = 3;
    pub const HTTP11: u8 = 4;
    pub const HTTP2: u8 = 5;
    pub const HTTP3: u8 = 6;
}
use core::slice;
use std::{ffi::c_void, ptr, sync::Arc};

use http::{http2::{client::Http2Request, core::{Http2Frame, Http2Settings}, server::Http2Socket, session::{Http2Session, Mode}}};
use httprs_core::ffi::{futures::FfiFuture, slice::{FfiSlice, ToFfiSlice}};
use tokio::io::{BufReader, ReadHalf, WriteHalf};

use crate::{DynStream, clients::DynHttpRequest, ffi::{server::FfiHeaderPair, utils::{heap_ptr}}, servers::DynHttpSocket, spawn_task_with};

pub type DynH2Sess = Http2Session<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>;



#[unsafe(no_mangle)]
pub extern "C" fn http2_new(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);
        let h2 = Http2Session::with(netr, netw, Mode::Ambiguous, true, Http2Settings::default());
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_new_client(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);
        let h2 = Http2Session::with(netr, netw, Mode::Client, true, Http2Settings::default());
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_new_server(stream: *mut DynStream, bufsize: usize) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);
        let h2 = Http2Session::with(netr, netw, Mode::Server, true, Http2Settings::default());
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_with(stream: *mut DynStream, bufsize: usize, mode: u8, strict: bool, settings: FfiSlice) -> *const DynH2Sess {
    unsafe {
        let stream = *Box::from_raw(stream);
        let (netr, netw) = tokio::io::split(stream);
        let netr = BufReader::with_capacity(bufsize, netr);

        let mode = match mode {
            1 => Mode::Client,
            2 => Mode::Server,
            _ => Mode::Ambiguous,
        };

        let settings = Http2Settings::from(settings.as_bytes());

        let h2 = Http2Session::with(netr, netw, mode, strict, settings);
        let h2 = Arc::into_raw(Arc::new(h2));
        h2
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_free(session: *const DynH2Sess) {
    unsafe {
        drop(Arc::from_raw(session));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_read_preface(fut: *const FfiFuture<bool>, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            Ok(heap_ptr(sess.read_preface().await?))
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_preface(fut: *const FfiFuture<c_void>, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            sess.send_preface().await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_next(fut: *const FfiFuture<u32>, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            if let Some(open) = sess.next().await? {
                Ok(heap_ptr(open))
            }
            else {
                Ok(ptr::null_mut())
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_read_raw(fut: *const FfiFuture<FfiSlice>, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            let frame = sess.read_frame().await?;
            Ok(heap_ptr(frame.source.to_ffi_slice()))
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_handle_raw(fut: *const FfiFuture<u32>, session: *const DynH2Sess, frame: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let frame = if frame.owned { Http2Frame::from_owned(frame.to_vec().unwrap()) } else { Http2Frame::from_borrow(frame.as_bytes_static()) };
        let frame = if let Some(frame) = frame { frame } else { return };

        spawn_task_with(fut, async move{
            if let Some(open) = sess.handle(frame).await? {
                Ok(heap_ptr(open))
            }
            else {
                Ok(ptr::null_mut())
            }
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_open_stream(session: *const DynH2Sess) -> u32 {
    unsafe {
        (*session).open_stream().unwrap_or(0)
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn http2_send_data(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, stream_id: u32, end: bool, buf: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            sess.send_data(stream_id, end, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_headers(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, stream_id: u32, end: bool, headers: *const FfiHeaderPair, length: usize) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let mut head = Vec::with_capacity(length);

        for hv in slice::from_raw_parts(headers, length) {
            head.push((hv.nam.as_bytes(), hv.val.as_bytes()));
        }

        spawn_task_with(fut, async move{
            sess.send_headers(stream_id, end, &head).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_priority(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, stream_id: u32, dependency: u32, weight: u8) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            sess.send_priority(stream_id, dependency, weight).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_rst_stream(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, stream_id: u32, code: u32) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            sess.send_rst_stream(stream_id, code).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, settings: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::from(settings.as_bytes());

        spawn_task_with(fut, async move{
            sess.send_settings(settings).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings_default(fut: *const FfiFuture<c_void>, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::default();

        spawn_task_with(fut, async move{
            sess.send_settings(settings).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings_default_no_push(fut: *const FfiFuture<c_void>, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::DEFAULT_NO_PUSH;

        spawn_task_with(fut, async move{
            sess.send_settings(settings).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_send_settings_maximum(fut: *const FfiFuture<c_void>, session: *const DynH2Sess) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let settings = Http2Settings::MAXIMUM;

        spawn_task_with(fut, async move{
            sess.send_settings(settings).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_push_promise(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, associate_id: u32, promise_id: u32, headers: *const FfiHeaderPair, length: usize) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;
        let mut head = Vec::with_capacity(length);

        for hv in slice::from_raw_parts(headers, length) {
            head.push((hv.nam.as_bytes(), hv.val.as_bytes()));
        }

        spawn_task_with(fut, async move{
            sess.send_push_promise(associate_id, promise_id, &head).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_ping(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, ack: bool, buf: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            sess.send_ping(ack, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http2_send_goaway(fut: *const FfiFuture<c_void>, session: *const DynH2Sess, stream_id: u32, code: u32, buf: FfiSlice) {
    unsafe {
        let sess = &*session;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            sess.send_goaway(stream_id, code, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}


// Arc::increment_strong_count(conf);
// Arc::from_raw(conf)

#[unsafe(no_mangle)]
pub extern "C" fn http2_client_handler(session: *const DynH2Sess, stream_id: u32) -> *mut DynHttpRequest {
    unsafe {
        let session = {
            Arc::increment_strong_count(session);
            Arc::from_raw(session)
        };

        if let Ok(req) = Http2Request::new(stream_id, session) {
            let req = DynHttpRequest::Http2(req);
            heap_ptr(req)
        }
        else {
            ptr::null_mut()
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http2_server_handler(session: *const DynH2Sess, stream_id: u32) -> *mut DynHttpSocket {
    unsafe {
        let session = {
            Arc::increment_strong_count(session);
            Arc::from_raw(session)
        };
        
        if let Ok(req) = Http2Socket::new(stream_id, session) {
            let req = DynHttpSocket::Http2(req);
            heap_ptr(req)
        }
        else {
            ptr::null_mut()
        }
    }
}
pub mod client;
pub mod server;
pub mod const_enums;
pub mod websocket;
pub mod tls_server;
pub mod utils;
pub mod http2;
use std::{ffi::{CStr, c_void}, net::SocketAddr, os::fd::{FromRawFd, RawFd}, ptr};

use http::{http1::server::Http1Socket, http2::session::Http2Session, shared::{HttpClient, HttpMethod, HttpSocket, HttpType, HttpVersion}, websocket::socket::WebSocket};
use httprs_core::ffi::{futures::FfiFuture, slice::FfiSlice, own::spawn_task};
use tokio::{io::{AsyncWriteExt, BufReader, ReadHalf, WriteHalf}, net::TcpListener};

use crate::{DynStream, errno::{Errno, TYPE_ERR}, ffi::utils::heap_ptr, servers::{DynHttpSocket, detect_prot}, spawn_task_with};


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
pub struct FfiHeaderPair{
    pub nam: FfiSlice,
    pub val: FfiSlice,
}

impl FfiClient{
    pub fn from_owned(client: HttpClient) -> Self{
        let mut pairs = Vec::new();
        client.headers.into_iter().for_each(|(h,vs)|vs.into_iter().for_each(|v| pairs.push(FfiHeaderPair { nam: FfiSlice::from_string(h.clone()), val: FfiSlice::from_string(v) })));
        let pair_ptr = pairs.as_ptr();
        let pairs_len = pairs.len();
        let pairs_cap = pairs.capacity();
        std::mem::forget(pairs);

        Self {
            owned: true,
            valid: client.valid,

            head_complete: client.head_complete,
            body_complete: client.body_complete,

            path: FfiSlice::from_string(client.path),
            method: match client.method { HttpMethod::Unknown(_) => 0, HttpMethod::Get => 1, HttpMethod::Head => 2, HttpMethod::Post => 3, HttpMethod::Put => 4, HttpMethod::Delete => 5, HttpMethod::Connect => 6, HttpMethod::Options => 7, HttpMethod::Trace => 8 },
            version: match client.version { HttpVersion::Unknown(_) => 0, HttpVersion::Debug => 1, HttpVersion::Http09 => 2, HttpVersion::Http10 => 3, HttpVersion::Http11 => 4, HttpVersion::Http2 => 5, HttpVersion::Http3 => 6 },
            method_str: FfiSlice::from_string(client.method.to_string()),

            headers_len: pairs_len,
            headers_cap: pairs_cap,
            headers: pair_ptr,
            body: FfiSlice::from_vec(client.body),

            host: client.host.and_then(|h|Some(FfiSlice::from_string(h))).unwrap_or(FfiSlice::empty()),
            scheme: client.scheme.and_then(|s|Some(FfiSlice::from_string(s))).unwrap_or(FfiSlice::empty()),
        }
    }
    pub fn from(client: &HttpClient) -> Self{
        let mut pairs = Vec::new();
        client.headers.iter().for_each(|(h,vs)|vs.into_iter().for_each(|v| pairs.push(FfiHeaderPair { nam: FfiSlice::from_str(h), val: FfiSlice::from_str(v) })));
        let pair_ptr = pairs.as_ptr();
        let pairs_len = pairs.len();
        let pairs_cap = pairs.capacity();
        std::mem::forget(pairs);

        Self {
            owned: false,
            valid: client.valid,

            head_complete: client.head_complete,
            body_complete: client.body_complete,

            path: FfiSlice::from_str(&client.path),
            method: match client.method { HttpMethod::Unknown(_) => 0, HttpMethod::Get => 1, HttpMethod::Head => 2, HttpMethod::Post => 3, HttpMethod::Put => 4, HttpMethod::Delete => 5, HttpMethod::Connect => 6, HttpMethod::Options => 7, HttpMethod::Trace => 8 },
            version: match client.version { HttpVersion::Unknown(_) => 0, HttpVersion::Debug => 1, HttpVersion::Http09 => 2, HttpVersion::Http10 => 3, HttpVersion::Http11 => 4, HttpVersion::Http2 => 5, HttpVersion::Http3 => 6 },
            method_str: FfiSlice::from_string(client.method.to_string()),

            headers_len: pairs_len,
            headers_cap: pairs_cap,
            headers: pair_ptr,
            body: FfiSlice::from_buf(&client.body),

            host: client.host.as_ref().and_then(|h|Some(FfiSlice::from_str(h))).unwrap_or(FfiSlice::empty()),
            scheme: client.scheme.as_ref().and_then(|s|Some(FfiSlice::from_str(s))).unwrap_or(FfiSlice::empty()),
        }
    }

    pub fn free(self){
        self.method_str.free();
        let pairs = unsafe { Vec::from_raw_parts(self.headers as *mut FfiHeaderPair, self.headers_len, self.headers_cap) };
        
        if self.owned{
            self.path.free();
            self.body.free();
            self.host.free();
            self.scheme.free();


            for h in pairs {
                h.nam.free();
                h.val.free();
            }
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn tcp_server_new(fut: *mut FfiFuture<TcpListener>, string: *mut i8){
    unsafe {
        let addr = CStr::from_ptr(string).to_string_lossy().to_string();
        let fut = &*fut;

        spawn_task_with(fut, async move {
            let lis = TcpListener::bind(addr).await?;
            Ok(heap_ptr(lis))
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn tcp_server_from_fd(fd: RawFd) -> *mut TcpListener {
    unsafe {
        let tcp = std::net::TcpListener::from_raw_fd(fd);
        
        if let Ok(tcp) = TcpListener::from_std(tcp) {
            heap_ptr(tcp)
        } 
        else { 
            ptr::null_mut()
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn tcp_server_free(listener: *mut TcpListener) {
    unsafe {
        drop(Box::from_raw(listener))
    }
}

// #[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
pub extern "C" fn tcp_server_accept(fut: *mut FfiFuture<FfiBundle>, server: *mut TcpListener){
    unsafe {
        let server = &mut *server;
        let fut = &*fut;

        spawn_task_with(fut, async move {
            let (sock, addr) = server.accept().await?;
            let sock = sock.into();
            let sock = heap_ptr(sock);

            let addr = heap_ptr(addr);

            let ffi = FfiBundle {
                sock,
                addr,
            };

            Ok(heap_ptr(ffi))
        });
    }
}
// #[allow(improper_ctypes_definitions)]
// #[unsafe(no_mangle)]
/*pub extern "C" fn server_loop(fut: *mut FfiFuture, server: *mut FfiServer, cb: extern "C" fn(*mut FfiBundle)){
    unsafe {
        let mut ser = Box::from_raw(server);
        let fut = Box::from_raw(fut);

        spawn_task(async move {
            loop {
                match ser.boxed.accept().await{
                    Ok((addr, sock)) => cb(Box::into_raw(Box::new(FfiBundle { sock, addr }))),
                    Err(e) => {
                        fut.cancel_with_err(e.get_errno(), e.to_string().into());
                        break;
                    },
                }
            }

            let _ = Box::into_raw(ser);
            let _ = Box::into_raw(fut);
        });
    }
}*/

#[unsafe(no_mangle)]
pub extern "C" fn addr_is_ipv4(addr: *const SocketAddr) -> bool{
    unsafe{
        (*addr).is_ipv4()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn addr_is_ipv6(addr: *const SocketAddr) -> bool{
    unsafe{
        (*addr).is_ipv6()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn get_addr_str(addr: *const SocketAddr) -> FfiSlice{
    unsafe{
        FfiSlice::from_string((*addr).to_string())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_detect_prot(fut: *mut FfiFuture<u8>, stream: *mut DynStream){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;

        spawn_task(async move {
            if let DynStream::Tcp(tcp) = stream {
                fut.complete(heap_ptr(detect_prot(tcp).await))
            }
            else {
                fut.cancel_with_err(TYPE_ERR, "socket not tcp".into())
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_new(ffi: *mut DynStream, bufsize: usize) -> *mut DynHttpSocket{
    unsafe{
        let ffi = *Box::from_raw(ffi);
        let http = Http1Socket::new(ffi, bufsize);
        let dhtt = DynHttpSocket::Http1(http);
        heap_ptr(dhtt)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_get_type(http: *mut DynHttpSocket) -> u8{
    unsafe {
        match (*http).get_type() {
            HttpType::Http1 => 1,
            HttpType::Http2 => 2,
            HttpType::Http3 => 3,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_read_client(fut: *mut FfiFuture<c_void>, http: *mut DynHttpSocket){
    unsafe{
        let http = &mut *http;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            http.read_client().await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_read_until_complete(fut: *mut FfiFuture<c_void>, http: *mut DynHttpSocket){
    unsafe{
        let http = &mut *http;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            http.read_until_complete().await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_read_until_head_complete(fut: *mut FfiFuture<c_void>, http: *mut DynHttpSocket){
    unsafe{
        let http = &mut *http;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            http.read_until_head_complete().await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_set_header(http: *mut DynHttpSocket, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str_lossy();
        let value = pair.val.as_str_lossy();

        (*http).set_header(&name, value.into_owned());
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_add_header(http: *mut DynHttpSocket, pair: FfiHeaderPair){
    unsafe{
        let name = pair.nam.as_str_lossy();
        let value = pair.val.as_str_lossy();

        (*http).add_header(&name, value.into_owned());
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_del_header(http: *mut DynHttpSocket, name: FfiSlice){
    unsafe{
        let name = name.as_str_lossy();
        let _ = (*http).del_header(&name);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_write(fut: *mut FfiFuture<c_void>, http: *mut DynHttpSocket, buf: FfiSlice){
    unsafe{
        let http = &mut *http;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            http.write(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_close(fut: *mut FfiFuture<c_void>, http: *mut DynHttpSocket, buf: FfiSlice){
    unsafe{
        let http = &mut *http;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            http.close(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_flush(fut: *mut FfiFuture<c_void>, http: *mut DynHttpSocket){
    unsafe{
        let fut = &*fut;
        let http = &mut *http;
        spawn_task_with(fut, async move{
            http.flush().await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_get_fficlient(http: *mut DynHttpSocket) -> *mut FfiClient {
    unsafe{
        heap_ptr(FfiClient::from(&(*http).get_client()))
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_free_fficlient(client: *mut FfiClient) {
    unsafe { 
        let cl = Box::from_raw(client);
        cl.free();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_method(http: *mut DynHttpSocket) -> u8 {
    unsafe {
        match (*http).get_client().method { HttpMethod::Unknown(_) => 0, HttpMethod::Get => 1, HttpMethod::Head => 2, HttpMethod::Post => 3, HttpMethod::Put => 4, HttpMethod::Delete => 5, HttpMethod::Connect => 6, HttpMethod::Options => 7, HttpMethod::Trace => 8 }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_method_str(http: *mut DynHttpSocket) -> FfiSlice {
    unsafe {
        (&(*http).get_client().method.to_string()).into()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_path(http: *mut DynHttpSocket) -> FfiSlice {
    unsafe {
        (&(*http).get_client().path).into()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_version(http: *mut DynHttpSocket) -> u8 {
    unsafe {
        match (*http).get_client().version { HttpVersion::Unknown(_) => 0, HttpVersion::Debug => 1, HttpVersion::Http09 => 2, HttpVersion::Http10 => 3, HttpVersion::Http11 => 4, HttpVersion::Http2 => 5, HttpVersion::Http3 => 6 }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_client_has_header(http: *mut DynHttpSocket, name: FfiSlice) -> bool {
    unsafe{
        (*http).get_client().headers.contains_key(name.as_str_lossy().as_ref())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_client_has_header_count(http: *mut DynHttpSocket, name: FfiSlice) -> usize {
    unsafe{
        (*http).get_client().headers.get(name.as_str_lossy().as_ref()).and_then(|h|Some(h.len())).unwrap_or(0)
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_first_header(http: *mut DynHttpSocket, name: FfiSlice) -> FfiSlice {
    unsafe{
        (*http).get_client().headers.get(name.as_str_lossy().as_ref()).and_then(|h|Some(FfiSlice::from_string(h[0].clone()))).unwrap_or(FfiSlice::empty())
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_header(http: *mut DynHttpSocket, name: FfiSlice, index: usize) -> FfiSlice {
    unsafe{
        (*http).get_client().headers.get(name.as_str_lossy().as_ref()).and_then(
            |h|h.get(index)
            .and_then(|h|Some(FfiSlice::from_string(h.clone())))
        ).unwrap_or(FfiSlice::empty())
    }
}
// #[unsafe(no_mangle)]
/*pub extern "C" fn http_client_get_all_headers(http: *mut DynHttpSocket) -> FfiSlice {
    unsafe{
        let mut pairs = Vec::new();
        (*http).get_client().headers.iter().for_each(|(h,vs)|vs.into_iter().for_each(|v| pairs.push(FfiHeaderPair { nam: FfiSlice::from_str(h), val: FfiSlice::from_str(v) })));
        let pair_ptr = pairs.as_ptr();
        let pairs_len = pairs.len();
        let pairs_cap = pairs.capacity();
        std::mem::forget(pairs);
    }
}*/
#[unsafe(no_mangle)]
pub extern "C" fn http_client_get_body(http: *mut DynHttpSocket) -> FfiSlice {
    unsafe {
        (&(*http).get_client().body).into()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http_free(http: *mut DynHttpSocket){
    unsafe{
        drop(Box::from_raw(http));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_direct_write(fut: *mut FfiFuture<c_void>, http: *mut DynHttpSocket, buf: FfiSlice){
    unsafe{
        let http = &mut *http;
        let fut = &*fut;
        spawn_task(async move{
            match http {
                DynHttpSocket::Http1(one) => {
                    match one.netw.write_all(buf.as_bytes()).await {
                        Ok(_) => fut.complete(ptr::null_mut()),
                        Err(e) => fut.cancel_with_err(e.get_errno(), e.to_string().into()),
                    }
                }
                _ => fut.cancel_with_err(TYPE_ERR, "not http1".into()),
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_websocket(fut: *mut FfiFuture<WebSocket<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>>, http: *mut DynHttpSocket){
    unsafe{
        let http = *Box::from_raw(http);
        let fut = &*fut;
        
        match http {
            DynHttpSocket::Http1(one) => {
                spawn_task_with(fut, async move {
                    let ws = one.websocket().await?;
                    Ok(heap_ptr(ws))
                })
            }
            _ => fut.cancel_with_err(TYPE_ERR, "not http1".into()),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn http1_h2c(fut: *mut FfiFuture<Http2Session<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>>, http: *mut DynHttpSocket){
    unsafe{
        let http = *Box::from_raw(http);
        let fut = &*fut;
        
        match http {
            DynHttpSocket::Http1(one) => {
                spawn_task_with(fut, async move {
                    let h2 = one.h2c(None).await?;
                    Ok(heap_ptr(h2))
                })
            }
            _ => fut.cancel_with_err(TYPE_ERR, "not http1".into()),
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn http1_h2_prior_knowledge(fut: *mut FfiFuture<Http2Session<BufReader<ReadHalf<DynStream>>, WriteHalf<DynStream>>>, http: *mut DynHttpSocket){
    unsafe{
        let http = *Box::from_raw(http);
        let fut = &*fut;

        match http {
            DynHttpSocket::Http1(one) => {
                spawn_task_with(fut, async move {
                    let h2 = one.http2_prior_knowledge().await?;
                    Ok(heap_ptr(h2))
                })
            }
            _ => fut.cancel_with_err(TYPE_ERR, "not http1".into()),
        }
    }
}
use std::{ffi::CStr, ptr, sync::Arc};

use httprs_core::ffi::{futures::FfiFuture, slice::FfiSlice};
use rustls::{ServerConfig, pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject}, sign::CertifiedKey};
use tokio_rustls::TlsAcceptor;

use crate::{DynStream, PROVIDER, ffi::utils::heap_ptr, servers::TlsCertSelector, spawn_task_with};



#[unsafe(no_mangle)]
pub extern "C" fn tls_config_single_cert_pem(certs: FfiSlice, key: FfiSlice, alpns: *mut i8) -> *const ServerConfig {
    let prov = (*PROVIDER).clone();

    let certs = CertificateDer::pem_reader_iter(certs.as_bytes()).map(|c| c.and_then(|c| Ok(c.into_owned()))).collect::<Result<Vec<_>, _>>();
    let key = PrivateKeyDer::from_pem_reader(key.as_bytes());

    let alpns = unsafe { CStr::from_ptr(alpns).to_string_lossy().to_string() };
    let alpns = alpns.split(',').map(|s|s.as_bytes().to_vec()).collect();

    if 
        let Ok(certs) = certs && 
        let Ok(key) = key && 
        let Ok(build) = ServerConfig::builder_with_provider(prov).with_protocol_versions(rustls::DEFAULT_VERSIONS) && 
        let Ok(mut conf) = build.with_no_client_auth().with_single_cert(certs, key) 
    {
        conf.alpn_protocols = alpns;
        Arc::into_raw(Arc::new(conf))
    }
    else {
        ptr::null()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tls_config_sni_builder() -> *const TlsCertSelector {
    let builder = TlsCertSelector::new();

    Arc::into_raw(builder.to_arc())
}

#[unsafe(no_mangle)]
pub extern "C" fn tls_config_sni_builder_with_pem(def_certs: FfiSlice, def_key: FfiSlice) -> *const TlsCertSelector {
    let certs = if let Ok(certs) = CertificateDer::pem_reader_iter(def_certs.as_bytes()).map(|c| c.and_then(|c| Ok(c.into_owned()))).collect::<Result<Vec<_>, _>>() { certs } else { return ptr::null() };
    let key = if let Ok(key) = PrivateKeyDer::from_pem_reader(def_key.as_bytes()) { key } else { return ptr::null() };
    let cert = if let Ok(cert) = CertifiedKey::from_der(certs, key, &PROVIDER) { cert } else { return ptr::null() };

    let builder = TlsCertSelector::with_default(cert);

    Arc::into_raw(builder.to_arc())
}

#[unsafe(no_mangle)]
pub extern "C" fn tls_config_sni_add_pem(sni_build: *const TlsCertSelector, domain: *mut i8, certs: FfiSlice, key: FfiSlice) -> bool {
    unsafe {
        let domain = CStr::from_ptr(domain).to_string_lossy().to_string();

        let certs = if let Ok(certs) = CertificateDer::pem_reader_iter(certs.as_bytes()).map(|c| c.and_then(|c| Ok(c.into_owned()))).collect::<Result<Vec<_>, _>>() { certs } else { return false };
        let key = if let Ok(key) = PrivateKeyDer::from_pem_reader(key.as_bytes()) { key } else { return false };
        let cert = if let Ok(cert) = CertifiedKey::from_der(certs, key, &PROVIDER) { cert } else { return false };

        (*sni_build).add_cert(domain, cert);
        true
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tls_config_sni_builder_build(sni_build: *const TlsCertSelector, alpns: *mut i8) -> *const ServerConfig {
    unsafe{
        let sni = Arc::from_raw(sni_build);
        let prov = (*PROVIDER).clone();

        let alpns = CStr::from_ptr(alpns).to_string_lossy().to_string();
        let alpns = alpns.split(',').map(|s|s.as_bytes().to_vec()).collect();
        
        if let Ok(build) = ServerConfig::builder_with_provider(prov).with_protocol_versions(rustls::DEFAULT_VERSIONS) {
            let mut conf = build.with_no_client_auth().with_cert_resolver(sni);
            conf.alpn_protocols = alpns;
            
            let conf = Arc::new(conf);
            Arc::into_raw(conf)
        }
        else{
            ptr::null()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tls_config_free(conf: *const ServerConfig) {
    unsafe {
        drop(Arc::from_raw(conf));
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn tcp_upgrade_tls(fut: *mut FfiFuture<DynStream>, stream: *mut DynStream, conf: *const ServerConfig){
    unsafe {
        let stream = *Box::from_raw(stream);
        let fut = &*fut;
        let con = {
            Arc::increment_strong_count(conf);
            Arc::from_raw(conf)
        };
        let acc = TlsAcceptor::from(con);

        spawn_task_with(fut, async move {
            match stream {
                DynStream::Tcp(tcp) => {
                    let tls = acc.accept(tcp).await?;
                    let stream: DynStream = tls.into();
                    Ok(heap_ptr(stream))
                },
                DynStream::Duplex(dup) => {
                    let tls = acc.accept(dup).await?;
                    let stream: DynStream = tls.into();
                    Ok(heap_ptr(stream))
                },
                _ => {
                    Ok(heap_ptr(stream))
                },
            }
        })
    }
}
use core::ffi::c_void;
use std::{os::fd::{AsRawFd, FromRawFd, RawFd}, ptr};

use httprs_core::ffi::{futures::FfiFuture, slice::{AsFfiSlice, FfiSlice}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use crate::{DynStream, errno::TYPE_ERR, spawn_task_with};


pub fn heap_ptr<T>(thing: T) -> *mut T{
    Box::into_raw(Box::new(thing))
}
pub fn heap_void_ptr<T>(thing: T) -> *mut c_void {
    Box::into_raw(Box::new(thing)) as *mut c_void
}
pub fn heap_const_ptr<T>(thing: T) -> *const T{
    Box::into_raw(Box::new(thing))
}


#[repr(C)]
#[derive(Debug)]
pub struct FfiDuoStream {
    pub one: *mut DynStream, // idk
    pub two: *mut DynStream, // 
}


#[unsafe(no_mangle)]
pub extern "C" fn create_duplex(bufsize: usize) -> FfiDuoStream {
    let duo = tokio::io::duplex(bufsize);
    FfiDuoStream {
        one: heap_ptr(duo.0.into()),
        two: heap_ptr(duo.1.into()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_from_fd(fd: RawFd) -> *mut DynStream {
    unsafe {
        let tcp = std::net::TcpStream::from_raw_fd(fd);
        
        if let Ok(tcp) = TcpStream::from_std(tcp) {
            heap_ptr(tcp.into())
        } 
        else { 
            ptr::null_mut()
        }
    }
}
#[cfg(feature = "unix-sockets")]
#[unsafe(no_mangle)]
pub extern "C" fn unix_from_fd(fd: RawFd) -> *mut DynStream {
    unsafe {
        use tokio::net::UnixStream;

        let tcp = std::os::unix::net::UnixStream::from_raw_fd(fd);
        
        if let Ok(tcp) = UnixStream::from_std(tcp) {
            heap_ptr(tcp.into())
        } 
        else { 
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tcp_to_fd(stream: *mut DynStream) -> RawFd {
    unsafe {
        match &mut *stream {
            DynStream::Tcp(tcp) => {tcp.as_raw_fd()},
            _ => 0
        }
    }
}
#[cfg(feature = "unix-sockets")]
#[unsafe(no_mangle)]
pub extern "C" fn unix_to_fd(stream: *mut DynStream) -> RawFd {
    unsafe {
        match &mut *stream {
            DynStream::Unix(unix) => {unix.as_raw_fd()},
            _ => 0
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn tcp_peek(fut: *mut FfiFuture<usize>, ffi: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let ffi = &*ffi;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        if let DynStream::Tcp(tcp) = ffi {
            spawn_task_with(fut, async move {
                Ok(heap_ptr(tcp.peek(buf).await?))
            });
        }
        else{
            fut.cancel_with_err(TYPE_ERR, "socket not tcp".into())
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tls_get_alpn(stream: *mut DynStream) -> FfiSlice {
    unsafe {
        match &*stream {
            DynStream::TcpTls(tls) => {
                let (_, info) = tls.get_ref();
                info.alpn_protocol().map(|alpn| alpn.to_vec().as_ffi_slice()).unwrap_or(FfiSlice::empty())
            }
            DynStream::TlsDuplex(tls) => {
                let (_, info) = tls.get_ref();
                info.alpn_protocol().map(|alpn| alpn.to_vec().as_ffi_slice()).unwrap_or(FfiSlice::empty())
            }
            _ => FfiSlice::empty(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_get_type(stream: *mut DynStream) -> u8 {
    unsafe {
        match &*stream {
            DynStream::Duplex(_) => 0,
            DynStream::TlsDuplex(_) => 1,
            DynStream::Tcp(_) => 2,
            DynStream::TcpTls(_) => 3,
            #[cfg(feature = "unix-sockets")]
            DynStream::Unix(_) => 4,
            #[cfg(feature = "unix-sockets")]
            DynStream::UnixTls(_) => 5,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_read(fut: *mut FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        spawn_task_with(fut, async move {
            Ok(heap_ptr(stream.read(buf).await?))
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_read_exact(fut: *mut FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        spawn_task_with(fut, async move {
            Ok(heap_ptr(stream.read_exact(buf).await?))
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_write(fut: *mut FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;
        let buf = (*buf).as_bytes();

        spawn_task_with(fut, async move {
            Ok(heap_ptr(stream.write(buf).await?))
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn stream_write_all(fut: *mut FfiFuture<usize>, stream: *mut DynStream, buf: *mut FfiSlice){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;
        let buf = (*buf).as_bytes_mut();

        spawn_task_with(fut, async move {
            stream.write_all(buf).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_flush(fut: *mut FfiFuture<c_void>, stream: *mut DynStream){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;

        spawn_task_with(fut, async move {
            stream.flush().await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_shutdown(fut: *mut FfiFuture<c_void>, stream: *mut DynStream){
    unsafe {
        let stream = &mut *stream;
        let fut = &*fut;

        spawn_task_with(fut, async move {
            stream.shutdown().await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stream_free(stream: *mut DynStream){
    unsafe {
        drop(Box::from_raw(stream))
    }
}

use std::{ffi::c_void, ptr};

use httprs_core::ffi::{futures::FfiFuture, slice::FfiSlice};
use tokio::io::{BufReader, ReadHalf, WriteHalf};

use http::{shared::Stream, websocket::{core::WebSocketFrame, socket::WebSocket}};

use crate::{ffi::utils::heap_ptr, spawn_task_with};


pub type DynWebSocket = WebSocket<BufReader<ReadHalf<Box<dyn Stream>>>, WriteHalf<Box<dyn Stream>>>;


#[repr(C)]
pub struct FfiWsFrame{
    pub fin: bool,
    pub rsv: u8,
    pub opcode: u8,
    pub masked: bool,
    pub payload: FfiSlice,
}
impl FfiWsFrame{
    pub fn from_owned(mut frame: WebSocketFrame) -> Self{
        frame.unmask_in_place();
        Self { 
            fin: frame.fin, 
            rsv: frame.rsv, 
            opcode: frame.opcode.into(), 
            masked: frame.masked,
            payload: frame.source[frame.payload].to_vec().into(),
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn websocket_read_frame(fut: *mut FfiFuture<FfiWsFrame>, ws: *mut DynWebSocket){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            Ok(heap_ptr(FfiWsFrame::from_owned(ws.read_frame().await?)))
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_free_frame(frame: *mut FfiWsFrame){
    unsafe{
        drop(Box::from_raw(frame))
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_flush(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.flush().await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_free(ws: *mut DynWebSocket){
    unsafe{
        drop(Box::from_raw(ws))
    }
}



#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_continuation(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_continuation(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_continuation_masked(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_continuation_masked(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_continuation_frag(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_continuation_frag(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_continuation_masked_frag(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_continuation_masked_frag(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_text(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_text(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_text_masked(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_text_masked(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_text_frag(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_text_frag(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_text_masked_frag(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_text_masked_frag(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_binary(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_binary(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_binary_masked(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_binary_masked(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_binary_frag(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_binary_frag(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_binary_masked_frag(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_binary_masked_frag(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_close(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, code: u16, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_close(code, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_close_masked(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, code: u16, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_close_masked(&mask, code, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_ping(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_ping(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_ping_masked(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_ping_masked(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_pong(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;

        spawn_task_with(fut, async move{
            ws.send_pong(buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn websocket_send_pong_masked(fut: *mut FfiFuture<c_void>, ws: *mut DynWebSocket, buf: FfiSlice){
    unsafe{
        let ws = &mut *ws;
        let fut = &*fut;
        let mut mask = [0u8; 4];
        rand::fill(&mut mask);

        spawn_task_with(fut, async move{
            ws.send_pong_masked(&mask, buf.as_bytes()).await?;
            Ok(ptr::null_mut())
        });
    }
}
use tokio::runtime::Runtime;
use crate::ffi::{futures::{self, FfiFuture}, slice::FfiSlice};
use std::{ffi::{CStr, c_void}, ptr, sync::{OnceLock, atomic::Ordering}};


// tokio

pub static RT: OnceLock<Runtime> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn init_rt() -> bool{
    if let Ok(rt) = tokio::runtime::Builder::new_multi_thread().enable_all().build(){
        RT.set(rt).is_ok()
    }
    else{
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn has_init() -> bool{
    RT.get().is_some()
}

pub fn spawn_task<F: Future<Output = ()> + Send + 'static>(future: F) {
    RT.get().unwrap().spawn(future);
}




// futures

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_new(cb: Option<extern "C" fn(*mut c_void, *mut c_void)>, userdata: *mut c_void) -> *mut FfiFuture{
    Box::into_raw(FfiFuture::new_boxed(cb, userdata))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_state(fut: *const FfiFuture) -> u8{
    unsafe { (*fut).state.load(Ordering::Acquire) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_result(fut: *const FfiFuture) -> *mut c_void{
    unsafe {
        if (*fut).state.load(Ordering::Acquire) == futures::READY{
            *(*fut).result.get()
        }
        else {
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_take_result(fut: *const FfiFuture) -> *mut c_void{
    unsafe {
        if (*fut).state.load(Ordering::Acquire) == futures::READY{
            let rptr = (*fut).result.get();
            let result = *rptr;
            *rptr = ptr::null_mut();
            result
        }
        else {
            ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_cancel(fut: *const FfiFuture) {
    unsafe { (*fut).cancel() }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_cancel_with_err(fut: *const FfiFuture, code: i32, msg: FfiSlice) {
    unsafe { (*fut).cancel_with_err(code, msg) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_complete(fut: *const FfiFuture, result: *mut c_void) {
    unsafe { (*fut).complete(result) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_free(fut: *mut FfiFuture) {
    unsafe { drop(Box::from_raw(fut)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_await(fut: *mut FfiFuture) {
    unsafe {
        let rfut = &mut *fut;
        RT.get().unwrap().block_on(async move {
            let _ = rfut.await;
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_get_errno(fut: *mut FfiFuture) -> i32 {
    unsafe {
        *(*fut).errno.get()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_get_errmsg(fut: *mut FfiFuture) -> *const FfiSlice {
    unsafe {
        (*fut).errmsg.get()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_reset(fut: *mut FfiFuture) {
    unsafe {
        (*fut) = FfiFuture::default()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_get_userdata(fut: *const FfiFuture) -> *mut c_void{
    unsafe {
        *(*fut).userdata.get()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn ffi_future_set_userdata(fut: *const FfiFuture, userdata: *mut c_void) {
    unsafe {
        *(*fut).userdata.get() = userdata;
    }
}


// async_ffi(crate) FfiFuture

#[unsafe(no_mangle)]
pub extern "C" fn rt_spawn_async_ffi_future(fut: async_ffi::FfiFuture<()>) {
    RT.get().unwrap().spawn(fut);
}

// slice

#[unsafe(no_mangle)]
pub extern "C" fn free_slice(slice: FfiSlice) {
    slice.free();
}

// test

#[unsafe(no_mangle)]
pub extern "C" fn add_i64(x: i64, y: i64) -> i64 {
    x + y
}

#[unsafe(no_mangle)]
pub extern "C" fn panic_test(message: *const i8) -> ! {
    if message.is_null() {
        panic!("")
    }
    else {
        unsafe {
            let cstr = CStr::from_ptr(message);
            let cstr = cstr.to_string_lossy();
            panic!("{}", cstr);
        }
    }
}
