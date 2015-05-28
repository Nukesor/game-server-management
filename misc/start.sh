#!/bin/bash

mkdir -p ./backup

# Shooter
~/.cache/cargo/bin/ut2004 startup am
~/.cache/cargo/bin/cod4 startup normal
~/.cache/cargo/bin/garrys startup ttt

#~/.cache/cargo/bin/factorio startup
#~/.cache/cargo/bin/minecraft startup vanilla

exit 0
