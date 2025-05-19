---
header-includes: |
  \usepackage{tikz}
  \usetikzlibrary{trees}
  \usepackage[a4paper, margin=3cm]{geometry}
output: pdf_document
---

# Exercícios Capítulo 2

Resolução dos exercícios 2.1, 2.8, 2.13, 2.15, 2.18, 2.24 e 2.33 do livro *Foundations of Multithreaded, Parallel, and Distributed Programming*.

## 1. Consider the outline of the program in Figure 2.1 that prints all the lines in a file that contain pattern.

### (a) Develop the missing code for synchronizing access to buffer. Use the await statement to program the synchronization code.

Para desenvolvimento da sincronização é necessário:

```
string buffer;
bool done = false;
bool bufferFull = false;
co #process 1:
    string line1;
    while (true) {
        <await (bufferFull || done);>
        if (done) break;
        line1 = buffer;
        bufferFull = false;
        look for pattern in line 1;
        if (pattern is in line1)
            write line1;
    }
// # process 2:
    string line2;
    while(true) {
        read next line of input into line2;
        if (EOF) {done = true; brea;}
        <await (bufferFull == false);>
        buffer = line 2;
        bufferFull = true;
    }
oc;
```

### (b) Extend your program so that it reads two files and prints all the lines that contain pattern. Identify the independent activities and use a separate process for each. Show all synchronization code that is required.

```
string buffer;
bool doneProcessFile1 = false;
bool doneProcessFile2 = false;
bool bufferFull = false;
co #process 1:
    string line1;
    while (true) {
        <await (buffer || doneProcessFile1 || doneProcessFile2);>
        if (doneProcessFile1 || doneProcessFile2) break;
        line1 = buffer;
        bufferFull = false
        look for pattern in line1;
        if (pattern is in line1)
            write line1;
    }
// #process 2:
    string line2;
    while(true) {
        read next line of input into line2;
        if (EOF) {doneProcessFile1 = true; break;}
        <await (bufferFull == false);>
        buffer = line2;
        bufferFull = true
    }
// # process 3:
    string line3;
    while(true) {
        read next line of input into line3;
        if (EOF) {doneProcessFile2 = true; break;}
        <await (bufferFull == false);>
        buffer = line3;
        bufferFull = true;
    }
oc;
```

## 8. A queue is often represented using a linked list. Assume that two variables, `head` and `tail`, point to the first and last elements of the list. Each element contains a data field and a link to the next element. Assume that a null link is represented by the constant `null`.

### (a) Write routines to:

**1.** Search the list for the first element (if any) that contains data value `d`. Should return `null` if they cannot succeed.

```
# receives the searched item d
link ptr = head;

while (ptr != null) {
    if (ptr.data == d)
        break;
    ptr = ptr.next;
}

return ptr;
```

**2.** Insert a new element at the end of the list.

```
# receives a new element e
if (tail == null) {
    head = e;
    tail = e;
} else {
    if (head == tail)
        head.next = e;
    tail.next = e;
    tail = e;
}
```

**3.** Delete the element from the front of the list. Should return `null` if they cannot succeed.

```
int data = tail.data;

if (head == tail)
    head = null;
tail = null;

return data;
```

### (b) Now assume that several processes access the linked list. Identify the read and write sets of each routine, as defined in (2.1). Which  combinations of routines can be executed in parallel? Which combinations of routines must execute one at a time (i.e., atomically)?

Conjuntos de leitura:

1. Search routine: {`d`, `head`}
2. Insert routine: {`e`}
3. Delete routine: {`data`}

Conjuntos de escrita:

1. Search routine: {`ptr`}
2. Insert routine: {`head`, `tail`}
3. Delete routine: {`head`, `tail`}

A rotina de pesquisa pode ser executada paralelamente com a rotina de inserção.
Todas as demais possibilidades devem ser realizadas como ações atômicas.

### (c) Add synchronization code to the three routines to enforce the synchronization you identified in your answer to (b). Make your atomic actions as small as possible, and do not delay a routine unnecessarily. Use the await statement to program the synchronization code.

**1.** Search the list for the first element (if any) that contains data value `d`. Should return `null` if they cannot succeed.

```
# receives the searched item d
link ptr = head;

while (ptr != null and head != null) {
    if (ptr.data == d)
        return ptr;
    if ((ptr == tail or ptr.next == tail) and removing)
        break;
    ptr = ptr.next;
}

return null;
```

**2.** Insert a new element at the end of the list.

```
# receives a new element e
if (tail == null) {
    <head = e;
    tail = e;>
} else {
    <if (head == tail)
        head.next = e;
    tail.next = e;
    tail = e;>
}
```

**3.** Delete the element from the front of the list. Should return `null` if they cannot succeed.

```
bool removing = true;
int data = tail.data;

<if (head == tail)
    head = null;
tail = null;>

bool removing = false;
return data;
```

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

## 15. Consider the following program:

```
int x = 0, y = 10; 
co while (x!=y) x = x + 1; 
// while (x != y) y = y - 1; 
oc
```

### (a) Does the program meet the requirements of the At-Most-Once Property (2.2)? Explain
A propriedade de *At-Most-Once* não é respeitada dado que em ambos os processos fazem referência para uma variável que é alterada em outro.

### (b) Will the program terminate? Always? Sometimes? Never1 Explain your answer

As vezes. A terminação depende da ordem em que ocorre a execução. Em situações onde a intercalação ocorra até que $x$ e $y$ foquem iguais então o programa irá terminar normalmente, contudo no caso de que um momento onde por emxemplo $x = 4$ e $y = 5$, $x$ poderá passar para $5$ e $y$ para $4$, assim $x > y$, dessa forma teremos então que o programa não vai terminar.

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

## 24. Consider the following precondition and assignment statement

```
{x >= 4} <x = x - 4;>
```

For each of the following triples, show whether the above statement interferes with the triple:

### (a) `{x >= 0} <x = x + 5;> {x >= 5}`
- $a = \langle x = x-4; \rangle$
- $C = \{x \geq 0\}$
- $pre(a) = \{x \geq 4\}$
$$
	NI(a, C): \{(x \geq 0) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 0\}  
$$
Fazendo a simplificação da pré-condição teremos:
$$
\{(x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 0\}
$$
$$
\{x - 4 \geq 4\} \ \langle x = x-4; \rangle \ \{x\geq 0\} \text{(axioma da atribuição)}
$$
Fazendo a simplificação teremos:
$$
\{x \geq 8\} \ \langle x = x-4; \rangle \ \{x\geq 0\}
$$
A partir da regra da consequência, para $\{x \geq 8\} \implies \{x \geq 4\}$
$$
\therefore \{(x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 0\} \ \text{c.q.d}
$$
- $a = \langle x = x-4; \rangle$
- $C = \{x \geq 5\}$
- $pre(a) = \{x \geq 4\}$
$$
	NI(a, C): \{(x \geq 5) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 5\}  
$$
Fazendo a simplificação da pré-condição teremos:
$$
\{(x \geq 5)\} \ \langle x = x-4; \rangle \ \{x\geq 5\}
$$
$$
\begin{align*}
&\{x - 4 \geq 5\} \ \langle x = x-4; \rangle \ \{x\geq 5\} \text{(axioma da atribuição)} \\
&\{x \geq 9\} \ \langle x = x-4; \rangle \ \{x\geq 5\} \text{(simplificação)} \\
&\{x \geq 5\} \ \langle x = x-4; \rangle \ \{x\geq 5\} \text{(A partir da regra da consequência onde} \ \{x \geq 9 \implies x \geq 5\} \\
&\text{c.q.d}
\end{align*}
$$
Assim temos que não ocorre interferência!

### (b) `{x >= 0} <x = x + 5;> {x >= 0}`
- $a = \langle x = x-4; \rangle$
- $C = \{x \geq 0\}$
- $pre(a) = \{x \geq 4\}$
$$
	NI(a, C): \{(x \geq 0) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 0\}  
$$
Fazendo a simplificação da pré-condição teremos:
$$
\{(x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 0\}
$$
$$
\{x - 4 \geq 4\} \ \langle x = x-4; \rangle \ \{x\geq 0\} \text{(axioma da atribuição)}
$$
Fazendo a simplificação teremos:
$$
\{x \geq 8\} \ \langle x = x-4; \rangle \ \{x\geq 0\}
$$
A partir da regra da consequência, para $\{x \geq 8\} \implies \{x \geq 4\}$
$$
\therefore \{(x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 0\} \ \text{c.q.d}
$$

Segue a mesma prova anterior, logo não ocorre insterferência!

### (c) `{x >= 10} <x = x + 5;> {x >= 11}`

- $a = \langle x = x-4; \rangle$
- $C = \{x \geq 10\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \geq 10) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 10\}  
$$

Fazendo a simplificação da pré-condição teremos:

$$
\begin{align*}
&\{x \geq 10\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \\  
&\{x - 4 \geq 10\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \text{(axioma da atribuição)} \\
&\{x \geq 14\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \text{(simplificação)} \\
&\{x \geq 10\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \text{(A partir da regra da consequência onde} \ \{x \geq 16 \implies x \geq 10\} \\
&\text{c.q.d}
\end{align*}
$$

- $a = \langle x = x-4; \rangle$
- $C = \{x \geq 11\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \geq 11) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 11\}  
$$

Fazendo a simplificação da pré-condição teremos:

$$
\begin{align*}
&\{x \geq 11\} \ \langle x = x-4; \rangle \ \{x\geq 11\} \\  
&\{x - 4 \geq 11\} \ \langle x = x-4; \rangle \ \{x\geq 11\} \text{(axioma da atribuição)} \\
&\{x \geq 15\} \ \langle x = x-4; \rangle \ \{x\geq 11\} \text{(simplificação)} \\
&\{x \geq 11\} \ \langle x = x-4; \rangle \ \{x\geq 11\} \text{(A partir da regra da consequência onde} \ \{x \geq 15 \implies x \geq 11\} \\
&\text{c.q.d}
\end{align*}
$$

Assim temos a não ocorrência de interferências.

### (d) `{x >= 10} <x = x + 5;> {x > = 12}`

- $a = \langle x = x-4; \rangle$
- $C = \{x \geq 10\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \geq 10) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 10\}  
$$

Fazendo a simplificação da pré-condição teremos:

$$
\begin{align*}
&\{x \geq 10\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \\  
&\{x - 4 \geq 10\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \text{(axioma da atribuição)} \\
&\{x \geq 14\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \text{(simplificação)} \\
&\{x \geq 10\} \ \langle x = x-4; \rangle \ \{x\geq 10\} \text{(A partir da regra da consequência onde} \ \{x \geq 16 \implies x \geq 10\} \\
&\text{c.q.d}
\end{align*}
$$

- $a = \langle x = x-4; \rangle$
- $C = \{x \geq 12\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \geq 12) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x\geq 12\}  
$$

Fazendo a simplificação da pré-condição teremos:

$$
\begin{align*}
&\{x \geq 12\} \ \langle x = x-4; \rangle \ \{x\geq 12\} \\  
&\{x - 4 \geq 12\} \ \langle x = x-4; \rangle \ \{x\geq 12\} \text{(axioma da atribuição)} \\
&\{x \geq 16\} \ \langle x = x-4; \rangle \ \{x\geq 12\} \text{(simplificação)} \\
&\{x \geq 12\} \ \langle x = x-4; \rangle \ \{x\geq 12\} \text{(A partir da regra da consequência onde} \ \{x \geq 16 \implies x \geq 12\} \\
&\text{c.q.d}
\end{align*}
$$

### (e) `{x is odd} <x = x + 5;> {x is even}`

- $a = \langle x = x-4; \rangle$
- $C = \{x \ \text{is odd}\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\}  
$$

Fazendo a simplificação da pré-condição teremos:

$$
\begin{align*}
&\{(x \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
&\{(x - 4 \ \text{is odd}) \ \text{e} \ (x - 4 \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
&\{(x \ \text{is odd}) \ \text{e} \ (x \geq 8)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
&\{(x \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
\end{align*}
$$

- $a = \langle x = x-4; \rangle$
- $C = \{x \ \text{is even}\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is even}\}  
$$

Teremos:

$$
\begin{align*}
&\{(x \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is even}\} \\
&\{(x - 4 \ \text{is even}) \ \text{e} \ (x - 4 \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is even}\} \\
&\{(x \ \text{is even}) \ \text{e} \ (x \geq 8)\} \ \langle x = x-4; \rangle \ \{x \ \text{is even}\} \\
&\{(x \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is even}\} \\
\end{align*}
$$

Temos que não ocorre interferência.

### (f) `{x is odd} <y = X + 1> {y is even}`

- $a = \langle x = x-4; \rangle$
- $C = \{x \ \text{is odd}\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\}  
$$

Teremos:

$$
\begin{align*}
&\{(x \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
&\{(x - 4 \ \text{is odd}) \ \text{e} \ (x - 4 \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
&\{(x \ \text{is odd}) \ \text{e} \ (x \geq 8)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
&\{(x \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is odd}\} \\
\end{align*}
$$

- $a = \langle x = x-4; \rangle$
- $C = \{y \ \text{is even}\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(y \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\}  
$$

Teremos:

$$
\begin{align*}
&\{(y \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
&\{(y \ \text{is even}) \ \text{e} \ (x - 4 \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
&\{(y \ \text{is even}) \ \text{e} \ (x \geq 8)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
&\{(y \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
\end{align*}
$$

### (g) `{y is odd} <y = y + 1;> {y is even}`

- $a = \langle x = x-4; \rangle$
- $C = \{y \ \text{is odd}\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(y \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is odd}\}  
$$

Teremos:

$$
\begin{align*}
&\{(y \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is odd}\} \\
&\{(y \ \text{is odd}) \ \text{e} \ (x - 4 \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is odd}\} \\
&\{(y \ \text{is odd}) \ \text{e} \ (x \geq 8)\} \ \langle x = x-4; \rangle \ \{y \ \text{is odd}\} \\
&\{(y \ \text{is odd}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is odd}\} \\
\end{align*}
$$

- $a = \langle x = x-4; \rangle$
- $C = \{y \ \text{is even}\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(y \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\}  
$$

Teremos:

$$
\begin{align*}
&\{(y \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
&\{(y \ \text{is even}) \ \text{e} \ (x - 4 \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
&\{(y \ \text{is even}) \ \text{e} \ (x \geq 8)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
&\{(y \ \text{is even}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{y \ \text{is even}\} \\
\end{align*}
$$

Não temos interferência.

### (h) `{x is a multiple of 3} y = x; {y is a multiple of 3}`

- $a = \langle x = x-4; \rangle$
- $C = \{x \ \text{is a multiple of 3}\}$
- $pre(a) = \{x \geq 4\}$

$$
	NI(a, C): \{(x \ \text{is a multiple of 3}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is a multiple of 3}\}  
$$

Teremos:

$$
\begin{align*}
&\{(x \ \text{is a multiple of 3}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is a multiple of 3}\}   \\
&\{(x - 4 \ \text{is a multiple of 3}) \ \text{e} \ (x - 4 \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is a multiple of 3}\} \\
&\{(x \ \text{is a multiple of 3}) \ \text{e} \ (x \geq 8)\} \ \langle x = x-4; \rangle \ \{x \ \text{is a multiple of 3}\} \\
&\{(x \ \text{is a multiple of 3}) \ \text{e} \ (x \geq 4)\} \ \langle x = x-4; \rangle \ \{x \ \text{is a multiple of 3}\} \\
\end{align*}
$$

Nesse caso não é verdade dado que se $x = 9$, temos que após isso $x$ não será um múltiplo de 3!

Logo teremos a interferência.

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
