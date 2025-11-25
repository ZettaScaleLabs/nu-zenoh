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
    ListStream, PipelineData, ShellError, Signature, SyntaxShape, Type,
};
use zenoh::Wait;

use crate::{
    call_ext2::CallExt2, conv, interruptible_channel::InterruptibleChannel,
    signature_ext::SignatureExt, State,
};

#[derive(Clone)]
pub(crate) struct Sub {
    state: State,
}

impl Sub {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Sub {
    fn name(&self) -> &str {
        "zenoh liveliness sub"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::list(Type::record()))
            .required("keyexpr", SyntaxShape::String, "key-expression")
            .switch("history", "GET liveliness history", None)
            .allowed_origin()
    }

    fn description(&self) -> &str {
        "Declare a liveliness subscriber"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        const SUB_CHANNEL_SIZE: usize = 256;
        let (tx, rx) = flume::bounded(SUB_CHANNEL_SIZE);

        let span = call.head;

        let keyexpr = call.req::<String>(engine_state, stack, 0)?;
        let history = call.has_flag(engine_state, stack, "history")?;

        assert!(!history);

        let sub = self
            .state
            .with_session(&call.session(engine_state, stack)?, move |sess| {
                let sub = sess
                    .liveliness()
                    .declare_subscriber(keyexpr)
                    .history(history)
                    .callback(move |sample| {
                        let _ = tx.send(sample);
                    });

                sub.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Liveliness subscriber declaration failed")
                    .with_label(
                        format!("Zenoh liveliness subscriber failed: {e}"),
                        call.head,
                    )
            })?;

        let iter = InterruptibleChannel::with_data(rx, engine_state.signals().clone(), sub)
            .map(move |sample| conv::sample_to_record_value(sample, span));

        Ok(ListStream::new(iter, call.head, engine_state.signals().clone()).into())
    }
}
