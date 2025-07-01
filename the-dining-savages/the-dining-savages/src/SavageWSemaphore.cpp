#include "SavageWSemaphore.hpp"
#include "Semaphore.hpp"
TDS::SavageWSemaphore::SavageWSemaphore(Semaphore *mutex, Semaphore *semEat,
                                        Semaphore *semWakeCook,
                                        Semaphore *semEmpty, Pot *pot)
    : mutex(mutex), semEat(semEat), semWakeCook(semWakeCook),
      semEmpty(semEmpty), pot(pot){};

void TDS::SavageWSemaphore::eat() { this->pot->available--; }
void TDS::SavageWSemaphore::run() {
  while (true) {
    this->mutex->proberen();
    if (this->pot->available == 0) {
      this->semEmpty->proberen();
      this->semWakeCook->verhogen();
    }
    this->semEat->proberen();
    this->pot->available--;
    this->mutex->verhogen();
  }
}