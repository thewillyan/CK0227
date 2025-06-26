#pragma once

#include <condition_variable>
#include <mutex>

namespace TDS {
class Monitor {
public:
  Monitor() = default;
  ~Monitor() = default;

  // Delete copy and move operations
  Monitor(const Monitor&) = delete;
  Monitor& operator=(const Monitor&) = delete;
  Monitor(Monitor&&) = delete;
  Monitor& operator=(Monitor&&) = delete;

  class ConditionVariable {
  public:
    ConditionVariable() = default;
    ~ConditionVariable() = default;

    // Delete copy and move operations
    ConditionVariable(const ConditionVariable&) = delete;
    ConditionVariable& operator=(const ConditionVariable&) = delete;
    ConditionVariable(ConditionVariable&&) = delete;
    ConditionVariable& operator=(ConditionVariable&&) = delete;

    // Templated wait with predicate (spurious wakeup protection)
    template <typename Predicate>
    void wait(std::unique_lock<std::mutex>& lock, Predicate pred) {
      cv_.wait(lock, pred);
    }

    // Basic wait (non-predicated)
    void wait(std::unique_lock<std::mutex>& lock);

    void notify_one() noexcept;
    void notify_all() noexcept;

  private:
    std::condition_variable cv_;
  };

  // Blocking lock acquisition
  [[nodiscard]] std::unique_lock<std::mutex> enter();
  
  // Non-blocking lock attempt
  [[nodiscard]] std::unique_lock<std::mutex> try_enter();

private:
  std::mutex monitor_mutex_;
};
} // namespace TDS
