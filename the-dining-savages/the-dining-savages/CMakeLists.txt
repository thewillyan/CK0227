add_library(TheDiningSavagesLib
    "${CMAKE_CURRENT_SOURCE_DIR}/include/Semaphore.hpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/Semaphore.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/include/Pot.hpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/Pot.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/include/SavageWSemaphore.hpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/SavageWSemaphore.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/include/CookWSemaphore.hpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/CookWSemaphore.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/include/Monitor.hpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/src/Monitor.cpp"

)

target_include_directories(TheDiningSavagesLib PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/include")

add_subdirectory("tests")
