cmake_minimum_required(VERSION 3.28)
project(TheDiningSavages)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_COMPILER clang++)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include_directories(lib)

enable_testing()
add_library(Doctest INTERFACE)
target_include_directories(Doctest INTERFACE "${CMAKE_SOURCE_DIR}/lib/doctest")

add_library(CLI11 INTERFACE)
target_include_directories(CLI11 INTERFACE "${CMAKE_SOURCE_DIR}/lib/CLI11")

add_subdirectory(the-dining-savages)
add_subdirectory(exe)
