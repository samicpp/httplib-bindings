#include<stdio.h>
#include<iostream>
#include "ffi.h"
#include "rsffi/base.h"
#include "wrapper.hpp"
#include <coroutine>


int add(int x, int y){
    return x + y;
}

int add_test(){
    // printf("herro\n");
    // double test = 1.0 + 2.0;
    // printf("result is %f\n", test);

    long long i64 = add_i64(1, 2);
    if (i64 != 3) return 1;

    return 0;
}

double add_f64(double x, double y){
    return x + y;
}

int server_test(){
    printf("hello???\n");
    char addr[] = "0.0.0.0:2048";

    printf("starting tokio rt\n");
    if (!has_init() && !init_rt()) return 1;

    FfiServer server;
    {
        bool done = false;
        printf("making future\n");
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        printf("passing future\n");
        tcp_server_new(fut, addr);
        printf("waiting for future\n");
        while (!done) ;
        // ffi_future_await(fut);
        printf("taking result\n");
        server = ffi_future_take_result(fut);
    }
    printf("server ptr = %p\n", server);
    // if (!server) return 2;

    FfiBundle* bundle;
    {
        bool done = false;
        printf("making future\n");
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        printf("passing future\n");
        tcp_server_accept(fut, server);
        printf("waiting for future\n");
        while (!done) ;
        // ffi_future_await(fut);
        printf("taking result\n");
        bundle = static_cast<FfiBundle*>(ffi_future_take_result(fut));
    }
    printf("bundle ptr = %p\n", bundle);
    // if (!bundle) return 3;

    auto ipaddr = get_addr_str(bundle->addr);
    printf("ip addr = %.*s\n", (int)ipaddr.len, ipaddr.ptr);

    FfiSocket http = http1_new(bundle->stream, 8 * 1024);

    {
        bool done = false;
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        http_read_until_head_complete(fut, http);
        while (!done) ;
        // ffi_future_await(fut);
    }

    http_set_header(http, HeaderPair { sliceFromCstr("Content-Type"), sliceFromCstr("text/plain") });
    http_set_header(http, HeaderPair { sliceFromCstr("Connection"), sliceFromCstr("close") });

    {
        bool done = false;
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        http_close(fut, http, sliceFromCstr("Hello, world!\n"));
        while (!done) ;
        // ffi_future_await(fut);
    }

    {
        bool done = false;
        auto fut = ffi_future_new([](void* userdata, void* result){
            bool* done = static_cast<bool*>(userdata);
            *done = true;
        }, &done);
        http_flush(fut, http);
        while (!done) ;
        // ffi_future_await(fut);
    }

    http_free(http);

    // printf("done, press enter\n");
    // std::cin.get();

    return 0;
}