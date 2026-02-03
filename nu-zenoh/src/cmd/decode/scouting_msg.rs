//
// Copyright (c) 2026 ZettaScale Technology
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
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    record, IntoValue, PipelineData, ShellError, Signature, Span, Type, Value,
};
use zenoh_codec::{RCodec, Zenoh080};
use zenoh_protocol::scouting::{HelloProto, Scout, ScoutingBody, ScoutingMessage};

use crate::signature_ext::SignatureExt;

#[derive(Clone)]
pub(crate) struct ScoutingMsg;

impl Command for ScoutingMsg {
    fn name(&self) -> &str {
        "zenoh decode scouting-msg"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Binary, Type::record())
            .zenoh_category()
    }

    fn description(&self) -> &str {
        "Decode a Zenoh scouting message from binary data"
    }

    fn run(
        &self,
        _engine_state: &EngineState,
        _stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let span = call.head;

        let bytes = match input {
            PipelineData::Value(Value::Binary { val, .. }, ..) => val,
            _ => {
                return Err(ShellError::GenericError {
                    error: "Expected binary input".to_string(),
                    msg: "Input must be binary data".to_string(),
                    span: Some(span),
                    help: Some("Pipe binary data to this command".to_string()),
                    inner: vec![],
                });
            }
        };

        let codec = Zenoh080::new();
        let mut reader = bytes.as_slice();

        let msg: ScoutingMessage = codec.read(&mut reader).map_err(|_| {
            nu_protocol::LabeledError::new("Failed to decode scouting message")
                .with_label("Zenoh080 Codec error", span)
        })?;

        let value = scouting_message_to_value(&msg, span);

        Ok(PipelineData::Value(value, None))
    }
}

fn scouting_message_to_value(msg: &ScoutingMessage, span: Span) -> Value {
    match &msg.body {
        ScoutingBody::Scout(scout) => scout_to_value(scout, span),
        ScoutingBody::Hello(hello) => hello_to_value(hello, span),
    }
}

fn scout_to_value(scout: &Scout, span: Span) -> Value {
    record!(
        "type" => "scout".into_value(span),
        "version" => (scout.version as i64).into_value(span),
        "what" => scout.what.to_string().into_value(span),
        "zid" => scout.zid.map(|z| z.to_string()).into_value(span),
    )
    .into_value(span)
}

fn hello_to_value(hello: &HelloProto, span: Span) -> Value {
    record!(
        "type" => "hello".into_value(span),
        "version" => (hello.version as i64).into_value(span),
        "whatami" => hello.whatami.to_string().into_value(span),
        "zid" => hello.zid.to_string().into_value(span),
        "locators" => hello.locators.iter().map(|l| l.to_string().into_value(span)).collect::<Vec<_>>().into_value(span),
    )
    .into_value(span)
}
