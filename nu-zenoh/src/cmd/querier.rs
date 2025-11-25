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
    PipelineData, ShellError, Signature, SyntaxShape, Type,
};
use zenoh::Wait;

use crate::{call_ext2::CallExt2, signature_ext::SignatureExt, State};

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
            .input_output_type(Type::Any, Type::Any)
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

        for value in input {
            querier.get().payload(value.as_str()?).wait().unwrap();
        }

        Ok(nu_protocol::PipelineData::empty())
    }
}
