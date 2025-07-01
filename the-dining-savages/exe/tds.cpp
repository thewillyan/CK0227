#include <chrono>
#include <climits>
#include <memory>
#include <stop_token>
#include <thread>

#include <CLI11/CLI11.hpp>

#include "Camp.hpp"
#include "CookWMonitor.hpp"
#include "Monitor.hpp"
#include "SavageWMonitor.hpp"

#define TDS_VERSION "0.1.0"

int main(int argc, char **argv) {
  CLI::App app("The Dining Savages problem");

  app.set_version_flag("-v,--version", std::string(TDS_VERSION));

  int n_savages{0};
  app.add_option("-s,--savages", n_savages, "Number of Savage threads")
      ->required(true);

  int max_serving{0};
  app.add_option("-m,--max-serving", max_serving, "Max capacity of the pot")
      ->required(true)
      ->check(CLI::Range(1, INT_MAX));

  CLI11_PARSE(app, argc, argv);

  // state variables
  std::shared_ptr<TDS::Monitor> monitor = std::make_shared<TDS::Monitor>();
  std::shared_ptr<TDS::ConditionVariable> empty_cv =
      std::make_shared<TDS::ConditionVariable>();
  std::shared_ptr<TDS::ConditionVariable> full_cv =
      std::make_shared<TDS::ConditionVariable>();
  std::shared_ptr<std::atomic_uint> servings =
      std::make_shared<std::atomic_uint>(0);

  // thread functions
  auto savage_func = [&](std::stop_token st) {
    TDS::SavageWMonitor savage{monitor, empty_cv, full_cv, servings};
    savage(st);
  };
  TDS::CookWMonitor cook{monitor, empty_cv, full_cv, servings,
                         static_cast<uint>(max_serving)};

  // camp thread manager
  TDS::Camp camp{n_savages, savage_func, cook};
  std::this_thread::sleep_for(std::chrono::milliseconds{500});
  camp.stop_all();

  return 0;
}
