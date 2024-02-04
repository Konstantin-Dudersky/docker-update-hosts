mod constants;
mod docker_host;
mod process_file;

use std::{collections::HashMap, fs};

use bollard::{service::ContainerInspectResponse, system::EventsOptions, Docker};
use futures::StreamExt;
use tokio::main;

use crate::{constants::HOSTS_FILE, docker_host::DockerHost, process_file::process_hosts_file};

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

const NW: &str = "ust-fermenter_network_internal";

/// Определяем имя хоста и ip запущенных контейнеров
async fn read_host_and_ip(docker: &Docker, network_name: &str) -> Vec<DockerHost> {
    // Определяем id запущенных контейнеров
    let container_ids = docker
        .list_containers::<String>(None)
        .await
        .unwrap()
        .into_iter()
        .filter_map(|c| c.id.clone())
        .collect::<Vec<String>>();

    let mut his = vec![];
    for id in container_ids {
        let container = docker.inspect_container(&id, None).await.unwrap();

        let hostname = extract_hostname(&container);
        let ip_address = extract_ip(&container, network_name);
        let hi = DockerHost::new(hostname, ip_address);
        let hi = match hi {
            Some(val) => val,
            None => continue,
        };
        his.push(hi);
    }
    his
}

async fn process(docker: &Docker) {
    let docker_hosts = read_host_and_ip(&docker, NW).await;

    let hosts_file = fs::read_to_string(HOSTS_FILE).expect("");
    let hosts_file = hosts_file.split("\n").collect::<Vec<&str>>();

    let new_file = process_hosts_file(&hosts_file, &docker_hosts);

    let new_file = new_file.join("\n");
    fs::write(HOSTS_FILE, new_file).unwrap();
    // println!("{}", new_file);
}

#[main]
async fn main() {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    // let version = docker.version().await.unwrap();
    // println!("{:?}", version);

    process(&docker).await;

    while let Some(_) = docker
        .events(Some(EventsOptions {
            since: None,
            until: None,
            filters: HashMap::from([
                ("event", vec!["start", "stop"]),
                ("type", vec!["container"]),
            ]),
        }))
        .next()
        .await
    {
        process(&docker).await;
    }
}
