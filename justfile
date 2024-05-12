amvm TARGET *args='':
  @{{ justfile_directory() }}/target/{{TARGET}}/amvm {{args}}

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
  @just amvm debug compile {{TARGET}} {{OUTPUT}}; \
  if [[ $? = "0" ]]; then \
    echo -e "\x1b[32m✓ Build {{TARGET}}\x1b[0m"; \
  else \
    echo -e "\x1b[33m╳ Build {{TARGET}} ($?)\x1b[0m"; \
    exit 1; \
  fi;

  @hexyl {{OUTPUT}}

  @just amvm debug run {{OUTPUT}}; \
  if [[ $? = "0" ]]; then \
    echo -e "\x1b[32m✓ Run {{TARGET}}\x1b[0m"; \
  else \
    echo -e "\x1b[33m╳ Run {{TARGET}} ($?)\x1b[0m"; \
  fi;

[unix]
test-all-examples:
  cargo build --release
  @for f in examples/*.aml3; do \
    just amvm release compile $f build.amb; \
    if [[ $? = "0" ]]; then \
      just amvm release inspect build.amb > /dev/null; \
      if [[ $? = "0" ]]; then \
        echo -e "\x1b[32m✓ $f\x1b[0m"; \
      else \
        echo -e "\x1b[33m╳ $f ($?)\x1b[0m"; \
      fi; \
    else \
      echo -e "\x1b[33m╳ $f ($?)\x1b[0m"; \
    fi; \
  done;

[unix]
run-all-examples *args='.skip.aml3':
  @echo -e "\x1b[1mSKIPPING: \x1b[31m{{ args }}\x1b[0m"

  cargo build --release
  @for f in examples/*.aml3; do \
    just should-be-ignored $f {{args}} &> /dev/null; \
    if [[ $? = "1" ]]; then \
      echo -e "\x1b[2m - Skipping $f\x1b[0m"; \
    else \
    just amvm release compile $f build.amb; \
    if [[ $? = "0" ]]; then \
    just amvm release inspect build.amb > /dev/null; \
    if [[ $? = "0" ]]; then \
      if [[ $f =~ ".fail" ]]; then \
        just amvm release run build.amb &> /dev/null; \
        if [[ $? = "1" ]]; then \
          echo -e "\x1b[2;32m ✓ Tested $f\x1b[0m"; \
        else \
          echo -e "\x1b[33m ╳ Test failed $f ($?)\x1b[0m"; \
        fi; \
      else \
        just amvm release run build.amb > /dev/null; \
        if [[ $? = "0" ]]; then \
          echo -e "\x1b[2;32m ✓ Tested $f\x1b[0m"; \
        else \
          echo -e "\x1b[33m ╳ Test failed $f ($?)\x1b[0m"; \
        fi; \
      fi; \
    else \
      echo -e "\x1b[33m ╳ Build failed $f ($?)\x1b[0m"; \
    fi; \
    else \
      echo -e "\x1b[33m ╳ Build failed $f ($?)\x1b[0m"; \
    fi; \
    fi; \
  done;

[unix]
should-be-ignored TARGET *rules='':
    @bash -c 'f=$1; shift; while (( "$#" )); do if [[ $f =~ "$1" ]]; then exit 1; fi; shift; done' -- {{ TARGET }} {{ rules }}
