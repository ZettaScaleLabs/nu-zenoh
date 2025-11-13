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
    PipelineData, ShellError, Signature, Type,
};
use zenoh::Wait;

use crate::{call_ext2::CallExt2, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Pub {
    state: State,
}

impl Pub {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Pub {
    fn name(&self) -> &str {
        "zenoh pub"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .keyexpr()
            .allowed_destination()
            .congestion_control()
            .reliable()
            .express()
            .priority()
            .encoding()
            .input_output_type(Type::Any, Type::Any)
    }

    fn description(&self) -> &str {
        "Zenoh Publisher"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let key = call.req::<String>(engine_state, stack, 0)?;

        let pub_ = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let mut pub_ = sess.declare_publisher(key);

                if let Some(encoding) = call.encoding(engine_state, stack)? {
                    pub_ = pub_.encoding(encoding);
                }

                if let Some(priority) = call.priority(engine_state, stack)? {
                    pub_ = pub_.priority(priority);
                }

                if let Some(congestion_control) = call.congestion_control(engine_state, stack)? {
                    pub_ = pub_.congestion_control(congestion_control);
                }

                if let Some(reliability) = call.reliable(engine_state, stack)? {
                    pub_ = pub_.reliability(reliability);
                }

                if let Some(express) = call.express(engine_state, stack)? {
                    pub_ = pub_.express(express);
                }

                if let Some(destination) = call.allowed_destination(engine_state, stack)? {
                    pub_ = pub_.allowed_destination(destination);
                }

                pub_.wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Declare publisher operation failed")
                    .with_label(format!("Declare publisher failed: {e}"), call.head)
            })?;

        for value in input {
            pub_.put(value.as_str()?).wait().unwrap();
        }

        Ok(nu_protocol::PipelineData::empty())
    }
}
