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
use std::sync::{Arc, Mutex};

use nu_protocol::{CustomValue, ShellError, Span, Value, record};
use serde::Serialize;
use zenoh::{key_expr::OwnedKeyExpr, liveliness::LivelinessToken};

pub(crate) mod decl;
pub(crate) mod get;
pub(crate) mod undecl;

#[derive(Debug, Clone)]
struct LivelinessTokenValue {
    handle: Arc<Mutex<Option<LivelinessToken>>>,
    keyexpr: OwnedKeyExpr,
}

impl CustomValue for LivelinessTokenValue {
    fn clone_value(&self, span: Span) -> Value {
        Value::custom(Box::new(self.clone()), span)
    }

    fn type_name(&self) -> String {
        self.keyexpr.to_string()
    }

    fn to_base_value(&self, span: Span) -> Result<Value, ShellError> {
        Ok(Value::record(
            record! {
                "keyexpr" => Value::string(self.keyexpr.to_string(), span)
            },
            span,
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    #[doc(hidden)]
    fn typetag_name(&self) -> &'static str {
        "LivelinessTokenValue"
    }

    #[doc(hidden)]
    fn typetag_deserialize(&self) {
        unimplemented!()
    }
}

impl Serialize for LivelinessTokenValue {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unimplemented!()
    }
}
