#include "SavageWMonitor.hpp"
#include <stop_token>

using namespace TDS;

SavageWMonitor::SavageWMonitor(std::shared_ptr<Monitor> lock,
                               std::shared_ptr<ConditionVariable> pot_empty_cv,
                               std::shared_ptr<ConditionVariable> pot_full_cv,
                               std::shared_ptr<std::atomic_uint> servings)
    : monitor{lock}, pot_empty{pot_empty_cv}, pot_full{pot_full_cv},
      curr_servings{servings} {}

void SavageWMonitor::operator()(std::stop_token st) const {
  while (!st.stop_requested()) {
    auto lock = monitor->enter();
    if (*curr_servings == 0) {
      pot_empty->notify_one();
      if (!pot_full->wait(lock, st, [this] { return *curr_servings > 0; })) {
        break; // Stop requested
      }
    }
    // Extra check after potential wakeup
    if (st.stop_requested())
      break;
    (*curr_servings)--;
  }
}
