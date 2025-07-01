#include <CLI11/CLI11.hpp>
#include <chrono>
#include <climits>
#include <memory>
#include <stop_token>
#include <string>

#include "Camp.hpp"
#include "CookWMonitor.hpp"
#include "CookWSemaphore.hpp"
#include "Monitor.hpp"
#include "Pot.hpp"
#include "SavageWMonitor.hpp"
#include "SavageWSemaphore.hpp"
#include "Semaphore.hpp"

#define TDS_VERSION "0.1.0"

template <typename Duration>
void monitor_pot(std::shared_ptr<std::atomic_uint> servings, uint max_servings,
                 const Duration &time_listening) {
  using namespace std::chrono;

  // ANSI color codes
  const char *GREEN = "\033[1;32m";
  const char *YELLOW = "\033[1;33m";
  const char *CYAN = "\033[1;36m";
  const char *RESET = "\033[0m";
  const char *BOLD = "\033[1m";

  auto start = steady_clock::now();
  uint initial_servings = servings->load();
  uint current_count = initial_servings;
  uint refilled_count = 0; // Start with 0 refills during monitoring

  // Precompute formatting parameters
  const int servings_width = std::to_string(max_servings).size();

  // Formatting helper function (returns colored string and visual length)
  auto format_display = [=](uint count,
                            uint refills) -> std::pair<std::string, size_t> {
    // Plain version for length calculation
    std::ostringstream plain;
    plain << "Servings: " << std::setw(servings_width) << count << "/"
          << max_servings << " | Refilled (approx.): " << refills;
    size_t visual_length = plain.str().length();

    // Colored version for display
    std::ostringstream colored;
    colored << BOLD << "Servings: " << RESET << YELLOW
            << std::setw(servings_width) << count << RESET << BOLD << GREEN
            << "/" << YELLOW << max_servings << RESET << " | " << BOLD
            << "Refilled (approx.): " << RESET << YELLOW << refills << RESET;

    return {colored.str(), visual_length};
  };

  // Initial display
  auto [display_str, visual_length] =
      format_display(current_count, refilled_count);
  std::cout << display_str << std::flush;
  size_t last_visual_length = visual_length;

  while (steady_clock::now() - start < time_listening) {
    uint new_count = servings->load();

    if (new_count != current_count) {
      // Update refill counter if servings increased
      if (current_count < new_count) {
        refilled_count++;
      }

      // Create new display string
      auto [new_display, new_visual_length] =
          format_display(new_count, refilled_count);

      // Update display
      std::cout << '\r' << new_display;

      // Pad with spaces if new output is shorter
      if (new_visual_length < last_visual_length) {
        std::cout << std::string(last_visual_length - new_visual_length, ' ');
      }
      std::cout << std::flush;

      // Update tracking variables
      last_visual_length = new_visual_length;
      current_count = new_count;
    }

    // Reduce CPU usage
    std::this_thread::sleep_for(10ms);
  }

  // Calculate total meals approximation
  uint total_served_approx =
      initial_servings + (refilled_count * max_servings) - current_count;

  // Clear final line and print summary
  std::cout << "\r" << std::string(last_visual_length, ' ') << "\r";
  std::cout << CYAN << "\nMonitoring Summary:" << RESET << "\n";
  std::cout << "  • Approximate refills: " << YELLOW << refilled_count << RESET
            << "\n";
  std::cout << "  • Total meals served: " << YELLOW << "≥ "
            << total_served_approx << RESET << "\n";
  std::cout << "    (Initial: " << initial_servings
            << ", Refills: " << refilled_count << " × " << max_servings
            << ", Final: " << current_count << ")" << RESET << std::endl;
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

  std::string method;
  app.add_option("--method-sync", method,
                 "Method to use for synchronization (monitor/semaphore)")
      ->required(true)
      ->check(CLI::IsMember({"monitor", "semaphore"}));

  CLI11_PARSE(app, argc, argv);

  if (method == "monitor") {
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
    return 1;
  } else if (method == "semaphore") {
    TDS::Semaphore mutex(1);
    TDS::Semaphore eat(0);
    TDS::Semaphore refill(0);
    TDS::Semaphore empty(1);

    TDS::Pot pot(max_serving);
    TDS::CookWSemaphore cook(&refill, &eat, &empty, &pot);

    {
      std::jthread cook_thread([&]() { cook.run(); });
      std::vector<std::jthread> savage_threads;
      for (int i = 0; i < n_savages; ++i) {
        savage_threads.emplace_back([&]() {
          TDS::SavageWSemaphore savage(i, &mutex, &eat, &refill, &empty, &pot);
          savage.run();
        });
      }
    }
  }
  return 0;
}
