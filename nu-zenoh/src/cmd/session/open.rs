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

use nu_engine::CallExt;
use nu_protocol::{
    LabeledError, PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
    engine::{Call, Command, EngineState, Stack},
};
use zenoh::{Wait, session};

use crate::{State, call_ext2::CallExt2, conv, signature_ext::SignatureExt};

#[derive(Clone)]
pub(crate) struct Open {
    state: State,
}

impl Open {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Open {
    fn name(&self) -> &str {
        "zenoh session open"
    }

    fn signature(&self) -> Signature {
        let sig = Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::Nothing)
            .named(
                "config-file",
                SyntaxShape::Filepath,
                "Path to a Zenoh configuration file",
                None,
            )
            .optional(
                "config",
                SyntaxShape::Record(vec![]),
                "Zenoh configuration object (see https://github.com/eclipse-zenoh/zenoh/blob/main/DEFAULT_CONFIG.json5)",
            );

        if self.state.options.internal_options {
            sig.named("runtime", SyntaxShape::String, "Runtime name", None)
        } else {
            sig
        }
    }

    fn description(&self) -> &str {
        "(re)Open a session"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        // FIXME(fuzzypixelz): refactor this (see 'zenoh runtime open')

        let file_path = call.get_flag::<PathBuf>(engine_state, stack, "config-file")?;
        let runtime_name = call.get_flag::<String>(engine_state, stack, "runtime")?;
        let config_record = call.opt::<Value>(engine_state, stack, 0)?;

        let config = match (
            file_path.as_ref(),
            config_record.as_ref(),
            runtime_name.as_ref(),
        ) {
            (Some(file_path), None, None) => zenoh::Config::from_file(file_path).map_err(|e| {
                nu_protocol::LabeledError::new("Failed to load config file").with_label(
                    format!("Could not read config from {}: {}", file_path.display(), e),
                    call.head,
                )
            })?,
            (None, Some(config_record), None) => match config_record {
                val @ Value::Record { .. } => {
                    let json_value =
                        conv::value_to_json_value(engine_state, val, call.head, false)?;
                    zenoh::Config::from_json5(&json_value.to_string()).map_err(|e| {
                        nu_protocol::LabeledError::new("Failed to parse config record")
                            .with_label(format!("Could not parse config record: {e}"), call.head)
                    })?
                }
                _ => {
                    return Err(ShellError::GenericError {
                        error: "Invalid config type".to_string(),
                        msg: "Config must be a record".to_string(),
                        span: Some(call.head),
                        help: Some("Provide a record with Zenoh configuration options".to_string()),
                        inner: vec![],
                    });
                }
            },
            (None, None, Some(runtime_name)) => {
                let runtime = self
                    .state
                    .runtimes
                    .read()
                    .unwrap()
                    .get(runtime_name)
                    .ok_or_else(|| {
                        LabeledError::new(format!("runtime '{runtime_name}' was not found"))
                    })?
                    .clone();

                let session_name = call.session(engine_state, stack)?;
                let mut sessions = self.state.sessions.write().unwrap();
                if let Some(sess) = sessions.remove(&session_name) {
                    sess.close().wait().map_err(|e| {
                        nu_protocol::LabeledError::new(
                            "Failed to reopen Zenoh session '{session_name}'",
                        )
                        .with_label(format!("Could not close Zenoh session: {e}"), call.head)
                    })?
                }
                let new_session = session::init(runtime.into()).wait().map_err(|e| {
                    nu_protocol::LabeledError::new("Failed to open Zenoh session")
                        .with_label(format!("Could not establish Zenoh session: {e}"), call.head)
                })?;
                sessions.insert(session_name, new_session);
                return Ok(PipelineData::Value(Value::nothing(call.head), None));
            }
            (None, None, None) => zenoh::Config::default(),
            _ => {
                return Err(ShellError::GenericError {
                    error: "Conflicting arguments".to_string(),
                    msg: "Only one of RECORD, --config-file or --runtime can be specified"
                        .to_string(),
                    span: Some(call.head),
                    help: None,
                    inner: vec![],
                });
            }
        };

        let session_name = call.session(engine_state, stack)?;
        let mut sessions = self.state.sessions.write().unwrap();
        if let Some(sess) = sessions.remove(&session_name) {
            sess.close().wait().map_err(|e| {
                nu_protocol::LabeledError::new("Failed to reopen Zenoh session '{session_name}'")
                    .with_label(format!("Could not close Zenoh session: {e}"), call.head)
            })?
        }
        let new_session = zenoh::open(config).wait().map_err(|e| {
            nu_protocol::LabeledError::new("Failed to open Zenoh session")
                .with_label(format!("Could not establish Zenoh session: {e}"), call.head)
        })?;
        sessions.insert(session_name, new_session);

        Ok(PipelineData::Value(Value::nothing(call.head), None))
    }
}
