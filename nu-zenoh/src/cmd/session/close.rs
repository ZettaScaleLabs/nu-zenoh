use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature, Type, Value,
};
use zenoh::Wait;

use crate::{call_ext2::CallExt2, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Close {
    state: State,
}

impl Close {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Close {
    fn name(&self) -> &str {
        "zenoh session close"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::Nothing)
    }

    fn description(&self) -> &str {
        "Close a session"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let session_name = call.session(engine_state, stack)?;
        let mut sessions = self.state.sessions.write().unwrap();
        if let Some(sess) = sessions.remove(&session_name) {
            sess.close().wait().map_err(|e| {
                nu_protocol::LabeledError::new("Failed to close Zenoh session '{session_name}'")
                    .with_label(format!("Could not close Zenoh session: {e}"), call.head)
            })?
        }

        Ok(PipelineData::Value(Value::nothing(call.head), None))
    }
}
