#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.1": *
#show: codly-init.with()


== 3.9. Consider the following variation on the n-process tie-breaker algorithm [Block and Woo 1990):

#codly(languages: codly-languages)
```
int in = 0, last[1:n]; # shared variables 
process CS[i = 1 to n] { 
  int stage; 
  while (true) { 
    <in = in + 1;>; stage = 1; last[stage] = i; 
    <await (last [stage] != i or in <= stage);>
    while (last [stage] != i) { # go to next stage 
      stage = stage + 1; last [stage] = i; 
      <await (last[stage] != i or in <= stage);> 
    } 
    critical section; 
    <in = in - 1; > 
    noncritical section; 
  } 
}
```

=== a) Explain clearly how this program ensures mutual exclusion, avoids deadlock, and ensures eventual entry

=== Resposta




=== b) Compare the performance of this algorithm to that of the tie-breaker algorithm in Figure 3.7. In particular, which is faster if only one process is trying to enter the critical section? How much faster? Which is faster if all n processes are trying to enter the critical section? How much faster?

=== Resposta

=== c) Convert the coarse-grained solution above to a fine-grained solution in which the only atomic actions are reading and writing variables. Do not assume increment and decrement are atomic. (Hint: Change in to an array.)

=== Resposta