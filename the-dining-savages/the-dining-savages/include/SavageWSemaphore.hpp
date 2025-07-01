#pragma once

#include "Semaphore.hpp"
namespace TDS {
class SavageWSemaphore {
public:
  Semaphore eat;
  Semaphore waitCook;

public:
  SavageWSemaphore(Semaphore eat, Semaphore waitCook);
};
} // namespace TDS