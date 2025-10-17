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
use nu_engine::{CallExt, ClosureEval};
use nu_protocol::{
    engine::{Call, Closure, Command, EngineState, Stack},
    ListStream, PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
};
use zenoh::Wait;

use crate::{
    call_ext2::CallExt2, conv, interruptible_channel::InterruptibleChannel,
    signature_ext::SignatureExt, State,
};

#[derive(Clone)]
pub(crate) struct Queryable {
    state: State,
}

impl Queryable {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Queryable {
    fn name(&self) -> &str {
        "zenoh queryable"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::Nothing)
            .session()
            .zenoh_category()
            .keyexpr()
            .required(
                "hanlder",
                SyntaxShape::Closure(Some(vec![SyntaxShape::Any])),
                "handler",
            )
            .complete()
            .allowed_origin()
    }

    fn description(&self) -> &str {
        "Zenoh Queryable declaration"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;

        const REPLY_CHANNEL_SIZE: usize = 256;
        let (tx, rx) = flume::bounded(REPLY_CHANNEL_SIZE);

        let handler = call.req::<Closure>(engine_state, stack, 1)?;
        let mut closure = ClosureEval::new(engine_state, stack, handler);
        let signals = engine_state.signals().clone();
        let engine = engine_state.clone();

        let queryable = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let mut queryable = sess
                    .declare_queryable(call.req::<String>(engine_state, stack, 0)?)
                    .callback(move |query| {
                        let _ = tx.send(query);
                    });

                if let Some(origin) = call.allowed_origin(engine_state, stack)? {
                    queryable = queryable.allowed_origin(origin);
                }

                if let Some(complete) = call.complete(engine_state, stack)? {
                    queryable = queryable.complete(complete);
                }

                queryable.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Queryable declaration failed")
                    .with_label(format!("Zenoh queryable failed: {e}"), span)
            })?;

        let iter = InterruptibleChannel::with_data(rx, engine.signals().clone(), queryable).map(
            move |query| {
                let value = conv::query_to_record_value(&query, span);

                match closure.run_with_value(value) {
                    Ok(stream) => {
                        for value in stream {
                            let bytes = match conv::value_to_bytes(&value) {
                                Ok(bytes) => bytes,
                                Err(err) => return Value::error(err, span),
                            };

                            if let Err(err) = query.reply(query.key_expr(), bytes).wait() {
                                return Value::error(ShellError::from(err), span);
                            }
                        }

                        Value::nothing(span)
                    }
                    Err(err) => match query.reply_err(err.to_string()).wait() {
                        Ok(()) => Value::nothing(span),
                        Err(err) => Value::error(ShellError::from(err), span),
                    },
                }
            },
        );

        Ok(ListStream::new(iter, span, signals).into())
    }
}
