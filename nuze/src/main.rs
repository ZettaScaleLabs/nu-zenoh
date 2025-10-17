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
use std::{
    env, process,
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use clap::Parser;
use nu_protocol::{
    engine::{EngineState, Stack},
    BannerKind, Config, PipelineData, Signals,
};

mod args;

fn main() {
    let entire_start_time = Instant::now();
    let args = args::Args::parse();

    let options = nu_zenoh::Config {
        internal_options: args.internal_options,
        no_default_session: args.no_default_session,
    };

    let (mut engine_state, mut stack) = nu_context(options);
    ctrlc_protection(&mut engine_state);

    nu_cli::eval_source(
        &mut engine_state,
        &mut stack,
        nu_zenoh::ZENOH_CONTEXT_EXTRAS,
        "<zenoh-context-extras>",
        PipelineData::Empty,
        false,
    );

    // NOTE: script, commands and execute are mutually exclusive

    if let Some(script) = args.script {
        if let Err(err) = nu_cli::evaluate_file(
            script
                .to_str()
                .expect("script path is not valid UTF-8")
                .to_string(),
            &[],
            &mut engine_state,
            &mut stack,
            PipelineData::Empty,
        ) {
            eprintln!("Error evaluating script '{}': {err}", script.display());
            process::exit(1);
        } else {
            return;
        }
    }

    if let Some(commands) = args.commands {
        let code = nu_cli::eval_source(
            &mut engine_state,
            &mut stack,
            commands.as_bytes(),
            "<commands>",
            PipelineData::Empty,
            true,
        );
        process::exit(code);
    }

    if let Some(execute) = args.execute {
        nu_cli::eval_source(
            &mut engine_state,
            &mut stack,
            execute.as_bytes(),
            "<commands>",
            PipelineData::Empty,
            false,
        );
    }

    if let Err(err) = nu_cli::evaluate_repl(&mut engine_state, stack, None, None, entire_start_time)
    {
        eprintln!("Error starting REPL: {err}");
        process::exit(1);
    }
}

fn nu_context(options: nu_zenoh::Config) -> (EngineState, Stack) {
    let config = Config {
        show_banner: BannerKind::None,
        ..Default::default()
    };

    let stack = Stack::new();

    let mut engine_state = nu_cmd_lang::create_default_context();
    engine_state.set_config(config);
    engine_state = nu_command::add_shell_command_context(engine_state);
    engine_state = nu_cmd_extra::add_extra_command_context(engine_state);
    engine_state = nu_cli::add_cli_context(engine_state);
    engine_state = nu_explore::add_explore_context(engine_state);
    engine_state = nu_zenoh::add_zenoh_context(engine_state, options);
    {
        let delta = {
            let mut working_set = nu_protocol::engine::StateWorkingSet::new(&engine_state);
            working_set.add_decl(Box::new(nu_cli::NuHighlight));
            working_set.add_decl(Box::new(nu_cli::Print));
            working_set.render()
        };

        engine_state
            .merge_delta(delta)
            .expect("failed to merge nu-highlight and print");
    };

    nu_cli::gather_parent_env_vars(
        &mut engine_state,
        &env::current_dir().expect("could not get current dir"),
    );

    (engine_state, stack)
}

/// Prevents the REPL from existing on Ctrl-C
fn ctrlc_protection(engine_state: &mut EngineState) {
    let signals = Signals::new(Arc::new(AtomicBool::new(false)));
    engine_state.set_signals(signals.clone());
    ctrlc::set_handler(move || signals.trigger()).expect("could not set Ctrl-C handler");
}
