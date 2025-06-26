#pragma once

namespace TDS {
class Pot {
public:
  Pot(int m);

  void refill(int m);

  int getM() const;

private:
  int available_;
  int m_;
};

} // namespace TDS