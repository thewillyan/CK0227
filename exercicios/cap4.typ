#align(center)[
  #set text(1.2em)
  *Exercícios Capítulo 4*

  André Willyan & Vitor Veras  
]

= Questão 8

Give all possible values of variable `x` in the following program.
Explain how you got your answer.

```
int x = 0; sem s1 = 1, s2 = 0;
co P(s2); P(s1); x = x*2; V(s1);
// P(s1); x = x*x; V(s1);
// P(s1); x = x+3; V(s2); V(s1);
oc
```

= Questão 21
Cosider the following solution to the readers/writers problem.
It employs the same counters and semaphores as in @livro-texto Figure 4.13, but uses them differently.

```
int nr = 0, nw = 0; # numbers of readers and writers
sem e = 1;
sem r = 0, w = 0;
int dr = 0, dw = 0;

process Reader[i = 1 to M] {
  while (true) {
    P(e);
      if (nw == 0) { nr = nr + 1; V(r); }
      else dr = dr+1;
    V(e);
    P(r); # wait for permission to read
    read the database;
    P(e);
      nr = nr-1;
      if (nr == 0 and dw > 0)
        { dw = dw-1; nw = nw+1; V(w); }
    V(e);
  }
}

process Writer[j = 1 to N] {
  while (true) {
    P(e);
      if (nr == 0 and nw == 0) { nw = nw+1; V(w); }
      eles dw = dw+1;
    V(e);
    P(w);
    write the database;
    P(e);
      nw = nw - 1;
      if (dw > 0) { dw = dw-1; nw = nw+1; V(w); }
      else
        while (dr > 0) { dr = dr-1; nr = nr+1; V(r); }
      V(e);
  }
}
``` 

== a) Carefully explain how this solution works.

What is the role of each semaphore?
Show that the solution ensures that writers have exclusive access to the
database and a writer excludes readers.

== b) What kind of preference the above solution have?

Readers preference? Writers preference? Alternating Preference?

== c) Compare this solution to the one in @livro-texto Figure 4.13.

How many `P` and `V` operations are executed by each process
in each solution in the best case?
In the worst case?
Which program do you find easier to understand, and why?

= Questão 27

_Cigarette Smokers problem_ @patil1971 @parnas1975.
Suppose there are three smoker processes and one agent process.
Each smoker continuously make as cigarette and smokes it.
Making a cigarette requires three ingredients: tobacco, paper and a match.
One smoker process has tobacco, the second paper, and the third matches.
Each has an infinite supply of these ingredients.
The agent places a random two ingredients on the table.
The smoker who has the third ingredient picks up the other two, makes a cigarette, then, smokes it.
The agent waits for the smoker to finish. The cycle then repeats.

Develop a solution to this problem using semaphores for synchronzation.
You may also need to use other variables.

#bibliography("bib.yml")
