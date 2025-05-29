= Exercícios Capítulo 4

Resolução dos exercícios 4.4 do livro *Foundations of Multithreaded, Parallel, and Distributed Programming*.

Alunos:

- Andre Willyan de Sousa Vital 537550
- Vitor de Oliveira Veras 520400

== 4.4. A precedence graph is a directed, acyclic graph. Nodes represent tasks, and arcs indicate the order in which tasks are to be accomplished. In particular, a task can execute as soon as all its predecessors have been completed. Assume that the tasks are processes and that each process has the following outline:

```
process T {
  wait for predecessors, if any;
  body of the task;
  signal successors, if any;
}
```

#set enum(numbering: "a)")

+ Using semaphores, show how to synchronize five processes whose permissible execution order is specified by the following precedence graph:

#set align(center)

#image("image.png")

#set align(left)

Minimize the number of semaphores that you use, and do not impose constraints not specified in the graph. For example, T2 and T3 can execute concurrently after T1 completes.

