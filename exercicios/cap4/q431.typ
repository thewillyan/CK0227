#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.1": *
#show: codly-init.with()

== 4.31. The One-Lane Bridge. Cars coming from the north and the south arrive at a one lane bridge. Cars heading in the same direction can cross the bridge at the same time, but cars heading in opposite directions cannot.

=== a) Develop a solution to this problem. First specify a global invariant, then develop a solution using semaphores for synchronization. Do not worry about fairness.

=== Resposta

A invariante que será usada é a seguinte: $("north_on_bridge" == 0) or ("south_on_bridge" == 0)$

#codly(languages: codly-languages)
```
sem mutex = 1;
int north_on_bridge = 0;
int south_on_bridge = 0;

process North[i = 1 to M] {
  while (true) {
    P(mutex);
    if (south_on_bridge == 0) {
      north_on_bridge++;
      V(mutex);
      atravessa a ponte;
      P(mutex);
      north_on_bridge--;
      V(mutex);
    } else {
      V(mutex);
    }
  }
}

process South[i = 1 to M] {
  while (true) {
    P(mutex);
    if (north_on_bridge == 0) {
      south_on_bridge++;
      V(mutex);
      atravessa a ponte;
      P(mutex);
      south_on_bridge--;
      V(mutex);
    } else {
      V(mutex);
    }
  }
}
```

=== b) Modify your answer to (b) to ensure that any car that is waiting to cross the bridge eventually gets to do so. You may want to solve the problem differently. (Hint: Use the technique of passing the baton.)

=== Resposta

#codly(languages: codly-languages)
```
sem mutex = 1;
sem north_queue = 0;
sem south_queue = 0;

int north_on_bridge = 0;
int south_on_bridge = 0;

int north_waiting = 0;
int south_waiting = 0;

process North[i = 1 to M] {
  while (true) {
    P(mutex);
    if (south_on_bridge > 0 || south_waiting > 0) {
      north_waiting++;
      V(mutex);
      P(north_queue);
      P(mutex);
      north_waiting--;
    }
    north_on_bridge++;
    V(mutex);
    atravessa a ponte;
    P(mutex);
    north_on_bridge--;
    if (north_on_bridge == 0) {
      if (south_waiting > 0) {
        V(south_queue);
      } else if (north_waiting > 0) {
        V(north_queue);
      }
    }
    V(mutex);
  }
}

process South[i = 1 to M] {
  while (true) {
    P(mutex);
    if (north_on_bridge > 0 || north_waiting > 0) {
      south_waiting++;
      V(mutex);
      P(south_queue);
      P(mutex);
      south_waiting--;
    }
    south_on_bridge++;
    V(mutex);
    atravessa a ponte;
    P(mutex);
    south_on_bridge--;
    if (south_on_bridge == 0) {
      if (north_waiting > 0) {
        V(north_queue);
      } else if (south_waiting > 0) {
        V(south_queue);
      }
    }
    V(mutex);
  }
}
```
