#include "Semaphore.hpp"
#include <pthread.h>

TDS::Semaphore::Semaphore(int initialValueCounter) {
  this->counter_ = initialValueCounter;
  pthread_mutex_init(&this->mutex_, nullptr);
  pthread_cond_init(&this->cond_, nullptr);
}

TDS::Semaphore::~Semaphore() {
  pthread_mutex_destroy(&this->mutex_);
  pthread_cond_destroy(&this->cond_);
}

void TDS::Semaphore::proberen() {
  pthread_mutex_lock(&this->mutex_);
  while (this->counter_ == 0) {
    pthread_cond_wait(&this->cond_, &this->mutex_);
  }
  this->counter_--;
  pthread_mutex_unlock(&this->mutex_);
}

void TDS::Semaphore::verhogen() {
  pthread_mutex_lock(&this->mutex_);
  this->counter_++;
  pthread_cond_signal(&this->cond_);
  pthread_mutex_unlock(&this->mutex_);
}