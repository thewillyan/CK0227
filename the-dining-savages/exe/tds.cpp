#include <CLI11/CLI11.hpp>

#define TDS_VERSION "0.1.0"

int main(int argc, char **argv) {
  CLI::App app("The Dining Savages problem");

  app.set_version_flag("-v,--version", std::string(TDS_VERSION));

  int n_savages{0};
  app.add_option("-s,--savages", n_savages, "Number of Savage threads")->required(true);

  CLI11_PARSE(app, argc, argv);
  
  return 0;
}
