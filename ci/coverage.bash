#!/usr/bin/env bash

set -ex

upload_coverage() {
    for file in $(ls target/x86_64-unknown-linux-gnu/debug/color_bruteforcer-* | grep -v "\.d"); do
        mkdir -p "target/cov/$(basename $file)";
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file";
    done

    bash <(curl -s https://codecov.io/bash)
}

main() {
    upload_coverage
}

main
