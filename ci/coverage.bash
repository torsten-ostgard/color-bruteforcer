#!/usr/bin/env bash

set -ex

build_kcov() {
    wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
    tar xzf master.tar.gz
    cd kcov-master
    mkdir build
    cd build
    cmake ..
    make
    sudo make install
    cd ../..
    rm -rf kcov-master
}

upload_coverage() {
    for file in $(ls target/debug/color_bruteforcer-* | grep -v "\.d"); do
        mkdir -p "target/cov/$(basename $file)";
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file";
    done

    bash <(curl -s https://codecov.io/bash)
}

main() {
    build_kcov
    upload_coverage
}

main
