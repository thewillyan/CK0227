#pragma once

#include <pthread.h>

namespace TDS {
class Semaphore {
public:
  Semaphore(int initialValueCounter);
  ~Semaphore();

public:
  /**
   * Delay a process until an event has occurred waiting until the value of the
   * semaphore is positive then decrements the value
   */
  void proberen();

  /**
   * Sinals the occurence of an event and increments the value of the semaphore
   */
  void verhogen();

private:
  int counter_;
  pthread_mutex_t mutex_;
  pthread_cond_t cond_;
};
} // namespace TDS