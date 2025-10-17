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
use zenoh::Wait;

use crate::{call_ext2::CallExt2, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Info {
    state: State,
}

impl Info {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Info {
    fn name(&self) -> &str {
        "zenoh info"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::record())
    }

    fn description(&self) -> &str {
        "Session information"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;

        let info = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| sess.info())?;

        Ok(PipelineData::Value(
            Value::record(
                record!(
                    "zid" => info.zid().wait().to_string().into_value(span),
                    "routers_zid" => info
                        .routers_zid()
                        .wait()
                        .map(|zid| zid.to_string().into_value(span))
                        .collect::<Vec<_>>()
                        .into_value(span),
                    "peers_zid" => info
                        .peers_zid()
                        .wait()
                        .map(|zid| zid.to_string().into_value(span))
                        .collect::<Vec<_>>()
                        .into_value(span),
                ),
                span,
            ),
            None,
        ))
    }
}
