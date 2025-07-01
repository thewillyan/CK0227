#pragma once

#include "Pot.hpp"
#include "Semaphore.hpp"
namespace TDS {
class CookWSemaphore {
public:
  Pot pot;
  Semaphore wake;
  Semaphore eat;

public:
  CookWSemaphore(Semaphore wake, Semaphore eat, Pot pot);

public:
  void refill();
};

} // namespace TDS