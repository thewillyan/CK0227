#pragma once

#include <atomic>
#include <condition_variable>
#include <functional>
#include <mutex>
#include <stop_token>
#include <thread>
#include <vector>

namespace TDS {
class Camp {
public:
  Camp(int num_savages, std::function<void(std::stop_token)> savage_func,
       std::function<void(std::stop_token)> cook_func);
  ~Camp();

  // Delete copy semantics
  Camp(const Camp &) = delete;
  Camp &operator=(const Camp &) = delete;

  // Control methods
  void stop_all();

private:
  // Worker threads
  std::vector<std::jthread> savage_threads_;

  // Manager thread
  std::jthread cook_thread_;

  // Synchronization
  mutable std::mutex mutex_;
  std::condition_variable_any cv_;
  std::atomic<bool> stop_requested_{false};
};

} // namespace TDS
