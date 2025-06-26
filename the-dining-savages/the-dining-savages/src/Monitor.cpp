#include "Monitor.hpp"

// ConditionVariable Implementation
void TDS::Monitor::ConditionVariable::wait(std::unique_lock<std::mutex>& lock) {
  cv_.wait(lock);
}

void TDS::Monitor::ConditionVariable::notify_one() noexcept {
  cv_.notify_one();
}

void TDS::Monitor::ConditionVariable::notify_all() noexcept {
  cv_.notify_all();
}

// Monitor Implementation
std::unique_lock<std::mutex> TDS::Monitor::enter() {
  return std::unique_lock<std::mutex>(monitor_mutex_);
}

std::unique_lock<std::mutex> TDS::Monitor::try_enter() {
  return std::unique_lock<std::mutex>(monitor_mutex_, std::try_to_lock);
}
