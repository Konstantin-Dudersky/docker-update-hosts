mod constants;
mod docker_host;
mod error;
mod process_file;

use std::{collections::HashMap, env, fs};

use bollard::{service::ContainerInspectResponse, system::EventsOptions, Docker};
use futures::StreamExt;
use sudo::escalate_if_needed;
use tokio::main;
use tracing::{error, info};

use crate::{
    constants::HOSTS_FILE, docker_host::DockerHost, error::Error, process_file::process_hosts_file,
};

fn extract_hostname(container: &ContainerInspectResponse) -> Option<String> {
    container.config.clone()?.hostname
}

fn extract_ip(container: &ContainerInspectResponse, network_name: &str) -> Option<String> {
    container
        .network_settings
        .clone()?
        .networks?
        .get(network_name)?
        .ip_address
        .clone()
}

/// Определяем имя хоста и ip запущенных контейнеров
async fn read_docker_host_and_ip(
    docker: &Docker,
    network_name: &str,
) -> Result<Vec<DockerHost>, Error> {
    // Определяем id запущенных контейнеров
    let container_ids = docker
        .list_containers::<String>(None)
        .await?
        .into_iter()
        .filter_map(|c| c.id.clone())
        .collect::<Vec<String>>();

    let mut dhs = vec![];
    for id in container_ids {
        let container = docker.inspect_container(&id, None).await?;

        let hostname = extract_hostname(&container);
        let ip_address = extract_ip(&container, network_name);
        let hi = DockerHost::new(hostname, ip_address);
        let hi = match hi {
            Some(val) => val,
            None => continue,
        };
        dhs.push(hi);
    }
    Ok(dhs)
}

async fn read_network_names(docker: &Docker) -> Result<Vec<String>, Error> {
    let networks = docker
        .list_networks::<String>(None)
        .await?
        .into_iter()
        .filter_map(|n| n.name.clone())
        .collect::<Vec<String>>();
    Ok(networks)
}

async fn update_hosts_file(docker: &Docker, selected_network: &str) -> Result<(), Error> {
    let docker_hosts = read_docker_host_and_ip(&docker, selected_network).await?;

    let docker_hosts_info = docker_hosts
        .iter()
        .map(|dh| format!("{}", dh))
        .collect::<Vec<String>>()
        .join("\n");
    info!("Found docker hosts:\n{}", docker_hosts_info);

    let hosts_file = fs::read_to_string(HOSTS_FILE).expect("");
    let hosts_file = hosts_file.split("\n").collect::<Vec<&str>>();

    let new_file = process_hosts_file(&hosts_file, &docker_hosts);

    let new_file = new_file.join("\n");
    escalate_if_needed().map_err(|e| Error::Sudo(e))?;
    fs::write(HOSTS_FILE, new_file).map_err(Error::WriteFile)?;
    Ok(())
}

async fn main_() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let docker = Docker::connect_with_socket_defaults()?;

    let networks = read_network_names(&docker).await?;
    if args.len() <= 1 {
        return Err(Error::NetworkNotSelect(networks));
    }

    let selected_network = args[1].clone();
    if !networks.contains(&selected_network) {
        return Err(Error::NetworkInvalidChoice {
            selected_network,
            networks,
        });
    }

    update_hosts_file(&docker, &selected_network).await?;

    let event_subs_options = EventsOptions {
        since: None,
        until: None,
        filters: HashMap::from([
            ("event", vec!["start", "stop"]),
            ("type", vec!["container"]),
        ]),
    };

    while let Some(_) = docker.events(Some(event_subs_options.clone())).next().await {
        info!("Update by event");
        update_hosts_file(&docker, &selected_network).await?;
    }
    Ok(())
}

#[main]
async fn main() {
    tracing_subscriber::fmt().init();

    let result = main_().await;
    if let Err(err) = result {
        error!("End of program: {}", err);
    }
}
