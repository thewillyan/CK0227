#include "Camp.hpp"

TDS::Camp::Camp(int num_savages,
                std::function<void(std::stop_token)> savage_func,
                std::function<void(std::stop_token)> cook_func) {
  // Create savage threads first (exception-safe)
  savage_threads_.reserve(num_savages);
  for (int i = 0; i < num_savages; ++i) {
    savage_threads_.emplace_back(savage_func);
  }

  // Start cook thread after object is fully constructed
  cook_thread_ = std::jthread(cook_func);
}

TDS::Camp::~Camp() {
  if (!stop_requested_) {
    stop_all();
  }
}

void TDS::Camp::stop_all() {
  if (stop_requested_.exchange(true))
    return;

  // Stop cook first
  cook_thread_.request_stop();
  cv_.notify_one();

  // Stop all savage threads
  for (auto &t : savage_threads_) {
    t.request_stop();
  }

  // Wait for cook thread to finish
  if (cook_thread_.joinable()) {
    cook_thread_.join();
  }

  // Wait for savage threads to finish
  for (auto &t : savage_threads_) {
    if (t.joinable()) {
      t.join();
    }
  }
}
