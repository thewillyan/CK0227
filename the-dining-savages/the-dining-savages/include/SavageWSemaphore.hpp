#pragma once

#include "Pot.hpp"
#include "Semaphore.hpp"
namespace TDS {
class SavageWSemaphore {
public:
  Semaphore *mutex;
  Semaphore *semEat;
  Semaphore *semWakeCook;
  Semaphore *semEmpty;
  Pot *pot;

public:
  SavageWSemaphore(Semaphore *mutex, Semaphore *semEat, Semaphore *semWakeCook,
                   Semaphore *semEmpty, Pot *pot);

public:
  void eat();
  void run();
};
} // namespace TDS