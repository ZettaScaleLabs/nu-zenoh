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
use nu_engine::CallExt;
use nu_protocol::{
    PipelineData, ShellError, Signature, SyntaxShape,
    engine::{Call, Command, EngineState, Stack},
};
use zenoh::Wait;

use crate::{State, call_ext2::CallExt2, signature_ext::SignatureExt};

#[derive(Clone)]
pub(crate) struct Put {
    state: State,
}

impl Put {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Put {
    fn name(&self) -> &str {
        "zenoh put"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .publication()
            .encoding()
            .required(
                "payload",
                SyntaxShape::OneOf(vec![SyntaxShape::String, SyntaxShape::Binary]),
                "Publication payload",
            )
    }

    fn description(&self) -> &str {
        "Zenoh PUT"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let key = call.req::<String>(engine_state, stack, 0)?;
        let value = call.req::<Vec<u8>>(engine_state, stack, 1)?;

        self.state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let mut put = sess.put(key, value);

                if let Some(encoding) = call.encoding(engine_state, stack)? {
                    put = put.encoding(encoding);
                }

                if let Some(priority) = call.priority(engine_state, stack)? {
                    put = put.priority(priority);
                }

                if let Some(congestion_control) = call.congestion_control(engine_state, stack)? {
                    put = put.congestion_control(congestion_control);
                }

                if let Some(reliability) = call.reliable(engine_state, stack)? {
                    put = put.reliability(reliability);
                }

                if let Some(express) = call.express(engine_state, stack)? {
                    put = put.express(express);
                }

                if let Some(attachment) = call.attachment(engine_state, stack)? {
                    put = put.attachment(attachment.as_bytes());
                }

                if let Some(timestamp) = call.timestamp(engine_state, stack)? {
                    put = put.timestamp(timestamp);
                }

                if let Some(destination) = call.allowed_destination(engine_state, stack)? {
                    put = put.allowed_destination(destination);
                }

                put.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Put operation failed")
                    .with_label(format!("Zenoh put failed: {e}"), call.head)
            })?;

        Ok(nu_protocol::PipelineData::empty())
    }
}
