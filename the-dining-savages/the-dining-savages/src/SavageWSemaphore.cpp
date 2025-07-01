#include "SavageWSemaphore.hpp"
#include "Semaphore.hpp"
TDS::SavageWSemaphore::SavageWSemaphore(Semaphore eat, Semaphore waitCook)
    : eat(eat), waitCook(waitCook){};