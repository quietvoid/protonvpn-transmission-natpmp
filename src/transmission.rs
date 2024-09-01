use anyhow::{anyhow, Result};
use transmission_rpc::{
    types::{BasicAuth, SessionSetArgs},
    TransClient,
};

use crate::config::TransmissionRpcConfig;

pub async fn set_transmission_port(rpc_cfg: &TransmissionRpcConfig, new_port: u16) -> Result<()> {
    let mut client = TransClient::with_auth(
        rpc_cfg.rpc_url.parse()?,
        BasicAuth {
            user: rpc_cfg.rpc_username.clone(),
            password: rpc_cfg.rpc_password.clone(),
        },
    );

    let current_port = client
        .session_get()
        .await
        .map(|session_info| session_info.arguments.peer_port as u16)
        .map_err(|e| anyhow!(e))?;

    if new_port == current_port {
        return Ok(());
    }

    let args = SessionSetArgs {
        peer_port: Some(new_port as i32),
        ..Default::default()
    };
    client.session_set(args).await.map_err(|e| anyhow!(e))?;

    log::info!("Successfully set transmission port from {current_port} to {new_port}");

    Ok(())
}
