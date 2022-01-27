#!/bin/bash

# SPDX-FileCopyrightText: 2022 Empo Inc.
#
# SPDX-License-Identifier: CC0-1.0

cd $(dirname "$0")
cd ..

reuse > /dev/null
if [ "$?" != 0 ]; then
    echo "Reuse is not installed on system"
    echo "Please install reuse on your system"
    echo "Using this guide: https://github.com/fsfe/reuse-tool#install"
fi

SRCS=$(find ./src -name "*.rs")

# Rust header
reuse addheader --style c --copyright "Empo Inc." --template mareel-rust --license "GPL-3.0-or-later" $SRCS
# Misc files
FILES=""
