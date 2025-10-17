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
use nu_protocol::{engine, ListStream, PipelineData, ShellError, Signature, SyntaxShape, Type};
use zenoh::Wait;

use crate::{
    call_ext2::CallExt2, conv, interruptible_channel::InterruptibleChannel,
    signature_ext::SignatureExt, State,
};

#[derive(Clone)]
pub(crate) struct Get {
    state: State,
}

impl Get {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl engine::Command for Get {
    fn name(&self) -> &str {
        "zenoh liveliness get"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::list(Type::record()))
            .required("keyexpr", SyntaxShape::String, "Key expression")
    }

    fn description(&self) -> &str {
        "Zenoh liveliness GET"
    }

    fn run(
        &self,
        engine_state: &engine::EngineState,
        stack: &mut engine::Stack,
        call: &engine::Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;

        const REPLY_CHANNEL_SIZE: usize = 256;
        let (tx, rx) = flume::bounded(REPLY_CHANNEL_SIZE);

        self.state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let mut get = sess
                    .liveliness()
                    .get(call.req::<String>(engine_state, stack, 0)?)
                    .callback(move |reply| {
                        let _ = tx.send(reply);
                    });

                // FIXME(liveliness): no timeout
                if let Some(timeout) = call.timeout(engine_state, stack)? {
                    // https://www.nushell.sh/lang-guide/chapters/types/basic_types/duration.html#additional-language-notes
                    get = get.timeout(timeout);
                }

                get.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Liveliness get operation failed")
                    .with_label(format!("Zenoh liveliness get failed: {e}"), call.head)
            })?;

        let iter =
            InterruptibleChannel::new(rx, engine_state.signals().clone()).map(move |reply| {
                match reply.into_result() {
                    Ok(sample) => conv::sample_to_record_value(sample, span),
                    Err(reply_error) => conv::reply_error_to_error_value(reply_error, span),
                }
            });

        Ok(ListStream::new(iter, call.head, engine_state.signals().clone()).into())
    }
}
