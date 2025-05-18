# Exercícios Capítulo 2

Resolução dos exercícios 2.1, 2.8, 2.13, 2.15, 2.18, 2.24 e 2.33 do livro *Foundations of Multithreaded, Parallel, and Distributed Programming*.

## 8. A queue is often represented using a linked list. Assume that two variables, `head` and `tail`, point to the first and last elements of the list. Each element contains a data field and a link to the next element. Assume that a null link is represented by the constant `null`.

### (a) Write routines to (1) search the list for the first element (if any) that contains data value `d`, (2) insert a new element at the end of the list, and (3) delete the element from the front of the list. The search and delete routines should return `null` if they cannot succeed.

### (b) Now assume that several processes access the linked list. Identify the read and write sets of each routine, as defined in (2.1). Which  combinations of routines can be executed in parallel? Which combinations of routines must execute one at a time (i.e., atomically)?

### (c) Add synchronization code to the three routines to enforce the synchronization you identified in your answer to (b). Make your atomic actions as small as possible, and do not delay a routine unnecessarily. Use the await statement to program the synchronization code.

## 13. Consider the following three statements bellow. Assume that `x` is initially `2` and that `y` is initially `5`. For each of the following, what er the possible final values of `x` and `y`? Explain your answers.

- `S1: x = x + y`;
- `S2: y = x - y`;
- `S3: x = x - y`;

### (a) `S1; S2; S3`
### (b) `co <S_1;> // <S2;> // <S3;> oc`
### (c) `co <await (x > y) S1; S2;> // <S3;> oc`

## 18. Consider the program bellow. For what initial values of `x` does the program terminate, assuming scheduling is weakly fair? What are the corresponding final values? Explain your answer.

```
co <await (x > 0) x = x - 1;>
// <await (x < 0) x = x + 2;>
// <await (x == 0) x = x - 1;>
oc
```

## 33. Consider the following program:

```
int x = 0, c = true;
co <await x == 0>; c = false;
// while (c) <x = x - 1>;
oc
```

### (a) Will the program terminate if scheduling is weakly fair? Explain.
### (b) Will the program terminate if scheduling is strongly fair? Explain.
### (c) Add the code bellow as a third arm of the `co` statement. Repeat parts (a) and (b) for this three-process program.

```
while (c) {if (x < 0) <x = 10>;}
```