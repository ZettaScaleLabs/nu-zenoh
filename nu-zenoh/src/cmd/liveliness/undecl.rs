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
    PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
    engine::{Call, Command, EngineState, Stack},
};
use zenoh::Wait;

use crate::{State, cmd::liveliness::LivelinessTokenValue, signature_ext::SignatureExt};

#[derive(Clone)]
pub(crate) struct Undecl {
    _state: State,
}

impl Undecl {
    pub(crate) fn new(state: State) -> Self {
        Self { _state: state }
    }
}

impl Command for Undecl {
    fn name(&self) -> &str {
        "zenoh liveliness undecl"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::list(Type::record()))
            .required("token", SyntaxShape::Any, "liveliness token")
            .allowed_origin()
    }

    fn description(&self) -> &str {
        "Zenoh liveliness token undeclaration"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let token_value = call.req::<Value>(engine_state, stack, 0)?;
        let mut custom_value = token_value.into_custom_value()?;

        let mut token_lock = custom_value
            .as_mut_any()
            .downcast_mut::<LivelinessTokenValue>()
            .unwrap()
            .handle
            .lock()
            .unwrap();

        if let Some(token) = token_lock.take() {
            token.undeclare().wait().unwrap();
        }

        Ok(PipelineData::Empty)
    }
}
