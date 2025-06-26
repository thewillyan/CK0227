#include "Monitor.hpp"
#include <atomic>
#include <doctest/doctest.h>
#include <thread>
#include <vector>

using namespace TDS;

TEST_SUITE("Monitor Tests") {
  TEST_CASE("Monitor Lock Acquisition") {
    Monitor monitor;

    SUBCASE("Basic lock acquisition") {
      auto lock = monitor.enter();
      CHECK(lock.owns_lock());
    }

    SUBCASE("Non-blocking try_enter") {
      auto lock1 = monitor.enter();
      CHECK(lock1.owns_lock());

      auto lock2 = monitor.try_enter();
      CHECK_FALSE(lock2.owns_lock());
    }
  }

  TEST_CASE("Condition Variable - Predicated Wait") {
    Monitor monitor;
    Monitor::ConditionVariable cv;
    bool flag = false;
    std::atomic<bool> started{false};

    std::thread worker([&] {
      auto lock = monitor.enter();
      started = true;
      cv.wait(lock, [&] { return flag; });
    });

    // Wait for worker to start and acquire lock
    while (!started)
      std::this_thread::yield();

    SUBCASE("Single notification") {
      {
        auto lock = monitor.enter();
        flag = true;
      }
      cv.notify_one();
    }

    SUBCASE("All notification") {
      {
        auto lock = monitor.enter();
        flag = true;
      }
      cv.notify_all();
    }

    worker.join();
    CHECK(flag);
  }

  TEST_CASE("Condition Variable - Basic Wait") {
    Monitor monitor;
    Monitor::ConditionVariable cv;
    bool flag = false;
    std::atomic<bool> started{false};

    std::thread worker([&] {
      auto lock = monitor.enter();
      started = true;
      while (!flag) {
        cv.wait(lock);
      }
    });

    // Wait for worker to start and acquire lock
    while (!started)
      std::this_thread::yield();

    SUBCASE("Notification after state change") {
      {
        auto lock = monitor.enter();
        flag = true;
      }
      cv.notify_one();
    }

    worker.join();
    CHECK(flag);
  }

  TEST_CASE("Multiple Threads Coordination") {
    Monitor monitor;
    Monitor::ConditionVariable cv;
    int counter = 0;
    constexpr int THREAD_COUNT = 10;
    std::vector<std::thread> threads;
    std::atomic<int> ready{0};

    for (int i = 0; i < THREAD_COUNT; ++i) {
      threads.emplace_back([&] {
        auto lock = monitor.enter();
        ready++;
        cv.wait(lock, [&] { return counter == 1; });
        counter++;
      });
    }

    // Wait for all threads to be ready
    while (ready < THREAD_COUNT)
      std::this_thread::yield();

    {
      auto lock = monitor.enter();
      counter = 1;
    }
    cv.notify_all();

    for (auto &t : threads)
      t.join();

    CHECK(counter == THREAD_COUNT + 1);
  }

  TEST_CASE("Concurrent Access Protection") {
    Monitor monitor;
    int counter = 0;
    constexpr int ITERATIONS = 10000;
    constexpr int THREAD_COUNT = 4;
    std::vector<std::thread> threads;

    for (int i = 0; i < THREAD_COUNT; ++i) {
      threads.emplace_back([&] {
        for (int j = 0; j < ITERATIONS; ++j) {
          auto lock = monitor.enter();
          counter++;
        }
      });
    }

    for (auto &t : threads)
      t.join();

    CHECK(counter == THREAD_COUNT * ITERATIONS);
  }
}
