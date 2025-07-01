#include "CookWSemaphore.hpp"
#include "Pot.hpp"
#include "SavageWSemaphore.hpp"
#include "Semaphore.hpp"
#include <cstdio>
#include <pthread.h>

FILE *log_file;

void *savage_process(void *arg) {
  TDS::SavageWSemaphore *savage = static_cast<TDS::SavageWSemaphore *>(arg);
  pthread_t tid = pthread_self();
  fprintf(log_file, "Savage %ld thread started\n", tid);
  printf("Savage %ld thread started\n", tid);
  while (true) {
    savage->mutex->proberen();
    if (savage->pot->available == 0) {
      savage->semEmpty->proberen();
      fprintf(log_file, "Savage %ld: Pot is empty, waking cook\n", tid);
      printf("Savage %ld: Pot is empty, waking cook\n", tid);
      savage->semWakeCook->verhogen();
    }
    savage->semEat->proberen();
    savage->eat();
    savage->mutex->verhogen();
    fprintf(log_file, "Savage %ld: Ate\n", tid);
    printf("Savage %ld: Ate\n", tid);
  }
  return NULL;
}

void *cook_process(void *arg) {
  TDS::CookWSemaphore *cook = static_cast<TDS::CookWSemaphore *>(arg);
  fprintf(log_file, "Cook thread started\n");
  printf("Cook thread started\n");
  while (true) {
    cook->semWake->proberen();
    fprintf(log_file, "Cook thread: Refilling pot\n");
    printf("Cook thread: Refilling pot\n");
    cook->refill();
    for (int i = 0; i < cook->pot->m; ++i)
      cook->semEat->verhogen();
    cook->semEmpty->verhogen();
  }
  return NULL;
}

int main() {
  log_file = fopen("log.txt", "w");
  if (!log_file) {
    perror("Failed to open log file");
    return 1;
  }

  pthread_t savages_threads[3];
  pthread_t cook_thread;

  TDS::Semaphore mutex(1);
  TDS::Semaphore eat(0);
  TDS::Semaphore refill(0);
  TDS::Semaphore empty(1);

  TDS::Pot pot(5);

  TDS::CookWSemaphore cook(&refill, &eat, &empty, &pot);
  TDS::SavageWSemaphore s0(&mutex, &eat, &refill, &empty, &pot);
  TDS::SavageWSemaphore s1(&mutex, &eat, &refill, &empty, &pot);
  TDS::SavageWSemaphore s2(&mutex, &eat, &refill, &empty, &pot);

  pthread_create(&cook_thread, NULL, cook_process, &cook);
  pthread_create(&savages_threads[0], NULL, savage_process, &s0);
  pthread_create(&savages_threads[1], NULL, savage_process, &s1);
  pthread_create(&savages_threads[2], NULL, savage_process, &s2);

  pthread_join(cook_thread, NULL);
  for (int i = 0; i < 3; i++) {
    pthread_join(savages_threads[i], NULL);
  }

  fclose(log_file);
  return 0;
}