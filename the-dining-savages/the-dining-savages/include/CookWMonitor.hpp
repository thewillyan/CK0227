#pragma once

#include "Monitor.hpp"
#include <memory>
#include <stop_token>

namespace TDS {
class CookWMonitor {
private:
  std::shared_ptr<Monitor> monitor;
  std::shared_ptr<ConditionVariable> pot_empty;
  std::shared_ptr<ConditionVariable> pot_full;
  std::shared_ptr<std::atomic_uint> curr_servings;
  const uint M;

public:
  CookWMonitor(std::shared_ptr<Monitor> lock,
               std::shared_ptr<ConditionVariable> pot_empty_cv,
               std::shared_ptr<ConditionVariable> pot_full_cv,
               std::shared_ptr<std::atomic_uint> servings, uint max_servings);

  void operator()(std::stop_token st) const;
};
} // namespace TDS
