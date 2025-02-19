use serde::{Deserialize, Serialize};

use crate::db::types::prompt_eval_run::PromptEvalRun;


#[derive(Debug, Serialize, Deserialize)]
pub struct PromptEvalRunResponse {

}

impl From<PromptEvalRun> for PromptEvalRunResponse {
    fn from(run: PromptEvalRun) -> Self {
        PromptEvalRunResponse {

        }
    }
}
