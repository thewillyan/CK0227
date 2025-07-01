#pragma once

#include "Pot.hpp"
#include "Semaphore.hpp"
namespace TDS {
class CookWSemaphore {
public:
  Pot *pot;
  Semaphore *semWake;
  Semaphore *semEat;
  Semaphore *semEmpty;

public:
  CookWSemaphore(Semaphore *semWake, Semaphore *semEat, Semaphore *semEmpty,
                 Pot *pot);

public:
  void refill();
  void run();
};

} // namespace TDS