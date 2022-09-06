#!/bin/sh

# SPDX-FileCopyrightText: 2022 Empo Inc.
#
# SPDX-License-Identifier: CC0-1.0

source /etc/lsb-release

cat <<EOF > /etc/apt/sources.list.d/multiarch.list
deb [arch=armhf,arm64] http://ports.ubuntu.com/ $DISTRIB_CODENAME main restricted
deb [arch=armhf,arm64] http://ports.ubuntu.com/ $DISTRIB_CODENAME-updates main restricted
deb [arch=armhf,arm64] http://ports.ubuntu.com/ $DISTRIB_CODENAME universe
deb [arch=armhf,arm64] http://ports.ubuntu.com/ $DISTRIB_CODENAME-updates universe
deb [arch=armhf,arm64] http://ports.ubuntu.com/ $DISTRIB_CODENAME multiverse
deb [arch=armhf,arm64] http://ports.ubuntu.com/ $DISTRIB_CODENAME-updates multiverse
deb [arch=armhf,arm64] http://ports.ubuntu.com/ $DISTRIB_CODENAME-backports main restricted universe multiverse
EOF

