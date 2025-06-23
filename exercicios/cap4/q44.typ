#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.1": *
#show: codly-init.with()

== 4.4. A precedence graph is a directed, acyclic graph. Nodes represent tasks, and arcs indicate the order in which tasks are to be accomplished. In particular, a task can execute as soon as all its predecessors have been completed. Assume that the tasks are processes and that each process has the following outline:

#codly(languages: codly-languages)
```
process T {
  wait for predecessors, if any;
  body of the task;
  signal successors, if any;
}
```

#set enum(numbering: "a)")

=== a) Using semaphores, show how to synchronize five processes whose permissible execution order is specified by the following precedence graph:

#set align(center)

#image("imgs/graphQ44.png")

#set align(left)

Minimize the number of semaphores that you use, and do not impose constraints not specified in the graph. For example, T2 and T3 can execute concurrently after T1 completes.

=== Resposta

#codly(languages: codly-languages)
```
sem S1 = 0,
    S2 = 0,
    S3 = 0,
    S4 = 0,
process T1 {
  body of the task;
  V(S1);
}

process T2 {
  P(S1);
  body of the task;
  V(S2);
}

process T3 {
  P(S1);
  body of the task;
  V(S3);
}

process T4 {
  P(S2);
  body of the task;
  V(S4);
}

process T5 {
  P(S3);
  P(S4);
  body of the task;
}
```

=== b) Describe how to synchronize processes, given an arbitrary precedence graph. In particular, devise a general method for assigning semaphores to edges or processes and for using them. Do not try to use the absolute minimum number of semaphores since determining that is an NP-hard problem for an arbitrary precedence graph!

=== Resposta

#codly(languages: codly-languages)
```
process T[i = 1 to M] {
  for (j = 1 to N) {
    if (m[i][j]) {
  for (j = 1 to N) {
    if (m[i][j]) {
      P(S[j]);
      P(S[j]);
    }
  }
  body of the task;
  V(S[i]);
}
```
