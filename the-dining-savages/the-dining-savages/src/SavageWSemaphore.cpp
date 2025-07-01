#include "SavageWSemaphore.hpp"
#include "Semaphore.hpp"
#include <cstdio>
TDS::SavageWSemaphore::SavageWSemaphore(long int id, Semaphore *mutex, Semaphore *semEat,
                                        Semaphore *semWakeCook,
                                        Semaphore *semEmpty, Pot *pot)
    : id(id), mutex(mutex), semEat(semEat), semWakeCook(semWakeCook),
      semEmpty(semEmpty), pot(pot){};

void TDS::SavageWSemaphore::eat() { this->pot->available--; }
void TDS::SavageWSemaphore::run() {
  while (true) {
    printf("Savage %ld: Trying to eat\n", this->id);
    this->mutex->proberen();
    if (this->pot->available == 0) {
      this->semEmpty->proberen();
      printf("Savage %ld: Pot is empty, waking cook\n", this->id);
      this->semWakeCook->verhogen();
    }
    this->semEat->proberen();
    this->pot->available--;
    this->mutex->verhogen();
    printf("Savage %ld: Ate\n", this->id);
  }
}