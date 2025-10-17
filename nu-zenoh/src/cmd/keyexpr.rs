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
use std::str::FromStr;

use nu_engine::CallExt;
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature, SyntaxShape, Value,
};
use zenoh::key_expr::KeyExpr;

#[derive(Clone)]
pub(crate) struct Includes;

impl Command for Includes {
    fn name(&self) -> &str {
        "zenoh keyexpr includes"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("lhs", SyntaxShape::String, "Left-hand side key-expreesion.")
            .required(
                "rhs",
                SyntaxShape::String,
                "Right-hand side key-expreesion.",
            )
    }

    fn description(&self) -> &str {
        "Returns true if the lhs keyexpr includes the rhs keyexpr"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let lhs =
            KeyExpr::from_str(&call.req::<String>(engine_state, stack, 0)?).map_err(|err| {
                nu_protocol::LabeledError::new("Invalid left-hand side key-expression")
                    .with_label(err.to_string(), call.arguments_span())
            })?;
        let rhs =
            KeyExpr::from_str(&call.req::<String>(engine_state, stack, 1)?).map_err(|err| {
                nu_protocol::LabeledError::new("Invalid right-hand side key-expression")
                    .with_label(err.to_string(), call.arguments_span())
            })?;

        Ok(PipelineData::Value(
            Value::bool(lhs.includes(&rhs), call.head),
            None,
        ))
    }
}

#[derive(Clone)]
pub(crate) struct Intersects;

impl Command for Intersects {
    fn name(&self) -> &str {
        "zenoh keyexpr intersects"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("lhs", SyntaxShape::String, "Left-hand side key-expreesion.")
            .required(
                "rhs",
                SyntaxShape::String,
                "Right-hand side key-expreesion.",
            )
    }

    fn description(&self) -> &str {
        "Returns true if the lhs and ths keyexprs intersect"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let lhs =
            KeyExpr::from_str(&call.req::<String>(engine_state, stack, 0)?).map_err(|err| {
                nu_protocol::LabeledError::new("Invalid left-hand side key-expression")
                    .with_label(err.to_string(), call.arguments_span())
            })?;
        let rhs =
            KeyExpr::from_str(&call.req::<String>(engine_state, stack, 1)?).map_err(|err| {
                nu_protocol::LabeledError::new("Invalid right-hand side key-expression")
                    .with_label(err.to_string(), call.arguments_span())
            })?;

        Ok(PipelineData::Value(
            Value::bool(lhs.intersects(&rhs), call.head),
            None,
        ))
    }
}
