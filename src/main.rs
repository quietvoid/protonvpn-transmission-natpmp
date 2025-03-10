use std::{fmt::Arguments, time::Duration};

use anyhow::{Result, bail};
use backon::ConstantBuilder;
use backon::Retryable;
use config::Config;
use fern::FormatCallback;
use futures::FutureExt;
use log::Record;
use transmission::set_transmission_port;

mod config;
mod signal_handling;
mod transmission;

const LOG_NAME: &str = "protonvpn-transmission-natpmp.log";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut exec_dir = std::env::current_exe()?;
    exec_dir.pop();

    let log_path = exec_dir.as_path().join(LOG_NAME);

    let local_offset = time::UtcOffset::current_local_offset()?;

    let default_formatter = move |out: FormatCallback, message: &Arguments, record: &Record| {
        out.finish(format_args!(
            "{} [{}] {}",
            time::OffsetDateTime::now_utc()
                .to_offset(local_offset)
                .format(&time::format_description::well_known::Iso8601::DATE_TIME)
                .unwrap(),
            record.level(),
            message
        ))
    };

    fern::Dispatch::new()
        .format(default_formatter)
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_path)?)
        .apply()?;

    let config = Config::new()?;
    let mut interval = tokio::time::interval(Duration::from_secs(50));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let res = (|| async { send_port_mapping_request(&config.gateway).await })
                    .retry(ConstantBuilder::default())
                    .await;

                match res {
                    Ok((udp_port, tcp_port)) => {
                        log::info!("Mapped public ports - UDP: {udp_port}, TCP: {tcp_port}");

                        if let Some(rpc_cfg) = config.transmission.as_ref() {
                            set_transmission_port(rpc_cfg, tcp_port).await
                                .inspect_err(|e| log::error!("Failed setting transmission port: {e}"))
                                .ok();
                        } else {
                            log::error!("Unconfigured torrent client config!");
                            break;
                        }
                    },
                    Err(e) => log::error!("Failed sending port mapping request: {e}")
                };
            }

            _ = signal_handling::wait_for_signal().fuse() => {
                log::info!("Shutting down daemon");
                break;
            }
        };
    }

    Ok(())
}

async fn send_port_mapping_request(gateway: &str) -> Result<(u16, u16)> {
    log::info!("Sending port mapping request");

    tokio::time::timeout(Duration::from_secs(10), async {
        let client = natpmp::new_tokio_natpmp_with(gateway.parse()?).await?;

        log::debug!("Sending UDP port mapping request");
        client
            .send_port_mapping_request(natpmp::Protocol::UDP, 0, 1, 60)
            .await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        log::debug!("Reading UDP response");
        let udp_res = client.read_response_or_retry().await?;
        let udp_public_port = if let natpmp::Response::UDP(e) = udp_res {
            e.public_port()
        } else {
            bail!("Unexpected UDP response");
        };

        // Wait a bit between requests
        tokio::time::sleep(Duration::from_millis(500)).await;

        log::debug!("Sending TCP port mapping request");
        client
            .send_port_mapping_request(natpmp::Protocol::TCP, 0, 1, 60)
            .await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        log::debug!("Reading TCP response");
        let tcp_res = client.read_response_or_retry().await?;
        let tcp_public_port = if let natpmp::Response::TCP(e) = tcp_res {
            e.public_port()
        } else {
            bail!("Unexpected TCP response");
        };

        Ok((udp_public_port, tcp_public_port))
    })
    .await?
}
