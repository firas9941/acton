use crate::modules::github::Github;
use anyhow::Result;

const WORKFLOW: &str = "update-abi-catalog.yml";

pub(crate) fn run() -> Result<()> {
    let workflow_url = Github::new().dispatch_workflow(WORKFLOW)?;

    println!("Triggered `{WORKFLOW}`.");
    println!("Workflow run: {workflow_url}");
    Ok(())
}
