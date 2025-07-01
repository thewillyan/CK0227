#include "CookWSemaphore.hpp"

TDS::CookWSemaphore::CookWSemaphore(Semaphore *semWake, Semaphore *semEat,
                                    Semaphore *semEmpty, Pot *pot)
    : semWake(semWake), semEat(semEat), semEmpty(semEmpty), pot(pot) {}

void TDS::CookWSemaphore::refill() { this->pot->available = this->pot->m; }
void TDS::CookWSemaphore::run() {
  while (true) {
    this->semWake->proberen();
    this->refill();
    for (int i = 0; i < this->pot->m; ++i)
      this->semEat->verhogen();
    this->semEmpty->verhogen();
  }
}