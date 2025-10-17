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
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    record, IntoValue, PipelineData, ShellError, Signature, Type, Value,
};

use crate::{signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct List {
    state: State,
}

impl List {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for List {
    fn name(&self) -> &str {
        "zenoh session list"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::record())
    }

    fn description(&self) -> &str {
        "List opened sessions"
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;
        let sessions = self.state.sessions.read().unwrap();
        let session_list = sessions
            .iter()
            .map(|(name, sess)| {
                record!(
                    "name" => name.clone().into_value(span),
                    "zid" => sess.zid().to_string().into_value(span)
                )
                .into_value(span)
            })
            .collect::<Vec<_>>();

        Ok(PipelineData::Value(Value::list(session_list, span), None))
    }
}
