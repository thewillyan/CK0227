#pragma once

#include "Pot.hpp"
#include "Semaphore.hpp"
namespace TDS {
class SavageWSemaphore {
public:
  long int id;
  Semaphore *mutex;
  Semaphore *semEat;
  Semaphore *semWakeCook;
  Semaphore *semEmpty;
  Pot *pot;

public:
  SavageWSemaphore(long int id, Semaphore *mutex, Semaphore *semEat,
                   Semaphore *semWakeCook, Semaphore *semEmpty, Pot *pot);

public:
  void eat();
  void run();
};
} // namespace TDS