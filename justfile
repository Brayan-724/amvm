compile TARGET OUTPUT="build.amb":
  cargo run -- compile {{TARGET}} {{OUTPUT}}
  @echo "Visualize output with hexyl"
  @hexyl {{OUTPUT}}

jit TARGET:
  cargo run -- jit {{TARGET}}

jit-ex TARGET:
  just jit examples/{{TARGET}}

test TARGET OUTPUT="build.amb":
  cargo build
  @{{ justfile_directory() }}/target/debug/amvm compile {{TARGET}} {{OUTPUT}}; \
  if [[ $? = "0" ]]; then \
    echo -e "\x1b[32m✓ Build {{TARGET}}\x1b[0m"; \
  else \
    echo -e "\x1b[33m╳ Build {{TARGET}} ($?)\x1b[0m"; \
    exit 1; \
  fi;

  @hexyl {{OUTPUT}}

  @{{ justfile_directory() }}/target/debug/amvm run {{OUTPUT}}; \
  if [[ $? = "0" ]]; then \
    echo -e "\x1b[32m✓ Run {{TARGET}}\x1b[0m"; \
  else \
    echo -e "\x1b[33m╳ Run {{TARGET}} ($?)\x1b[0m"; \
  fi;

[unix]
test-all-examples:
  cargo build --release
  @for f in examples/*.aml3; do \
    {{ justfile_directory() }}/target/release/amvm compile $f build.amb; \
    if [[ $? = "0" ]]; then \
      {{ justfile_directory() }}/target/release/amvm inspect build.amb > /dev/null; \
      if [[ $? = "0" ]]; then \
        echo -e "\x1b[32m✓ $f\x1b[0m"; \
      else \
        echo -e "\x1b[33m╳ $f ($?)\x1b[0m"; \
      fi; \
    else \
      echo -e "\x1b[33m╳ $f ($?)\x1b[0m"; \
    fi; \
  done;
