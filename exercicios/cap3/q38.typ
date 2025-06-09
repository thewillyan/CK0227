#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.1": *
#show: codly-init.with()

== 3.8. Suppose your machine has the following atomic instruction:

#codly(languages: codly-languages)
```
flip(lock)
  < lock = (lock + 1) % 2; # flip the lock
    return (lock); >       # return the new value
```
Someone suggest the following solution to the critical section problem for two processes:
```
int lock = 0;
process CS[i = 1 to 2] {
  while (true) {
    while(flip(lock) != 1)
      while(lock != 0) skip;
    critical section;
    lock = 0;
    noncritical section;
  }
}
```
=== a) Explain why this solution will not work—in other words, give an execution order that results in both processes being in their critical sections at the same time.

=== Resposta

Suponha que ambos os processos `P1` e `P2` iniciem ao mesmo tempo, ambos entram pelo primeiro `while(true)` e ambos vão na linha 4 executar a função `flip(lock)`. A partir desse ponto é possível imaginar um cenário onde `P1` executa em um momento mais rápido e temos o valor de `lock = 1`, assim `P2` ao executar a linha 4 entrará dentro do loop `while(lock != 0) skip;` até que `P1` entre na seção crítica e troque o valor `lock` de 1 para 0 e faça com que `P2` saia do loop podendo entrar livremente na seção crítica, assim teremos os 2 processos na seção crítica ao mesmo tempo.

=== b) Suppose that the first line in the body of flip is changed to do addition modulo 3 rather than modulo 2. Will the solution now work for two processes? Explain your answer.

=== Resposta

Com a mudança de `% 2` para `% 3` a solução falha em garantir a propriedade de _Eventual Entry_, ou seja, que um processo não fique indefinidamente sem entrar em sua seção crítica. Considere o caso:

#table(
  columns: 4,
  table.header[*Passo*][*P1*][*P2*][*lock*],
  [1], [-], [-], [0],
  [2], [`flip`], [-], [1],
  [3], [Entrada na seção crítica], [-], [1],
  [4], [-], [`flip`], [2],
  [5], [-], [Entrada em loop `while (lock != 0)`], [2],
  [6], [`lock = 0` e saída da seção crítica], [-], [0],
  [7], [`flip`], [-], [1],
  [8], [Entrada na seção crítica], [`flip`], [2],
  [9], [-], [Entrada em loop `while (lock != 0)`], [2],
)

Assim temos que o processo `P2` pode ter de ficar indefinidamente sem entrar na seção crítica.
