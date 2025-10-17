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
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use nu_engine::CallExt;
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
};
use zenoh::{key_expr::OwnedKeyExpr, Wait};

use crate::{
    call_ext2::CallExt2, cmd::liveliness::LivelinessTokenValue, signature_ext::SignatureExt, State,
};

#[derive(Clone)]
pub(crate) struct Decl {
    state: State,
}

impl Decl {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Decl {
    fn name(&self) -> &str {
        "zenoh liveliness decl"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::list(Type::record()))
            .required("keyexpr", SyntaxShape::String, "key-expression")
            .allowed_origin()
    }

    fn description(&self) -> &str {
        "Zenoh liveliness token declaration"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let keyexpr = call.req::<String>(engine_state, stack, 0)?;

        let token = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                sess.liveliness().declare_token(&keyexpr).wait()
            })?
            .map_err(|e| {
                nu_protocol::LabeledError::new("Liveliness token declaration failed").with_label(
                    format!("Zenoh Liveliness token declaration failed: {e}"),
                    call.head,
                )
            })?;

        Ok(PipelineData::Value(
            Value::custom(
                Box::new(LivelinessTokenValue {
                    handle: Arc::new(Mutex::new(Some(token))),
                    keyexpr: OwnedKeyExpr::from_str(&keyexpr).unwrap(),
                }),
                call.head,
            ),
            None,
        ))
    }
}
