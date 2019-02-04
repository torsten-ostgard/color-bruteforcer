#!/usr/bin/env bash

set -ex

wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz
cd kcov-master
mkdir build
cd build
cmake ..
make
sudo make install
cd ../..
echo $PATH
ls /usr/local/bin
