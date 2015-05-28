#!/bin/bash

mkdir -p ./backup

~/.cache/cargo/bin/factorio backup
~/.cache/cargo/bin/minecraft.sh backup vanilla

exit 0
