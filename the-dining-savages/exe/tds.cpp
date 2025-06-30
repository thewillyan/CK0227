#include <CLI11/CLI11.hpp>
#include <chrono>
#include <stop_token>
#include <thread>

#include "Camp.hpp"

#define TDS_VERSION "0.1.0"

void dummy_savage_func(std::stop_token st) {
  std::cout << "Worker started working...\n";
  std::chrono::milliseconds sleep_time{500};
  while (!st.stop_requested()) {
    std::this_thread::sleep_for(sleep_time);
  }
  std::cout << "Worker exiting cleanly\n";
}

int main(int argc, char **argv) {
  CLI::App app("The Dining Savages problem");

  app.set_version_flag("-v,--version", std::string(TDS_VERSION));

  int n_savages{0};
  app.add_option("-s,--savages", n_savages, "Number of Savage threads")
      ->required(true);

  CLI11_PARSE(app, argc, argv);

  Camp camp{n_savages, dummy_savage_func};
  for (int i = 0; i < 10; ++i) {
    camp.task();
    std::this_thread::sleep_for(std::chrono::milliseconds{100});
  }
  camp.stop_all();

  return 0;
}
