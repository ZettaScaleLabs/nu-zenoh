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
    IntoValue, PipelineData, ShellError, Signature, Type,
};

use crate::{call_ext2::CallExt2, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Zid {
    state: State,
}

impl Zid {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Zid {
    fn name(&self) -> &str {
        "zenoh zid"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .session()
            .switch("short", "Shorten the ZID to a prefix", None)
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::String)
    }

    fn description(&self) -> &str {
        "Session ZID"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call<'_>,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        pub fn short(mut zid: String) -> String {
            const MAX_ZID_LEN: usize = 8;
            zid.truncate(MAX_ZID_LEN);
            zid
        }

        let is_short = call.has_flag(engine_state, stack, "short")?;

        let data = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                let zid = sess.zid().to_string();
                let value = if is_short { short(zid) } else { zid };
                PipelineData::Value(value.into_value(call.head), None)
            })?;

        Ok(data)
    }
}
