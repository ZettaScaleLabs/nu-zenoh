#!/usr/bin/env nu

use std/log

const cwd = path self | path dirname

def main [] {
    help main
}

def "main test" [] {
    cd $cwd

    cargo build -q --manifest-path ($cwd | path join Cargo.toml) -p nuze --bin nuze
    let nuze = $cwd | path join target debug nuze

    const always_exclude = "**/_*.nu"

    for test in (glob tests/**/*.nu --exclude [$always_exclude experimental/**]) {
        try {
            ^$nuze -0 $test
        } catch { |err|
            ko $test $nuze
            continue
        }

        ok $test
    }

    for test in (glob tests/experimental/**/*.nu --exclude [$always_exclude]) {
        try {
            ^$nuze -X0 $test
        } catch { |err|
            ko $test $nuze
            continue
        }

        ok $test
    }
}

def ko [test, nuze, -x] {
    if $x {
        print -e $"(ansi red_bold)KO:(ansi reset) ($test | path relative-to ($cwd)/tests) \(experimental\); rerun with `($nuze) -X0 ($test)`"
    } else {
        print -e $"(ansi red_bold)KO:(ansi reset) ($test | path relative-to ($cwd)/tests); rerun with `($nuze) -0 ($test)`"
    }
}

def ok [test, -x] {
    if $x {
        print -e $"(ansi green_bold)OK:(ansi reset) ($test | path relative-to ($cwd)/tests) \(experimental\)"
    } else {
        print -e $"(ansi green_bold)OK:(ansi reset) ($test | path relative-to ($cwd)/tests)"
    }
}
