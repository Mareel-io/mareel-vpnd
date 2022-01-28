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
    exit -1
fi


# License boilerplate for mareel-vpnd
SRCS=$(find ./mareel-vpnd -name "*.rs")
reuse addheader --style c --copyright "Empo Inc." --template mareel-rust --license "GPL-3.0-or-later" $SRCS

# License boilerplate for wgctrl
SRCS=$(find ./wgctrl -name "*.rs")
reuse addheader --style c --copyright "Empo Inc." --template mareel-rust --license "GPL-3.0-or-later" $SRCS

# Boilerplate for talpid-dbus
SRCS=$(find ./talpid-dbus -name "*.rs")
reuse addheader --style c --copyright "Empo Inc." --template mareel-rust --license "GPL-3.0-or-later" $SRCS
