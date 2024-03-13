compile TARGET OUTPUT="build.amb":
  cargo run -- compile {{TARGET}} {{OUTPUT}}
  @echo "Visualize output with hexyl"
  @hexyl {{OUTPUT}}

jit TARGET:
  cargo run -- jit {{TARGET}}

jit-ex TARGET:
  just jit examples/{{TARGET}}
