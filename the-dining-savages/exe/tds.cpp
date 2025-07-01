#include <CLI11/CLI11.hpp>
#include <chrono>
#include <climits>
#include <memory>
#include <stop_token>

#include "Camp.hpp"
#include "CookWMonitor.hpp"
#include "Monitor.hpp"
#include "SavageWMonitor.hpp"

#define TDS_VERSION "0.1.0"

template <typename Duration>
void monitor_pot(std::shared_ptr<std::atomic_uint> servings, uint max_servings,
                 const Duration &time_listening) {
  using namespace std::chrono;
  auto start = steady_clock::now();
  uint current_count = servings->load();
  uint refilled_count = current_count == max_servings ? 1 : 0;

  // Precompute formatting parameters
  const int servings_width = std::to_string(max_servings).size();
  const std::string header =
      "Servings:   / " + std::to_string(max_servings) + " | Refilled: ";
  const int refill_padding = 5; // Allow up to 5 digits for refill count

  // Format initial display
  std::cout << header << std::setw(refill_padding) << refilled_count
            << std::flush;
  size_t last_length = header.size() + refill_padding;

  while (steady_clock::now() - start < time_listening) {
    uint new_count = servings->load();

    if (new_count != current_count) {
      // Update refill counter if servings increased
      if (current_count < new_count) {
        refilled_count++;
      }

      // Build new display string
      std::ostringstream display;
      display << "\rServings: " << std::setw(servings_width) << new_count << "/"
              << max_servings << " | Refilled: " << std::setw(refill_padding)
              << refilled_count;

      // Update display with padding to cover previous output
      std::string display_str = display.str();
      std::cout << display_str << std::flush;

      // Pad with spaces if new output is shorter
      if (display_str.size() < last_length) {
        std::cout << std::string(last_length - display_str.size(), ' ');
      }

      // Update tracking variables
      last_length = std::max(display_str.size(), last_length);
      current_count = new_count;
    }

    // Reduce CPU usage
    std::this_thread::sleep_for(10ms);
  }

  // Clear final line before exit
  std::cout << "\r" << std::string(last_length, ' ') << "\r" << std::flush;
}

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
  monitor_pot(servings, max_serving, std::chrono::seconds{10});
  camp.stop_all();

  return 0;
}
