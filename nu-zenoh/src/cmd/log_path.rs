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
use std::{fs::File, path::PathBuf};

use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature, Type, Value,
};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::{signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct LogPath {
    _state: State,
    log_path: PathBuf,
}

impl LogPath {
    pub(crate) fn new(state: State) -> Self {
        let log_path = tempfile::tempdir()
            .unwrap()
            .keep()
            .join("zenoh.log")
            .to_path_buf();

        const ENV_FILTER_NAME: &str = "ZENOH_NU_LOG";
        const ENV_FILTER_DEFAULT: &str = "zenoh=trace";

        let env_filter = EnvFilter::try_from_env(ENV_FILTER_NAME)
            .unwrap_or_else(|_| EnvFilter::new(ENV_FILTER_DEFAULT));

        let fmt = tracing_subscriber::fmt::layer()
            .with_writer(File::create(&log_path).unwrap())
            .with_ansi(false)
            .with_span_events(FmtSpan::ACTIVE);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt)
            .init();

        Self {
            _state: state,
            log_path,
        }
    }
}

impl Command for LogPath {
    fn name(&self) -> &str {
        "zenoh log-path"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::String)
    }

    fn description(&self) -> &str {
        "Get the log file path for the specified session"
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        // FIXME(fuzzypixelz): it's not possible to filter tracing events by
        // session as the zenoh crate doesn't (yet) provide that information.
        Ok(PipelineData::Value(
            Value::string(self.log_path.to_string_lossy(), call.head),
            None,
        ))
    }
}
