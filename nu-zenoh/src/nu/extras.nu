#!/usr/bin/env nu
#
# Copyright (c) 2025 ZettaScale Technology
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
#

$env.PROMPT_COMMAND = { ||
    if (zenoh session list | where name == 'default' | is-not-empty) {
        $"(ansi light_blue_bold)(zenoh zid --short)(ansi reset)"
    } else {
        ""
    }
}
$env.PROMPT_INDICATOR = $"(ansi light_blue_bold)> (ansi reset)"

alias 'zenoh open' = zenoh session open

# Various commands to interact with Zenoh systems.
#
# You must use one of the following subcommands. Using this command as-is will only produce this help message.
export def "zenoh" [] {
    help zenoh
}
