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
use nu_protocol::{Category, Signature, SyntaxShape};

pub(crate) trait SignatureExt: Sized {
    fn session(self) -> Self;

    fn zenoh_category(self) -> Self;

    fn publication(self) -> Self;

    fn allowed_destination(self) -> Self;

    fn allowed_origin(self) -> Self;

    fn encoding(self) -> Self;

    fn complete(self) -> Self;

    fn keyexpr(self) -> Self;
}

impl SignatureExt for Signature {
    fn session(self) -> Self {
        self.named(
            "session",
            SyntaxShape::String,
            "Session name (defauls to 'default')",
            Some('s'),
        )
    }

    fn zenoh_category(self) -> Self {
        const CATEGORY: &str = "Zenoh";
        self.category(Category::Custom(CATEGORY.to_string()))
    }

    fn allowed_destination(self) -> Self {
        self.named(
            "allowed-destination",
            SyntaxShape::String,
            "Allowed destination (either 'any', 'remote' or 'session-local')",
            None,
        )
    }

    fn allowed_origin(self) -> Self {
        self.named(
            "allowed-origin",
            SyntaxShape::String,
            "Allowed origin (either 'any', 'remote' or 'session-local')",
            None,
        )
    }

    fn publication(self) -> Self {
        self.named("priority", SyntaxShape::String, "Priority (0-7)", None)
            .named(
                "congestion-control",
                SyntaxShape::Int,
                "Congestion control (0 for DROP, 1 for BLOCK)",
                None,
            )
            .named(
                "reliable",
                SyntaxShape::Boolean,
                "Sets reliable transmission",
                None,
            )
            .named(
                "express",
                SyntaxShape::Boolean,
                "Sets express transmission",
                None,
            )
            .named("attachment", SyntaxShape::String, "Attachment data", None)
            .named(
                "timestamp",
                SyntaxShape::String,
                "Custom timestamp (expects the '<ZID>/<RFC3339>' format)",
                None,
            )
            .allowed_destination()
    }

    fn encoding(self) -> Self {
        self.named(
            "encoding",
            SyntaxShape::String,
            "Encoding (e.g., 'text/plain', 'application/json', etc)",
            None,
        )
    }

    fn complete(self) -> Self {
        self.named(
            "complete",
            SyntaxShape::Boolean,
            "Queryable completeness (complete if true and the given key expression includes the query key expression)",
            None,
        )
    }

    fn keyexpr(self) -> Self {
        self.required("keyexpr", SyntaxShape::String, "Key expression")
    }
}
