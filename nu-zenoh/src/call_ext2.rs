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
use std::time::Duration;

use nu_engine::CallExt;
use nu_protocol::{
    engine::{Call, EngineState, Stack},
    LabeledError, Value,
};
use zenoh::{
    bytes::Encoding,
    qos::{CongestionControl, Priority, Reliability},
    query::{ConsolidationMode, QueryTarget},
    sample::Locality,
    time::Timestamp,
};

use crate::State;

pub(crate) trait CallExt2 {
    fn allowed_origin(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Locality>, LabeledError>;

    fn allowed_destination(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Locality>, LabeledError>;

    fn priority(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Priority>, LabeledError>;

    fn reliable(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Reliability>, LabeledError>;

    fn consolidation(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<ConsolidationMode>, LabeledError>;

    fn target(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<QueryTarget>, LabeledError>;

    fn congestion_control(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<CongestionControl>, LabeledError>;

    fn timestamp(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Timestamp>, LabeledError>;

    fn session(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<String, LabeledError>;

    fn complete(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<bool>, LabeledError>;

    fn encoding(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Encoding>, LabeledError>;

    fn express(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<bool>, LabeledError>;

    fn attachment(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<String>, LabeledError>;

    fn timeout(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Duration>, LabeledError>;
}

impl CallExt2 for Call<'_> {
    fn allowed_origin(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Locality>, LabeledError> {
        self.get_flag::<String>(engine_state, stack, "allowed-origin")?
            .map(|o| parse_locality(&o, self.head))
            .transpose()
    }

    fn allowed_destination(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Locality>, LabeledError> {
        self.get_flag::<String>(engine_state, stack, "allowed-destination")?
            .map(|o| parse_locality(&o, self.head))
            .transpose()
    }

    fn priority(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Priority>, LabeledError> {
        /// Helper function to parse priority values
        fn parse_priority(
            priority_str: &str,
            span: nu_protocol::Span,
        ) -> Result<Priority, nu_protocol::LabeledError> {
            let priority_val = priority_str.parse::<u8>().map_err(|_| {
                nu_protocol::LabeledError::new("Invalid priority")
                    .with_label("Priority must be a number between 0-7", span)
            })?;
            Priority::try_from(priority_val).map_err(|_| {
                nu_protocol::LabeledError::new("Invalid priority")
                    .with_label("Priority must be between 0-7", span)
            })
        }

        self.get_flag::<String>(engine_state, stack, "priority")?
            .map(|p| parse_priority(&p, self.head))
            .transpose()
    }

    fn consolidation(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<ConsolidationMode>, LabeledError> {
        /// Helper function to parse consolidation mode values
        fn parse_consolidation(
            consolidation_str: &str,
            span: nu_protocol::Span,
        ) -> Result<ConsolidationMode, nu_protocol::LabeledError> {
            match consolidation_str {
                "auto" => Ok(ConsolidationMode::Auto),
                "latest" => Ok(ConsolidationMode::Latest),
                "monotonic" => Ok(ConsolidationMode::Monotonic),
                "none" => Ok(ConsolidationMode::None),
                _ => Err(nu_protocol::LabeledError::new("Invalid consolidation mode")
                    .with_label("Must be 'auto', 'latest', 'monotonic', or 'none'", span)),
            }
        }

        self.get_flag::<String>(engine_state, stack, "consolidation")?
            .map(|c| parse_consolidation(&c, self.head))
            .transpose()
    }

    fn target(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<QueryTarget>, LabeledError> {
        /// Helper function to parse target values
        fn parse_target(
            target_str: &str,
            span: nu_protocol::Span,
        ) -> Result<QueryTarget, nu_protocol::LabeledError> {
            match target_str {
                "all" => Ok(QueryTarget::All),
                "all-complete" => Ok(QueryTarget::AllComplete),
                "best-matching" => Ok(QueryTarget::BestMatching),
                _ => Err(nu_protocol::LabeledError::new("Invalid target")
                    .with_label("Must be 'all', 'all-complete', or 'best-matching'", span)),
            }
        }

        self.get_flag::<String>(engine_state, stack, "target")?
            .map(|c| parse_target(&c, self.head))
            .transpose()
    }

    fn congestion_control(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<CongestionControl>, LabeledError> {
        /// Helper function to parse congestion control values
        fn parse_congestion_control(
            cc_val: i64,
            span: nu_protocol::Span,
        ) -> Result<CongestionControl, nu_protocol::LabeledError> {
            match cc_val {
                0 => Ok(CongestionControl::Drop),
                1 => Ok(CongestionControl::Block),
                _ => Err(nu_protocol::LabeledError::new("Invalid congestion control")
                    .with_label("Must be 0 (drop) or 1 (block)", span)),
            }
        }

        self.get_flag::<i64>(engine_state, stack, "congestion-control")?
            .map(|c| parse_congestion_control(c, self.head))
            .transpose()
    }

    fn timestamp(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Timestamp>, LabeledError> {
        /// Helper function to parse timestamp values
        fn parse_timestamp(
            timestamp_str: &str,
            span: nu_protocol::Span,
        ) -> Result<Timestamp, nu_protocol::LabeledError> {
            Timestamp::parse_rfc3339(timestamp_str).map_err(|e| {
                nu_protocol::LabeledError::new("Invalid timestamp")
                    .with_label(format!("Failed to parse RFC3339 timestamp: {e:?}"), span)
            })
        }

        self.get_flag::<String>(engine_state, stack, "timestamp")?
            .map(|c| parse_timestamp(&c, self.head))
            .transpose()
    }

    fn session(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<String, LabeledError> {
        Ok(self
            .get_flag::<String>(engine_state, stack, "session")?
            .unwrap_or(State::DEFAULT_SESSION_NAME.to_string()))
    }

    fn complete(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<bool>, LabeledError> {
        self.get_flag::<bool>(engine_state, stack, "complete")
            .map_err(|err| LabeledError::from_diagnostic(&err))
    }

    fn reliable(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Reliability>, LabeledError> {
        self.get_flag::<bool>(engine_state, stack, "reliable")
            .map(|opt| opt.map(Reliability::from))
            .map_err(|err| LabeledError::from_diagnostic(&err))
    }

    fn encoding(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Encoding>, LabeledError> {
        Ok(self
            .get_flag::<String>(engine_state, stack, "encoding")?
            .map(Encoding::from))
    }

    fn express(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<bool>, LabeledError> {
        self.get_flag::<bool>(engine_state, stack, "express")
            .map_err(|err| LabeledError::from_diagnostic(&err))
    }

    fn attachment(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<String>, LabeledError> {
        self.get_flag::<String>(engine_state, stack, "attachment")
            .map_err(|err| LabeledError::from_diagnostic(&err))
    }

    fn timeout(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
    ) -> Result<Option<Duration>, LabeledError> {
        match self.get_flag::<Value>(engine_state, stack, "timeout")? {
            Some(v) => Ok(Some(Duration::from_nanos(v.as_duration()? as u64))),
            None => Ok(None),
        }
    }
}

/// Helper function to parse locality values
fn parse_locality(
    value: &str,
    span: nu_protocol::Span,
) -> Result<Locality, nu_protocol::LabeledError> {
    match value {
        "any" => Ok(Locality::Any),
        "remote" => Ok(Locality::Remote),
        "session-local" => Ok(Locality::SessionLocal),
        _ => Err(nu_protocol::LabeledError::new("Invalid locality")
            .with_label("Must be 'any', 'remote', or 'session-local'", span)),
    }
}
