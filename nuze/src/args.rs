//
// Copyright (c) 2025 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use std::path::PathBuf;

#[derive(clap::Parser, Clone, Debug)]
#[command(
    version=env!("CARGO_PKG_VERSION"),
    about="Zenoh Interactive Shell",
    long_about="Zenoh Interactive Shell (zetta-r2i) is a standalone command-line \
        interpreter that extends Nu with the Zenoh plugin.",
)]
pub(crate) struct Args {
    #[arg(
        value_name = "SCRIPT",
        help = "Path to a Nu script file",
        conflicts_with_all(["commands", "execute"])
    )]
    pub script: Option<PathBuf>,

    #[arg(
        short = 'c',
        long = "commands",
        value_name = "COMMANDS",
        help = "Run the given Nu program in the Zenoh context and then exit",
        conflicts_with_all(["script", "execute"])
    )]
    pub commands: Option<String>,
    #[arg(
        short = 'e',
        long = "execute",
        value_name = "COMMANDS",
        help = "Run the given Nu program in the Zenoh context and then drop into a REPL",
        conflicts_with_all(["script", "commands"])
    )]
    pub execute: Option<String>,
    #[arg(
        long = "internal-options",
        help = "Enable internal Zenoh options and commands"
    )]
    pub internal_options: bool,
    #[arg(long = "no-default-session", help = "Don't open a 'default' session")]
    pub no_default_session: bool,
}
