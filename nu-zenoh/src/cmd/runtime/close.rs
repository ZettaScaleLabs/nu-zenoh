use nu_engine::CallExt;
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
};
use zenoh::Wait;

use crate::{signature_ext::SignatureExt, State};

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
        "zenoh runtime close"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("runtime", SyntaxShape::Filepath, "Runtime name")
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::Nothing)
    }

    fn description(&self) -> &str {
        "Close a runtime"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let runtime_name = call.req::<String>(engine_state, stack, 0)?;
        let mut runtimes = self.state.runtimes.write().unwrap();
        if let Some(runtime) = runtimes.remove(&runtime_name) {
            runtime.close().wait().map_err(|e| {
                nu_protocol::LabeledError::new("Failed to close Zenoh session '{session_name}'")
                    .with_label(format!("Could not close Zenoh session: {e}"), call.head)
            })?
        }

        Ok(PipelineData::Value(Value::nothing(call.head), None))
    }
}
