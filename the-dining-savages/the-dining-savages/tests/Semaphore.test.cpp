#include "Semaphore.hpp"
#include "doctest/doctest.h"
#include <atomic>
#include <chrono>
#include <thread>

TEST_SUITE("Semaphore Tests") {
  TEST_CASE("proberen should waits until counter is incremented by verhogen") {
    TDS::Semaphore sem(0);
    std::atomic<bool> flag(false);

    std::thread t([&]() {
      sem.proberen();
      flag.store(true);
    });

    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    CHECK(flag.load() == false);

    sem.verhogen();
    t.join();
    CHECK(flag.load() == true);
  }
}