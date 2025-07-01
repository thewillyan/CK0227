#include "CookWSemaphore.hpp"

TDS::CookWSemaphore::CookWSemaphore(Semaphore wake, Semaphore eat, Pot pot)
    : wake(wake), eat(eat), pot(pot) {}

void TDS::CookWSemaphore::refill() { this->pot.available = this->pot.m; }