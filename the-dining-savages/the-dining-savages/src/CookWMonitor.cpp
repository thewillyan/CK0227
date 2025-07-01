#include "CookWMonitor.hpp"
#include <stop_token>

TDS::CookWMonitor::CookWMonitor(std::shared_ptr<Monitor> lock,
                                std::shared_ptr<ConditionVariable> pot_empty_cv,
                                std::shared_ptr<ConditionVariable> pot_full_cv,
                                std::shared_ptr<std::atomic_uint> servings,
                                uint max_servings)
    : monitor{lock}, pot_empty{pot_empty_cv}, pot_full{pot_full_cv},
      curr_servings{servings}, M{max_servings} {}

void TDS::CookWMonitor::operator()(std::stop_token st) const {
  while (!st.stop_requested()) {
    auto lock = monitor->enter();
    if (!pot_empty->wait(lock, st, [this] { return *curr_servings == 0; })) {
      break; // Stop requested
    }
    *curr_servings = M;
    pot_full->notify_all();
  }
}
