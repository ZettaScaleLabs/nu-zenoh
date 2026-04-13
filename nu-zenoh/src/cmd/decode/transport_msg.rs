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
use std::convert::TryFrom;

use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    record,
    shell_error::generic::GenericError,
    IntoValue, PipelineData, ShellError, Signature, Span, Type, Value,
};
use zenoh_codec::{RCodec, Zenoh080};
use zenoh_protocol::{
    common::ZExtBody,
    core::{Field, Locator, Resolution, WhatAmI, WireExpr},
    network::{declare::DeclareBody, interest::InterestMode, oam::id::OAM_LINKSTATE, NetworkBody},
    transport::{
        close::reason_to_str, Fragment, Frame, InitAck, InitSyn, Join, OpenAck, OpenSyn,
        TransportBody, TransportMessage,
    },
};

use crate::signature_ext::SignatureExt;

#[derive(Clone)]
pub(crate) struct TransportMsg;

impl Command for TransportMsg {
    fn name(&self) -> &str {
        "zenoh decode transport-msg"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Binary, Type::record())
            .zenoh_category()
    }

    fn description(&self) -> &str {
        "Decode a Zenoh transport message from binary data"
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
                return Err(ShellError::Generic(
                    GenericError::new("Expected binary input", "Input must be binary data", span)
                        .with_help("Pipe binary data to this command"),
                ));
            }
        };

        let codec = Zenoh080::new();
        let mut reader = bytes.as_slice();

        let msg: TransportMessage = codec.read(&mut reader).map_err(|_| {
            nu_protocol::LabeledError::new("Failed to decode transport message")
                .with_label("Zenoh080 Codec error", span)
        })?;

        Ok(PipelineData::Value(
            transport_message_to_value(&msg, span),
            None,
        ))
    }
}

fn transport_message_to_value(msg: &TransportMessage, span: Span) -> Value {
    match &msg.body {
        TransportBody::InitSyn(m) => init_syn_to_value(m, span),
        TransportBody::InitAck(m) => init_ack_to_value(m, span),
        TransportBody::OpenSyn(m) => open_syn_to_value(m, span),
        TransportBody::OpenAck(m) => open_ack_to_value(m, span),
        TransportBody::Close(m) => record!(
            "type"    => "close".into_value(span),
            "reason"  => reason_to_str(m.reason).into_value(span),
            "session" => m.session.into_value(span),
        )
        .into_value(span),
        TransportBody::KeepAlive(_) => keep_alive_to_value(span),
        TransportBody::Frame(m) => frame_to_value(m, span),
        TransportBody::Fragment(m) => fragment_to_value(m, span),
        TransportBody::OAM(m) => transport_oam_to_value(m, span),
        TransportBody::Join(m) => join_to_value(m, span),
    }
}

fn init_syn_to_value(m: &InitSyn, span: Span) -> Value {
    record!(
        "type"        => "init-syn".into_value(span),
        "version"     => (m.version as i64).into_value(span),
        "whatami"     => m.whatami.to_string().into_value(span),
        "zid"         => m.zid.to_string().into_value(span),
        "resolution"  => resolution_to_value(&m.resolution, span),
        "batch_size"  => (m.batch_size as i64).into_value(span),
        "ext_qos"     => m.ext_qos.is_some().into_value(span),
        "ext_lowlatency" => m.ext_lowlatency.is_some().into_value(span),
        "ext_compression" => m.ext_compression.is_some().into_value(span),
        "ext_patch"   => (m.ext_patch.raw() as i64).into_value(span),
    )
    .into_value(span)
}

fn init_ack_to_value(m: &InitAck, span: Span) -> Value {
    let cookie_hex: String = m
        .cookie
        .as_slice()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    record!(
        "type"        => "init-ack".into_value(span),
        "version"     => (m.version as i64).into_value(span),
        "whatami"     => m.whatami.to_string().into_value(span),
        "zid"         => m.zid.to_string().into_value(span),
        "resolution"  => resolution_to_value(&m.resolution, span),
        "batch_size"  => (m.batch_size as i64).into_value(span),
        "cookie"      => cookie_hex.into_value(span),
        "ext_qos"     => m.ext_qos.is_some().into_value(span),
        "ext_lowlatency" => m.ext_lowlatency.is_some().into_value(span),
        "ext_compression" => m.ext_compression.is_some().into_value(span),
        "ext_patch"   => (m.ext_patch.raw() as i64).into_value(span),
    )
    .into_value(span)
}

fn open_syn_to_value(m: &OpenSyn, span: Span) -> Value {
    let cookie_hex: String = m
        .cookie
        .as_slice()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    record!(
        "type"       => "open-syn".into_value(span),
        "lease_ms"   => (m.lease.as_millis() as i64).into_value(span),
        "initial_sn" => (m.initial_sn as i64).into_value(span),
        "cookie"     => cookie_hex.into_value(span),
        "ext_qos"    => m.ext_qos.is_some().into_value(span),
        "ext_lowlatency"  => m.ext_lowlatency.is_some().into_value(span),
        "ext_compression" => m.ext_compression.is_some().into_value(span),
    )
    .into_value(span)
}

fn open_ack_to_value(m: &OpenAck, span: Span) -> Value {
    record!(
        "type"       => "open-ack".into_value(span),
        "lease_ms"   => (m.lease.as_millis() as i64).into_value(span),
        "initial_sn" => (m.initial_sn as i64).into_value(span),
        "ext_qos"    => m.ext_qos.is_some().into_value(span),
        "ext_lowlatency"  => m.ext_lowlatency.is_some().into_value(span),
        "ext_compression" => m.ext_compression.is_some().into_value(span),
    )
    .into_value(span)
}

fn keep_alive_to_value(span: Span) -> Value {
    record!("type" => "keep-alive".into_value(span)).into_value(span)
}

fn frame_to_value(m: &Frame, span: Span) -> Value {
    let messages: Vec<Value> = m
        .payload
        .iter()
        .map(|nm| network_message_to_value(nm, span))
        .collect();
    record!(
        "type"        => "frame".into_value(span),
        "reliability" => m.reliability.to_string().to_lowercase().into_value(span),
        "sn"          => (m.sn as i64).into_value(span),
        "messages"    => messages.into_value(span),
    )
    .into_value(span)
}

fn fragment_to_value(m: &Fragment, span: Span) -> Value {
    let payload: String = m
        .payload
        .as_slice()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    record!(
        "type"        => "fragment".into_value(span),
        "reliability" => m.reliability.to_string().to_lowercase().into_value(span),
        "sn"          => (m.sn as i64).into_value(span),
        "more"        => m.more.into_value(span),
        "first"       => m.ext_first.is_some().into_value(span),
        "drop"        => m.ext_drop.is_some().into_value(span),
        "payload" => payload.into_value(span),
    )
    .into_value(span)
}

fn transport_oam_to_value(oam: &zenoh_protocol::transport::Oam, span: Span) -> Value {
    let mut rec = record!(
        "type" => "oam".into_value(span),
        "id"   => (oam.id as i64).into_value(span),
    );

    match &oam.body {
        ZExtBody::Unit => {
            rec.push("ext_body", "unit".into_value(span));
        }
        ZExtBody::Z64(v) => {
            rec.push("ext_body", "z64".into_value(span));
            rec.push("value", (*v as i64).into_value(span));
        }
        ZExtBody::ZBuf(zbuf) => {
            rec.push("ext_body", "zbuf".into_value(span));
            let hex: String = zbuf
                .zslices()
                .flat_map(|s| s.as_slice().iter())
                .map(|b| format!("{b:02x}"))
                .collect();
            rec.push("payload", hex.into_value(span));
        }
    }

    rec.into_value(span)
}

fn join_to_value(m: &Join, span: Span) -> Value {
    record!(
        "type"        => "join".into_value(span),
        "version"     => (m.version as i64).into_value(span),
        "whatami"     => m.whatami.to_string().into_value(span),
        "zid"         => m.zid.to_string().into_value(span),
        "resolution"  => resolution_to_value(&m.resolution, span),
        "batch_size"  => (m.batch_size as i64).into_value(span),
        "lease_ms"    => (m.lease.as_millis() as i64).into_value(span),
        "next_sn" => record!(
            "reliable"    => (m.next_sn.reliable as i64).into_value(span),
            "best_effort" => (m.next_sn.best_effort as i64).into_value(span),
        ).into_value(span),
        "ext_qos"     => m.ext_qos.is_some().into_value(span),
        "ext_patch"   => (m.ext_patch.raw() as i64).into_value(span),
    )
    .into_value(span)
}

fn network_message_to_value(msg: &zenoh_protocol::network::NetworkMessage, span: Span) -> Value {
    let reliability = msg.reliability.to_string().to_lowercase();
    match &msg.body {
        NetworkBody::Push(m) => push_to_value(m, reliability, span),
        NetworkBody::Request(m) => request_to_value(m, reliability, span),
        NetworkBody::Response(m) => response_to_value(m, reliability, span),
        NetworkBody::ResponseFinal(m) => response_final_to_value(m, reliability, span),
        NetworkBody::Interest(m) => interest_to_value(m, reliability, span),
        NetworkBody::Declare(m) => declare_to_value(m, reliability, span),
        NetworkBody::OAM(m) => network_oam_to_value(m, reliability, span),
    }
}

fn push_to_value(push: &zenoh_protocol::network::Push, reliability: String, span: Span) -> Value {
    record!(
        "type"         => "push".into_value(span),
        "reliability"  => reliability.into_value(span),
        "wire_expr"    => wire_expr_to_value(&push.wire_expr, span),
        "payload_size" => (push.payload_size() as i64).into_value(span),
    )
    .into_value(span)
}

fn request_to_value(
    request: &zenoh_protocol::network::Request,
    reliability: String,
    span: Span,
) -> Value {
    record!(
        "type"         => "request".into_value(span),
        "reliability"  => reliability.into_value(span),
        "id"           => (request.id as i64).into_value(span),
        "wire_expr"    => wire_expr_to_value(&request.wire_expr, span),
        "payload_size" => (request.payload_size() as i64).into_value(span),
    )
    .into_value(span)
}

fn response_to_value(
    response: &zenoh_protocol::network::Response,
    reliability: String,
    span: Span,
) -> Value {
    record!(
        "type"         => "response".into_value(span),
        "reliability"  => reliability.into_value(span),
        "rid"          => (response.rid as i64).into_value(span),
        "wire_expr"    => wire_expr_to_value(&response.wire_expr, span),
        "payload_size" => (response.payload_size() as i64).into_value(span),
    )
    .into_value(span)
}

fn response_final_to_value(
    rf: &zenoh_protocol::network::ResponseFinal,
    reliability: String,
    span: Span,
) -> Value {
    record!(
        "type"        => "response-final".into_value(span),
        "reliability" => reliability.into_value(span),
        "rid"         => (rf.rid as i64).into_value(span),
    )
    .into_value(span)
}

fn interest_to_value(
    interest: &zenoh_protocol::network::Interest,
    reliability: String,
    span: Span,
) -> Value {
    let mode = match interest.mode {
        InterestMode::Final => "final",
        InterestMode::Current => "current",
        InterestMode::Future => "future",
        InterestMode::CurrentFuture => "current-future",
    };
    let wire_expr = interest
        .wire_expr
        .as_ref()
        .map(|we| wire_expr_to_value(we, span))
        .into_value(span);
    let options = record!(
        "key_exprs"   => interest.options.keyexprs().into_value(span),
        "subscribers" => interest.options.subscribers().into_value(span),
        "queryables"  => interest.options.queryables().into_value(span),
        "tokens"      => interest.options.tokens().into_value(span),
        "restricted"  => interest.options.restricted().into_value(span),
        "aggregate"   => interest.options.aggregate().into_value(span),
    )
    .into_value(span);
    record!(
        "type"        => "interest".into_value(span),
        "reliability" => reliability.into_value(span),
        "id"          => (interest.id as i64).into_value(span),
        "mode"        => mode.into_value(span),
        "options"     => options,
        "wire_expr"   => wire_expr,
    )
    .into_value(span)
}

fn declare_to_value(
    declare: &zenoh_protocol::network::Declare,
    reliability: String,
    span: Span,
) -> Value {
    record!(
        "type"        => "declare".into_value(span),
        "reliability" => reliability.into_value(span),
        "interest_id" => declare.interest_id.map(|id| id as i64).into_value(span),
        "body"        => declare_body_to_value(&declare.body, span),
    )
    .into_value(span)
}

fn declare_body_to_value(body: &DeclareBody, span: Span) -> Value {
    match body {
        DeclareBody::DeclareKeyExpr(d) => record!(
            "type"      => "declare-key-expr".into_value(span),
            "id"        => (d.id as i64).into_value(span),
            "wire_expr" => wire_expr_to_value(&d.wire_expr, span),
        )
        .into_value(span),
        DeclareBody::UndeclareKeyExpr(u) => record!(
            "type" => "undeclare-key-expr".into_value(span),
            "id"   => (u.id as i64).into_value(span),
        )
        .into_value(span),
        DeclareBody::DeclareSubscriber(d) => record!(
            "type"      => "declare-subscriber".into_value(span),
            "id"        => (d.id as i64).into_value(span),
            "wire_expr" => wire_expr_to_value(&d.wire_expr, span),
        )
        .into_value(span),
        DeclareBody::UndeclareSubscriber(u) => record!(
            "type" => "undeclare-subscriber".into_value(span),
            "id"   => (u.id as i64).into_value(span),
        )
        .into_value(span),
        DeclareBody::DeclareQueryable(d) => record!(
            "type"      => "declare-queryable".into_value(span),
            "id"        => (d.id as i64).into_value(span),
            "wire_expr" => wire_expr_to_value(&d.wire_expr, span),
            "complete"  => d.ext_info.complete.into_value(span),
            "distance"  => (d.ext_info.distance as i64).into_value(span),
        )
        .into_value(span),
        DeclareBody::UndeclareQueryable(u) => record!(
            "type" => "undeclare-queryable".into_value(span),
            "id"   => (u.id as i64).into_value(span),
        )
        .into_value(span),
        DeclareBody::DeclareToken(d) => record!(
            "type"      => "declare-token".into_value(span),
            "id"        => (d.id as i64).into_value(span),
            "wire_expr" => wire_expr_to_value(&d.wire_expr, span),
        )
        .into_value(span),
        DeclareBody::UndeclareToken(u) => record!(
            "type" => "undeclare-token".into_value(span),
            "id"   => (u.id as i64).into_value(span),
        )
        .into_value(span),
        DeclareBody::DeclareFinal(_) => {
            record!("type" => "declare-final".into_value(span)).into_value(span)
        }
    }
}

fn network_oam_to_value(
    oam: &zenoh_protocol::network::oam::Oam,
    reliability: String,
    span: Span,
) -> Value {
    let mut rec = record!(
        "type"          => "oam".into_value(span),
        "reliability"   => reliability.into_value(span),
        "id"            => (oam.id as i64).into_value(span),
    );

    match &oam.body {
        ZExtBody::Unit => {
            rec.push("ext_body", "unit".into_value(span));
        }
        ZExtBody::Z64(v) => {
            rec.push("ext_body", "z64".into_value(span));
            rec.push("value", (*v as i64).into_value(span));
        }
        ZExtBody::ZBuf(zbuf) => {
            rec.push("ext_body", "zbuf".into_value(span));
            if oam.id == OAM_LINKSTATE {
                let zslice = zbuf.to_zslice();
                let link_states = decode_link_state_list(zslice.as_slice(), span);
                rec.push("linkstate", link_states);
            } else {
                let hex: String = zbuf
                    .zslices()
                    .flat_map(|s| s.as_slice().iter())
                    .map(|b| format!("{b:02x}"))
                    .collect();
                rec.push("payload", hex.into_value(span));
            }
        }
    }

    rec.into_value(span)
}

/// Decode a `LinkStateList` from raw OAM payload bytes using `Zenoh080`.
///
/// The wire format mirrors zenoh's internal `Zenoh080Routing` codec.
fn decode_link_state_list(bytes: &[u8], span: Span) -> Value {
    let mut reader: &[u8] = bytes;
    let codec = Zenoh080::new();

    let len: usize = match RCodec::<usize, _>::read(codec, &mut reader) {
        Ok(l) => l,
        Err(_) => {
            return Value::error(
                ShellError::Generic(GenericError::new(
                    "LinkStateList decode error",
                    "could not read list length",
                    span,
                )),
                span,
            );
        }
    };

    let mut states = Vec::with_capacity(len);
    for _ in 0..len {
        match decode_link_state(&mut reader, codec, span) {
            Ok(v) => states.push(v),
            Err(e) => return e,
        }
    }

    states.into_value(span)
}

fn decode_link_state(reader: &mut &[u8], codec: Zenoh080, span: Span) -> Result<Value, Value> {
    const LS_PID: u64 = 1; // zid is present
    const LS_WAI: u64 = 1 << 1; // whatami is present
    const LS_LOC: u64 = 1 << 2; // locators are present
    const LS_WGT: u64 = 1 << 3; // link weights are present

    macro_rules! read_field {
        ($ty:ty, $label:literal) => {
            RCodec::<$ty, _>::read(codec, reader).map_err(|_| {
                Value::error(
                    ShellError::Generic(GenericError::new(
                        "LinkState decode error",
                        concat!("could not read field: ", $label),
                        span,
                    )),
                    span,
                )
            })
        };
    }

    let options: u64 = read_field!(u64, "options")?;
    let psid: u64 = read_field!(u64, "psid")?;
    let sn: u64 = read_field!(u64, "sn")?;

    let zid: Option<String> = if options & LS_PID != 0 {
        let z: zenoh_protocol::core::ZenohIdProto =
            read_field!(zenoh_protocol::core::ZenohIdProto, "zid")?;
        Some(z.to_string())
    } else {
        None
    };

    let whatami: Option<String> = if options & LS_WAI != 0 {
        let wai: u8 = read_field!(u8, "whatami")?;
        WhatAmI::try_from(wai).ok().map(|w| w.to_string())
    } else {
        None
    };

    let locators: Option<Vec<Value>> = if options & LS_LOC != 0 {
        let locs: Vec<Locator> = read_field!(Vec<Locator>, "locators")?;
        Some(
            locs.iter()
                .map(|l| l.to_string().into_value(span))
                .collect(),
        )
    } else {
        None
    };

    let links_len: usize = read_field!(usize, "links_len")?;
    let mut links: Vec<Value> = Vec::with_capacity(links_len);
    for _ in 0..links_len {
        let l: u64 = read_field!(u64, "link")?;
        links.push((l as i64).into_value(span));
    }

    let link_weights: Option<Vec<Value>> = if options & LS_WGT != 0 {
        let mut weights = Vec::with_capacity(links_len);
        for _ in 0..links_len {
            let w: u16 = read_field!(u16, "link_weight")?;
            weights.push((w as i64).into_value(span));
        }
        Some(weights)
    } else {
        None
    };

    Ok(record!(
        "psid"         => (psid as i64).into_value(span),
        "sn"           => (sn as i64).into_value(span),
        "zid"          => zid.into_value(span),
        "whatami"      => whatami.into_value(span),
        "locators"     => locators.into_value(span),
        "links"        => links.into_value(span),
        "link_weights" => link_weights.into_value(span),
    )
    .into_value(span))
}

fn resolution_to_value(resolution: &Resolution, span: Span) -> Value {
    record!(
        "frame_sn"   => resolution.get(Field::FrameSN).to_string().into_value(span),
        "request_id" => resolution.get(Field::RequestID).to_string().into_value(span),
    )
    .into_value(span)
}

fn wire_expr_to_value(wire_expr: &WireExpr<'_>, span: Span) -> Value {
    wire_expr.to_string().into_value(span)
}
