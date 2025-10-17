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
        "zenoh get"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::list(Type::record()))
            .required("keyexpr", SyntaxShape::String, "Key expression")
            .named(
                "target",
                SyntaxShape::String,
                "Query target (either 'all', 'all-complete' or 'best-matching')",
                None,
            )
            .named(
                "consolidation",
                SyntaxShape::String,
                "Consolidation mode (either 'auto', 'latest', 'monotonic' or 'none')",
                None,
            )
            .named("timeout", SyntaxShape::Duration, "Query timeout", None)
            .named("payload", SyntaxShape::String, "Query payload", None)
            .named("encoding", SyntaxShape::String, "Query encoding", None)
            .named("attachment", SyntaxShape::String, "Query attachment", None)
            .allowed_destination()
    }

    fn description(&self) -> &str {
        "Zenoh GET"
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
                    .get(call.req::<String>(engine_state, stack, 0)?)
                    .callback(move |reply| {
                        let _ = tx.send(reply);
                    });

                if let Some(target) = call.target(engine_state, stack)? {
                    get = get.target(target);
                }

                if let Some(consolidation) = call.consolidation(engine_state, stack)? {
                    get = get.consolidation(consolidation);
                }

                if let Some(timeout) = call.timeout(engine_state, stack)? {
                    // https://www.nushell.sh/lang-guide/chapters/types/basic_types/duration.html#additional-language-notes
                    get = get.timeout(timeout);
                }

                if let Some(value) = call.get_flag::<String>(engine_state, stack, "payload")? {
                    get = get.payload(value);
                }

                if let Some(encoding) = call.encoding(engine_state, stack)? {
                    get = get.encoding(encoding);
                }

                if let Some(attachment) = call.attachment(engine_state, stack)? {
                    get = get.attachment(attachment.as_bytes());
                }

                if let Some(destination) = call.allowed_destination(engine_state, stack)? {
                    get = get.allowed_destination(destination);
                }

                get.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Get operation failed")
                    .with_label(format!("Zenoh get failed: {e}"), call.head)
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
