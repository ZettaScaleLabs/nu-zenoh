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
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature,
};
use zenoh::Wait;

use crate::{call_ext2::CallExt2, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Delete {
    state: State,
}

impl Delete {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Delete {
    fn name(&self) -> &str {
        "zenoh delete"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(Command::name(self))
            .session()
            .zenoh_category()
            .publication()
    }

    fn description(&self) -> &str {
        "Zenoh DELETE"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        self.state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let mut delete = sess.delete(call.req::<String>(engine_state, stack, 0)?);

                if let Some(priority) = call.priority(engine_state, stack)? {
                    delete = delete.priority(priority);
                }

                if let Some(congestion_control) = call.congestion_control(engine_state, stack)? {
                    delete = delete.congestion_control(congestion_control);
                }

                if let Some(reliability) = call.reliable(engine_state, stack)? {
                    delete = delete.reliability(reliability);
                }

                if let Some(express) = call.express(engine_state, stack)? {
                    delete = delete.express(express);
                }

                if let Some(attachment) = call.attachment(engine_state, stack)? {
                    delete = delete.attachment(attachment.as_bytes());
                }

                if let Some(timestamp) = call.timestamp(engine_state, stack)? {
                    delete = delete.timestamp(timestamp);
                }

                if let Some(destination) = call.allowed_destination(engine_state, stack)? {
                    delete = delete.allowed_destination(destination);
                }

                delete.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Delete operation failed")
                    .with_label(format!("Zenoh delete failed: {e}"), call.head)
            })?;
        Ok(nu_protocol::PipelineData::empty())
    }
}
