cmake_minimum_required(VERSION 3.10.0)
project(httpcpp VERSION 0.1.0 LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 20)
# set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
# set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")


file(GLOB_RECURSE SOURCES "src/*.c" "src/*.cpp" "src/*.cc" "src/*.h" "src/*.hpp")
add_library(httpcpp ${SOURCES})

include(CTest)

# set(HTTPRS_PATH "${CMAKE_SOURCE_DIR}/../target/debug")

# if(WIN32) 
#     set(HTTPRS_LIB "httprs.lib")
# # elseif(APPLE) 
# #    set(HTTPRS_LIB "libhttprs.dylib")
# elseif(UNIX) 
#     set(HTTPRS_LIB "libhttprs.so")
# endif()

# # add_library(httprs STATIC IMPORTED)
# # set_target_properties(httprs PROPERTIES
# #     IMPORTED_LOCATION "${HTTPRS_PATH}/${HTTPRS_LIB}"
# # )
# target_link_libraries(httpcpp PRIVATE "${HTTPRS_PATH}/${HTTPRS_LIB}")
