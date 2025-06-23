#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.1": *
#show: codly-init.with()

== 4.15. Another way to solve the bounded buffer problem is as follows. Let count be an integer between 0 and n. Then deposit and fetch can be programmed as follows:

#codly(languages: codly-languages)
```
deposit:
  < await (count < n)
    buf[rear] = data;
    rear = (rear+1) % n; count = count+1; >
fetch:
  < await (count > 0)
    result = buf[front];
    front = (front+1) % n; count = count-1; >
```

Implement these await statements using semaphores. {Hint: Use a variation of passing the baton.)

=== Resposta

#codly(languages: codly-languages)
```
deposit:
  P(e);
  if (count == n) {
    V(e);
    P(d);
    P(e);
  }
  buf[rear] = data;
  rear = (rear+1) % n;
  count = count+1;
  SIGNAL;

fetch:
  P(e);
  if (count == 0) {
    V(e);
    P(f);
    P(e);
  }
  result = buf[front];
  front = (front+1) % n;
  count = count-1;
  SIGNAL;

SIGNAL:
  if (count < n) {
    V(d);
  } elseif (count > 0) {
    V(f);
  } else {
    V(e);
  }
```
