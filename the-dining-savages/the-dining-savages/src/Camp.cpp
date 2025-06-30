#include "Camp.hpp"
#include <iostream>

Camp::Camp(int num_savages, std::function<void(std::stop_token)> savage_func) {
  // Create savage threads first (exception-safe)
  savage_threads_.reserve(num_savages);
  for (int i = 0; i < num_savages; ++i) {
    savage_threads_.emplace_back(savage_func);
  }

  // Start cook thread after object is fully constructed
  cook_thread_ = std::jthread(&Camp::cook_thread_func, this);
}

Camp::~Camp() {
  if (!stop_requested_) {
    stop_all();
  }
}

void Camp::cook_thread_func(std::stop_token st) {
  std::cout << "Cook started\n";

  while (!st.stop_requested()) {
    // Wait for notification or stop request
    std::unique_lock lock(mutex_);
    cv_.wait(lock, [&] { return cook_woken_ || st.stop_requested(); });

    if (st.stop_requested())
      break;

    // Reset wake flag
    cook_woken_ = false;
    lock.unlock();

    // Perform cook task
    task_counter_++;
    std::cout << "Cook performed task #" << task_counter_.load() << "\n";
  }

  std::cout << "Cook exiting\n";
}

void Camp::stop_all() {
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

void Camp::task() {
  // Early exit if stop requested
  if (stop_requested_) {
    return;
  }

  {
    std::lock_guard lock(mutex_);
    cook_woken_ = true;
  }
  cv_.notify_one();
}

int Camp::get_task_count() const { return task_counter_.load(); }
