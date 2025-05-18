---
header-includes: |
  \usepackage{tikz}
  \usetikzlibrary{trees}
  \usepackage[a4paper, margin=3cm]{geometry}
output: pdf_document
---

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

O programa é executado de forma sequencial portanto temos:

1. `S1: x = x + y` $\rightarrow$ `x = 2 + 5` $\rightarrow$ `x = 7` (`x = 7`, `y = 5`);
2. `S2: y = x - y` $\rightarrow$ `y = 7 - 5` $\rightarrow$ `y = 2` (`x = 7`, `y = 2`);
3. `S3: x = x - y` $\rightarrow$ `x = 7 - 2` $\rightarrow$ `x = 5` (`x = 5`, `y = 2`);

Portanto o valor final, ilustrado entre parênteses, é `x = 5` e `y = 2`.

### (b) `co <S1;> // <S2;> // <S3;> oc`

Neste caso `S1`, `S2` e `S3` irão ser executados como ações atômicas de forma paralela.
Assim, esse programa possui 3 processos com 1 ação atômica cada resultanto emum número finito de histórias igual a $3! = 6$.
Podemos representar as permultações em uma estrutura de árvore de estados com variáveis `x` e `y` da seguinte forma.

```{=latex}
\begin{tikzpicture}[
    level 1/.style = {sibling distance = 5cm, level distance = 2cm},
    level 2/.style = {sibling distance = 2.5cm, level distance = 2cm},
    level 3/.style = {sibling distance = 1.25cm, level distance = 2cm},
    edge from parent/.style = {->, draw, thick},
    every node/.style = {font=\small, fill=white, inner sep=2pt, rectangle}
]

\node {x = 2, y = 5}
    child {node {x = 7, y = 5}
        child {node {x = 7, y = 2}
            child {node {x = 5, y = 2}
                edge from parent node[left] {S3}
            }
            edge from parent node[left] {S2}
        }
        child {node {x = 2, y = 5}
            child {node {x = 2, y = -3}
                edge from parent node[right] {S2}
            }
            edge from parent node[right] {S3}
        }
        edge from parent node[left=3pt] {S1}
    }
    child {node {x = 2, y = -3}
        child {node {x = -1, y = -3}
            child {node {x = 2, y = -3}
                edge from parent node[left] {S3}
            }
            edge from parent node[left] {S1}
        }
        child {node {x = 5, y = -3}
            child {node {x = 2, y = -3}
                edge from parent node[right] {S1}
            }
            edge from parent node[right] {S3}
        }
        edge from parent node[left=3pt] {S2}
    }
    child {node {x = -3, y = 5}
        child {node {x = 2, y = 5}
            child {node {x = 2, y = -3}
                edge from parent node[left] {S2}
            }
            edge from parent node[left] {S1}
        }
        child {node {x = -3, y = -8}
            child {node {x = -11, y = -8}
                edge from parent node[right] {S1}
            }
            edge from parent node[right] {S2}
        }
        edge from parent node[right=3pt] {S3}
    };
\end{tikzpicture}
```

Assim, temos que ao final da execução do programa $x \in \{-11, 2,5\}$ e $y \in \{-8, -3, 2\}$.

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
