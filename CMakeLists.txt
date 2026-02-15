cmake_minimum_required(VERSION 4.1)
project(rrad_xla C)

set(CMAKE_C_STANDARD 11)

add_executable(rrad_xla
        expertnal_xla/pjrt_c_api.h.c)
