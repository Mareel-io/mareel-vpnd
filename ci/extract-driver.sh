#!/bin/sh

# SPDX-FileCopyrightText: 2022 Empo Inc.
#
# SPDX-License-Identifier: CC0-1.0

unzip -j wireguard-nt.zip wireguard-nt/bin/amd64/wireguard.dll -d bin/x86_64-pc-windows-msvc/release/
unzip -j windll.zip windns/x64-Release/windns.dll -d bin/x86_64-pc-windows-msvc/release/
