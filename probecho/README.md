# Probecho

Implementação de técnicas de _broadcasting_ em grafos arbtrários.

# Executáveis

Este programa possuí alguns executáveis que se utilizam da nossa bibilioteca.
Segue abaixo uma pequena descrição sobre eles e como executá-los.

## Ping-pong

Uma demonstração simples de [IPC](https://en.wikipedia.org/wiki/Inter-process_communication) com
[iceoryx2](https://docs.rs/iceoryx2/latest/iceoryx2/index.html).

```shell
cargo build --bin ping-pong-node && cargo run --bin ping-pong
```

## Spanning Tree Broadcast Simulation

Simula um Spanning Tree Broadcast com 8 nós em uma topologia arbtrária onde o nó iniciador
manda uma menssagem para um nó reptor.

A simulação termina quando o nó iniciador recebe uma resposta do receptor.

A quantidade de nós deve ser uma constante por isso deve ser alterada no binário (por hora).

```shell
cargo run --bin stb-simulation manager -t ring -d 5
```
