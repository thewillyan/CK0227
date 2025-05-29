= Exercícios Capítulo 3

Resolução dos exercícios 3.1 do livro *Foundations of Multithreaded, Parallel, and Distributed Programming*.

Alunos:

- Andre Willyan de Sousa Vital 537550
- Vitor de Oliveira Veras 520400

== 3.1. Following is Dekker's algorithm, the first solution to the critical section problem for two processes:

```
bool enter1 = false, enter2 = false;
int turn = 1;
process P1 {
  while (true) {
    enter1 = true;
    while (enter2)
      if (turn == 2) {
        enter1 = false;
        while (turn == 2) skip;
        enter1 = true;
      }
    critical section;
    enter1 = false; turn = 2;
    noncritical section;
  }
}
process P2 {
  while (true) {
    enter2 = true;
    while (enter1)
      if (turn == 1){
         enter2 = false;
         while (turn == 1) skip;
         enter2 = true;
      }
    critical section;
    enter2 = false; turn = 1;
    noncritical section;
  }
}
```

Explain clearly how the program ensures mutual exclusion, avoids deadlock, avoids unnecessary delay, and ensures eventual entry. For the eventual entry property, how many times can one process that wants to enter its critical section be bypassed by the other before the first gets in? Explain.

=== Resposta

A propriedade de excusão mútua (_mutual exclusion_) é assegurada de forma com que `P1` inicia juntamente com `P2`, contudo é possível perceber que independente da velocidade dos dois processos `P2` vai fazer com que `enter2 = false` e em seguida vai entrar em um loop

```
while (turn == 1) skip;
```

Enquanto isso `P1` poderá seguir para sua seção crítica, onde fará com que `enter1 = false` e que `turn = 2`, nesse momento ele vai sair da sua seção crítica, enquanto que isso `P2` terá saído do loop e feito com que `enter2 = true`, `P1` terá recomeçado e assim como ocorreu com `P2` inicialmente ele entrará em um loop. `P2` então entrará em sua seção crítica, após isso será realizado `enter2 = false` e `turn = 1` de forma com que será reiniciado novamente.

A propriedade de evitar _deadlocks_ (_avoid deadlocks_) é presetvada de forma na utilização da variável `enter` que faz com que o processo que não está na vez fique preso em um `while` até que um deles troque o valor e dê a vez para o outro processo.

A propriedade de espera desnecessária (_unecessary delay_) é preservada dado que assim que um processo termina de sair da seção crítica o outro processo que estava dentro do `while` já tem permissão de entrar dado a atualização das variáveis `enter` e `turn`.

A propriedade de entrada eventual (_eventual entry_) é assegurada quando é presente no algoritmo a troca dos valores das variáveis que fazem com que o valor em espera possa finalmente entrar na seção crítica, sem essa linha o processo ficaria em espera indefinida.

No caso de um processo em espera de entrar na sua vez ser ultrapassado por outro nessa solução nenhum, dado que quando o processo da vez altera o valor de `enter` o outro processo já pode sair do loop atualizando o outro valor de `enter` para que o antigo processo quando reiniciar entre em loop.
