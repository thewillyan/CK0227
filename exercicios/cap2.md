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

Este programa tem 2 processos, cada um com 1 única ação atômica resultando em $2! = 2$ possíveis histórias.
Entretanto, como ao início do programa a condição `x > y` é falsa não há história que o primeiro processo execute primeiro.
Portanto, `<S3;>` ocorrerá transformando o estado do programa para `x = -3` e `y = 5` e terminando  o primeiro processo.

Porém, mesmo após `<S3;>` executar a condição `x > y` continua sendo falsa e o segundo processo nunca termina deixando o programa no estado 
`x = -3` e `y = 5`.

## 18. Consider the program bellow. For what initial values of `x` does the program terminate, assuming scheduling is weakly fair? What are the corresponding final values? Explain your answer.

```
co <await (x > 0) x = x - 1;>
// <await (x < 0) x = x + 2;>
// <await (x == 0) x = x - 1;>
oc
```


Para que a condição de término seja atingida é necessário que:

1. As condições $x > 0$, $x < 0$ e $x = 0$ sejam atingidas em algum momento da execução;
2. Uma vez que sejam atingidas não mudem até que seu processo correspondente seja escolhido pelo escalonador;

Dado que $x > 0$, $x < 0$ e $x = 0$ não podem ser verdade ao mesmo tempo temos que apenas uma delas deve ser verdade no início do programa.

**I. Assumindo que $x > 0$ é verdadeiro:**

Temos que `x = x - 1` é executado e que $x > 0 \Rightarrow x - 1 > -1$.
Portanto, depois da execução, temos $y > -1$ é verdadeiro, sendo $y$ o novo valor de `x`.

Dado que estamos tentanto atingir o término $y < 0 \vee y = 0$ deve
ser verdadeiro.

**I.I. Assumindo que $y < 0$ é verdadeiro:**

`x = x + 2` é executado e que $-1 < y < 0$ temos que, após a execução, $1 < z < 2$
sendo $z$ o novo valor de `x`.

Para que o programa termine é necessário que

$$z = 0 \Rightarrow y + 2 = 0 \Rightarrow x - 1 + 2 = 0 \Rightarrow x = -1$$

**I.II. Assumindo que $y = 0$:**

Assim, temos que $x - 1 = 0 \Rightarrow x = 1$.

Além disso, `x = x - 1` é executado, e assim, $z = -1$, sendo $z$ o novo valor de `x`.

Dado que $z < 0$ o programa termina com $x = 1$.

**II. Assumindo que $x < 0$ é verdadeiro:**

Temos que `x = x + 2 ` é executado e que $y < 2$, sendo $y$ o novo valor de `x`.

Para que haja termino $y > 0 \vee y = 0$ deve ser verdadeiro.

**II.I. Assumindo que $y > 0$ é verdadeiro:**

Temos que `x = x - 1` é executado e que $z > -1$, sendo $z$ o novo valor de `x`.

Para que haja término $z = 0$ deve ser verdadeiro, portanto

$$z = 0 \Rightarrow y - 1 = 0 \Rightarrow x + 2 - 1 = 0 \Rightarrow x = -1$$

**II.II. Assumindo que $y = 0$ é verdadeiro:**

Temos que `x = x - 1` é executado e que $z = -1$, sendo $z$ o novo valor de `x`.

Dessa forma temos

$$z = -1 \Rightarrow y - 1 = -1 \Rightarrow x + 1 - 1 = -1 \Rightarrow x = -1$$

**III. Assumindo que $x = 0$ é verdadeiro:**

Temos que `x = x - 1` é executado, assim $y = -1$, sendo $y$ o novo valor de `x`.

Para que haja término $y > 0 \vee y < 0$ deve ser verdadeira.
Dado que $y = -1 \Rightarrow y < 0$, `x = x + 2` é executado com $z = 1$, sendo $z$ o novo valor de `x`.

Análogamente $z = 1 \Rightarrow z > 0$, `x = x - 1` é executado, terminando assim o programa para $x = 0$.

**Conclusão:**
Assim para que o programa termine é necessário que $x \in \{-1, 0, 1\}$.

## 33. Consider the following program:

```
int x = 0, c = true;
co <await x == 0>; c = false;
// while (c) <x = x - 1>;
oc
```

### (a) Will the program terminate if scheduling is weakly fair? Explain.

Não, dado que nada garate que o `while` seja executado enquanto `c` ainda seja verdadeiro.
Um escalonador fracamente justo apenas garante a execução das ações atômicas condicionais
as quais a condição continua verdadeira até que seja vista pelo escalonador.

### (b) Will the program terminate if scheduling is strongly fair? Explain.

Não, pois a condição `c` não é incondicionalmente verdadeira.

### (c) Add the code bellow as a third arm of the `co` statement. Repeat parts (a) and (b) for this three-process program.

```
while (c) {if (x < 0) <x = 10>;}
```

As respostas continuam as mesmas dado que esse terceiro braço de nada interfere com condição `c`.
