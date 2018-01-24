use std::collections::HashSet;
use std::error::Error;
use std::net::SocketAddr;
use std::thread;

use curl::easy::Easy;
use serde_json;
use exonum::crypto::PublicKey;

use dmbc::config;

// TODO: duplicates structure in service-discovery crate. Generalize this.
#[derive(Debug, Hash, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct ValidatorInfo {
    pub public: SocketAddr,
    pub private: SocketAddr,
    pub peer: SocketAddr,
    pub consensus: PublicKey,
    pub service: PublicKey,
}

pub fn connect_validator(info: &ValidatorInfo) -> Result<HashSet<ValidatorInfo>, Box<Error>> {
    let discovery = config::config().service_discovery().address();

    let nodes = receive_nodes(&discovery)?;
    if nodes.contains(info) {
        return Ok(nodes);
    }

    send_node(&discovery, info)?;

    Ok(nodes)
}

fn receive_nodes(discovery: &str) -> Result<HashSet<ValidatorInfo>, Box<Error>> {
    let mut nodes_get = Vec::new();

    let mut handle = Easy::new();
    handle.url(discovery).map_err(Box::new)?;
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|data| {
                nodes_get.extend_from_slice(data);
                Ok(data.len())
            })
            .map_err(Box::new)?;
        transfer.perform().map_err(Box::new)?;
    }

    let nodes = serde_json::from_slice(&nodes_get);
    nodes.map_err(|e| e.into())
}

fn send_node(discovery: &str, info: &ValidatorInfo) -> Result<(), Box<Error>> {
    let node_post = serde_json::to_string(info).map_err(Box::new)?;

    let mut handle = Easy::new();
    handle.url(discovery).map_err(Box::new)?;
    handle.post(true).map_err(Box::new)?;
    handle
        .post_fields_copy(node_post.as_bytes())
        .map_err(Box::new)?;
    thread::spawn(move || match handle.perform() {
        Err(e) => eprintln!("Error in send_node(): {}", e),
        _ => (),
    });

    Ok(())
}
