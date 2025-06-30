add_executable(TDS
  "${CMAKE_CURRENT_SOURCE_DIR}/tds.cpp"
)

# Define executable output directory
set(EXE_BIN_DIR "${CMAKE_CURRENT_BINARY_DIR}/bin")

set_target_properties(TDS PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${EXE_BIN_DIR}
)
target_compile_features(TDS PRIVATE cxx_std_20)
target_link_libraries(TDS PRIVATE CLI11 TheDiningSavagesLib)

# Add "run" subcommand
add_custom_target(run
    COMMAND $<TARGET_FILE:TDS> $ENV{ARGS}
    DEPENDS TDS
    COMMENT "Running: $<TARGET_FILE_NAME:TDS> $ENV{ARGS}"
    WORKING_DIRECTORY ${EXE_BIN_DIR}
)
