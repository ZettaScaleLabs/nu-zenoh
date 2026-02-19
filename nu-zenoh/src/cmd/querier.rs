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
use std::{mem, time::Duration};

use flume::RecvTimeoutError;
use nu_engine::CallExt;
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    record, IntoValue, ListStream, PipelineData, PipelineIterator, ShellError, Signals, Signature,
    Span, SyntaxShape, Type, Value,
};
use zenoh::Wait;

use crate::{call_ext2::CallExt2, conv, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Querier {
    state: State,
}

impl Querier {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Querier {
    fn name(&self) -> &str {
        "zenoh querier"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .keyexpr()
            .allowed_destination()
            .congestion_control()
            .express()
            .priority()
            .target()
            .consolidation()
            .named("timeout", SyntaxShape::Duration, "Query timeout", None)
            .input_output_type(Type::Any, Type::list(Type::list(Type::record())))
    }

    fn description(&self) -> &str {
        "Declare a querier"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;
        let key = call.req::<String>(engine_state, stack, 0)?;

        let querier = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let mut querier = sess.declare_querier(key);

                if let Some(priority) = call.priority(engine_state, stack)? {
                    querier = querier.priority(priority);
                }

                if let Some(congestion_control) = call.congestion_control(engine_state, stack)? {
                    querier = querier.congestion_control(congestion_control);
                }

                if let Some(express) = call.express(engine_state, stack)? {
                    querier = querier.express(express);
                }

                if let Some(destination) = call.allowed_destination(engine_state, stack)? {
                    querier = querier.allowed_destination(destination);
                }

                if let Some(timeout) = call.timeout(engine_state, stack)? {
                    // https://www.nushell.sh/lang-guide/chapters/types/basic_types/duration.html#additional-language-notes
                    querier = querier.timeout(timeout);
                }

                if let Some(consolidation) = call.consolidation(engine_state, stack)? {
                    querier = querier.consolidation(consolidation);
                }

                if let Some(target) = call.target(engine_state, stack)? {
                    querier = querier.target(target);
                }

                querier.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Declare querier operation failed")
                    .with_label(format!("Declare querier failed: {e}"), call.head)
            })?;

        struct Iter {
            input: PipelineIterator,
            querier: zenoh::query::Querier<'static>,
            receiver: Option<flume::Receiver<zenoh::query::Reply>>,
            signals: Signals,
            buffer: Vec<Value>,
            span: Span,
        }

        impl Iterator for Iter {
            type Item = Value;

            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if self.signals.interrupted() {
                        return None;
                    }

                    match self.receiver.as_ref().map(|r| {
                        r.recv_timeout(Duration::from_millis(50))
                            .map(|r| r.into_result())
                    }) {
                        None => {
                            let input = self.input.next()?;
                            let Ok(payload) = input.as_str() else {
                                continue;
                            };
                            const QUERY_CHANNEL_SIZE: usize = 256;
                            let receiver = self
                                .querier
                                .get()
                                .payload(payload)
                                .with(flume::bounded(QUERY_CHANNEL_SIZE))
                                .wait()
                                .unwrap();
                            self.receiver.replace(receiver);
                        }
                        Some(Ok(Ok(sample))) => self
                            .buffer
                            .push(conv::sample_to_record_value(sample, self.span)),
                        Some(Ok(Err(reply_err))) => self
                            .buffer
                            .push(conv::reply_error_to_error_value(reply_err, self.span)),
                        Some(Err(RecvTimeoutError::Timeout)) => continue,
                        Some(Err(RecvTimeoutError::Disconnected)) => {
                            let _ = self.receiver.take();
                            let values = mem::take(&mut self.buffer);
                            return Some(Value::list(values, self.span));
                        }
                    }
                }
            }
        }

        let signals = engine_state.signals().clone();

        Ok(ListStream::new(
            Iter {
                input: input.into_iter(),
                querier,
                receiver: None,
                signals: signals.clone(),
                buffer: Vec::default(),
                span,
            },
            span,
            signals.clone(),
        )
        .into())
    }
}

#[derive(Clone)]
pub(crate) struct MatchingStatus {
    state: State,
}

impl MatchingStatus {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for MatchingStatus {
    fn name(&self) -> &str {
        "zenoh querier matching-status"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .keyexpr()
            .allowed_destination()
            .target()
            .input_output_type(Type::Nothing, Type::record())
    }

    fn description(&self) -> &str {
        "Returns `true` if there are matching queryables"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let key = call.req::<String>(engine_state, stack, 0)?;

        let querier = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let mut querier = sess.declare_querier(key);

                if let Some(destination) = call.allowed_destination(engine_state, stack)? {
                    querier = querier.allowed_destination(destination);
                }

                if let Some(target) = call.target(engine_state, stack)? {
                    querier = querier.target(target);
                }

                querier.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Failed to declare querier")
                    .with_label(format!("Failed to declare querier: {e}"), call.head)
            })?;

        let status = querier.matching_status().wait().map_err(|e| {
            nu_protocol::LabeledError::new("Failed to get querier matching status").with_label(
                format!("Failed to get querier matching status: {e}"),
                call.head,
            )
        })?;

        Ok(nu_protocol::PipelineData::Value(
            record!(
                "matching" => status.matching().into_value(call.head),
            )
            .into_value(call.head),
            None,
        ))
    }
}
