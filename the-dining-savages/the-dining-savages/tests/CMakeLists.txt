set(TestsToRun
    "${CMAKE_CURRENT_SOURCE_DIR}/Semaphore.test.cpp"
    "${CMAKE_CURRENT_SOURCE_DIR}/Monitor.test.cpp"
)

# Define output directory for test binaries
set(TEST_BIN_DIR "${CMAKE_CURRENT_BINARY_DIR}/bin")

# Create the bin directory if it doesn't exist
file(MAKE_DIRECTORY ${TEST_BIN_DIR})

foreach(test ${TestsToRun})
    get_filename_component(TName ${test} NAME_WE)
    add_executable(${TName} ${test} "${CMAKE_CURRENT_SOURCE_DIR}/main.cpp")
    
    # Set binary output directory
    set_target_properties(${TName} PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY ${TEST_BIN_DIR}
    )
    
    target_compile_features(${TName} PRIVATE cxx_std_17)
    target_link_libraries(${TName} PRIVATE Doctest TheDiningSavagesLib)
    
    # Update test command to use binary from bin directory
    add_test(NAME ${TName} COMMAND ${TEST_BIN_DIR}/${TName})
endforeach()
