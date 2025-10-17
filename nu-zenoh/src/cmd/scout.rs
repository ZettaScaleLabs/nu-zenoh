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

use std::time::Instant;

use nu_protocol::{
    IntoValue, ListStream, PipelineData, ShellError, Signature, Span, SyntaxShape, Type, Value,
    engine::{Call, Command, EngineState, Stack},
    record,
};
use zenoh::{Wait, config::WhatAmIMatcher, scouting::Hello};

use crate::{
    State, call_ext2::CallExt2, interruptible_channel::InterruptibleChannel,
    signature_ext::SignatureExt,
};

#[derive(Clone)]
pub(crate) struct Scout {
    state: State,
}

impl Scout {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Scout {
    fn name(&self) -> &str {
        "zenoh scout"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .named("timeout", SyntaxShape::Duration, "Scouting timeout", None)
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::record())
    }

    fn description(&self) -> &str {
        "Scout the Zenoh network"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;

        const SCOUT_CHANNEL_SIZE: usize = 256;
        let (tx, rx) = flume::bounded(SCOUT_CHANNEL_SIZE);

        let config = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                zenoh::Config::from_json5(&sess.config().to_json()).expect("Failed to copy Config from generared JSON!")
            })?;

        let scout = zenoh::scout(WhatAmIMatcher::empty().client().peer().router(), config)
            .callback(move |scout| {
                let _ = tx.send(scout);
            })
            .wait()
            .map_err(|e| {
                nu_protocol::LabeledError::new("Scout operation failed")
                    .with_label(format!("Zenoh scout failed: {e}"), call.head)
            })?;

        fn hello_to_value(hello: &Hello, span: Span) -> Value {
            record!(
                    "zid" => hello.zid().to_string().into_value(span),
                    "whatami" => hello.whatami().to_string().into_value(span),
                    "locators" => hello.locators().iter().map(|l| l.to_string().into_value(span)).collect::<Vec<_>>().into_value(span),
                )
                .into_value(span)
        }

        if let Some(timeout) = call.timeout(engine_state, stack)? {
            let deadline = Instant::now() + timeout;
            let mut values = Vec::new();

            while let Ok(hello) = rx.recv_deadline(deadline) {
                values.push(hello_to_value(&hello, span))
            }

            Ok(PipelineData::Value(Value::list(values, span), None))
        } else {
            let iter = InterruptibleChannel::with_data(rx, engine_state.signals().clone(), scout)
                .map(move |hello| hello_to_value(&hello, span));

            Ok(ListStream::new(iter, span, engine_state.signals().clone()).into())
        }
    }
}
