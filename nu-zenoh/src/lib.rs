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
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nu_protocol::{
    engine::{EngineState, StateWorkingSet},
    LabeledError,
};
use zenoh::{internal::runtime::Runtime, Session, Wait};

mod call_ext2;
mod cmd;
mod conv;
mod interruptible_channel;
mod signature_ext;

#[derive(Debug, Clone)]
pub struct Config {
    pub experimental_options: bool,
    pub no_default_session: bool,
}

/// Adds extra context (e.g. aliases) as Nu source code
///
/// This should be called after [`crate::add_zenoh_context`].
pub const ZENOH_CONTEXT_EXTRAS: &[u8] = include_bytes!("nu/extras.nu");

/// Adds all `zenoh *` commands to the given [`nu_protocol::engine::EngineState`].
pub fn add_zenoh_context(mut engine_state: EngineState, options: Config) -> EngineState {
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);

        let state = State::new(options.clone());

        if options.experimental_options {
            working_set.add_decl(Box::new(cmd::runtime::list::List::new(state.clone())));
            working_set.add_decl(Box::new(cmd::runtime::open::Open::new(state.clone())));
            working_set.add_decl(Box::new(cmd::runtime::close::Close::new(state.clone())));

            working_set.add_decl(Box::new(cmd::pub_::Pub::new(state.clone())));
            working_set.add_decl(Box::new(cmd::querier::Querier::new(state.clone())));

            working_set.add_decl(Box::new(cmd::liveliness::declare_token::DeclareToken::new(
                state.clone(),
            )));
            working_set.add_decl(Box::new(
                cmd::liveliness::undeclare_token::UndeclareToken::new(state.clone()),
            ));
            working_set.add_decl(Box::new(cmd::liveliness::sub::Sub::new(state.clone())));

            working_set.add_decl(Box::new(cmd::pub_::MatchingStatus::new(state.clone())));
            working_set.add_decl(Box::new(cmd::querier::MatchingStatus::new(state.clone())));
        }

        working_set.add_decl(Box::new(cmd::put::Put::new(state.clone())));
        working_set.add_decl(Box::new(cmd::delete::Delete::new(state.clone())));
        working_set.add_decl(Box::new(cmd::get::Get::new(state.clone())));
        working_set.add_decl(Box::new(cmd::sub::Sub::new(state.clone())));
        working_set.add_decl(Box::new(cmd::zid::Zid::new(state.clone())));

        working_set.add_decl(Box::new(cmd::liveliness::get::Get::new(state.clone())));
        working_set.add_decl(Box::new(cmd::session::list::List::new(state.clone())));
        working_set.add_decl(Box::new(cmd::session::open::Open::new(state.clone())));
        working_set.add_decl(Box::new(cmd::session::close::Close::new(state.clone())));

        working_set.add_decl(Box::new(cmd::log_path::LogPath::new(state.clone())));
        working_set.add_decl(Box::new(cmd::queryable::Queryable::new(state.clone())));
        working_set.add_decl(Box::new(cmd::scout::Scout::new(state.clone())));
        working_set.add_decl(Box::new(cmd::info::Info::new(state.clone())));
        working_set.add_decl(Box::new(cmd::config::Config::new(state)));

        working_set.add_decl(Box::new(cmd::keyexpr::Includes));
        working_set.add_decl(Box::new(cmd::keyexpr::Intersects));

        working_set.render()
    };

    if let Err(err) = engine_state.merge_delta(delta) {
        eprintln!("Error creating Zenoh command context: {err:?}");
    }

    engine_state
}

#[derive(Clone)]
struct State {
    options: Config,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    runtimes: Arc<RwLock<HashMap<String, Runtime>>>,
}

impl State {
    const DEFAULT_SESSION_NAME: &str = "default";

    fn new(options: Config) -> Self {
        let mut sessions = HashMap::new();
        if !options.no_default_session {
            let default_session = zenoh::open(zenoh::Config::default())
                .wait()
                .expect("could not open default session");
            sessions.insert(Self::DEFAULT_SESSION_NAME.to_string(), default_session);
        }

        Self {
            options,
            sessions: Arc::new(RwLock::new(sessions)),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl State {
    pub(crate) fn with_session<F, T>(&self, name: &str, f: F) -> Result<T, LabeledError>
    where
        F: FnOnce(&Session) -> T,
    {
        let sessions = self.sessions.read().unwrap();
        let session = sessions
            .get(name)
            .ok_or_else(|| LabeledError::new(format!("session '{name}' not found")))?;
        Ok(f(session))
    }
}
