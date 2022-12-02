use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use reqwest::header;
use serde::Deserialize;
use tar::Archive;

#[derive(Debug)]
pub enum DockerError {
    RequestFailed,
    Parse,
    Http(u16),
    Extract,
    InvalidImage,
    ArchitectureNotFound,
    Unpack,
}

#[derive(Deserialize)]
struct AuthData {
    token: String,
    access_token: String,
    expires_in: i32,
    issued_at: String,
}

#[derive(Deserialize)]
struct Platform {
    architecture: String,
    os: String,
}

#[derive(Deserialize)]
struct Manifest {
    digest: String,
    mediaType: String,
    platform: Platform,
    size: i32,
}

#[derive(Deserialize)]
struct ManifestsData {
    manifests: Vec<Manifest>,
}

#[derive(Deserialize)]
struct Config {
    mediaType: String,
    size: i32,
    digest: String,
}

#[derive(Deserialize)]
struct Layer {
    mediaType: String,
    size: i32,
    digest: String,
}

#[derive(Deserialize)]
struct ManifestLayers {
    schemaVersion: i32,
    mediaType: String,
    config: Config,
    layers: Vec<Layer>,
}

type Result<T> = std::result::Result<T, DockerError>;

fn get_auth_token(image_name: &str) -> Result<String> {
    let auth_url = format!(
        "https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}:pull",
        image_name
    );

    let auth_req = match reqwest::blocking::get(auth_url) {
        Ok(r) => r,
        Err(e) => {
            if let Some(status) = e.status() {
                return Err(DockerError::Http(status.as_u16()));
            }

            return Err(DockerError::RequestFailed);
        }
    };

    match auth_req.json::<AuthData>() {
        Ok(data) => Ok(data.token),
        Err(_) => Err(DockerError::Parse),
    }
}

fn get_manifest(token: &str, image_name: &str, image_version: &str) -> Result<Manifest> {
    let manifest_url = format!(
        "https://registry-1.docker.io/v2/{}/manifests/{}",
        image_name, image_version
    );

    let manifest_req = match reqwest::blocking::Client::new()
        .get(manifest_url)
        .bearer_auth(token)
        .header(
            reqwest::header::ACCEPT,
            "application/vnd.docker.distribution.manifest.list.v2+json",
        )
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            if let Some(status) = e.status() {
                return Err(DockerError::Http(status.as_u16()));
            }

            return Err(DockerError::RequestFailed);
        }
    };

    let data: ManifestsData = match manifest_req.json::<ManifestsData>() {
        Ok(d) => d,
        Err(_) => return Err(DockerError::Parse),
    };

    for manifest in data.manifests {
        if manifest.platform.architecture.eq("amd64") {
            return Ok(manifest);
        }
    }

    Err(DockerError::ArchitectureNotFound)
}

fn get_layer(token: &str, image_name: &str, manifest: &Manifest) -> Result<Layer> {
    let layer_url = format!(
        "https://registry-1.docker.io/v2/{}/manifests/{}",
        image_name, manifest.digest
    );

    let layer_req = match reqwest::blocking::Client::new()
        .get(layer_url)
        .bearer_auth(token)
        .header(reqwest::header::ACCEPT, manifest.mediaType.as_str())
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            if let Some(status) = e.status() {
                return Err(DockerError::Http(status.as_u16()));
            }

            return Err(DockerError::RequestFailed);
        }
    };

    let manifest_layers = match layer_req.json::<ManifestLayers>() {
        Ok(m) => m,
        Err(_) => return Err(DockerError::Parse),
    };

    match manifest_layers.layers.into_iter().last() {
        Some(l) => Ok(l),
        None => Err(DockerError::Parse),
    }
}

fn download_layer(token: &str, image_name: &str, layer: &Layer, output: &Path) -> Result<()> {
    let layer_conf_url = format!(
        "https://registry-1.docker.io/v2/{}/blobs/{}",
        image_name, layer.digest
    );

    let layer_req = match reqwest::blocking::Client::new()
        .get(layer_conf_url)
        .bearer_auth(token)
        .send()
    {
        Ok(r) => r,
        Err(e) => {
            if let Some(status) = e.status() {
                return Err(DockerError::Http(status.as_u16()));
            }

            return Err(DockerError::RequestFailed);
        }
    };

    let bytes = match layer_req.bytes() {
        Ok(b) => b,
        Err(_) => return Err(DockerError::Parse),
    };

    let tar = GzDecoder::new(&bytes[..]);

    let mut archive = Archive::new(tar);

    match archive.unpack(output) {
        Ok(()) => Ok(()),
        Err(_) => Err(DockerError::Unpack),
    }
}

pub fn download(image: &str, output: &Path) -> Result<()> {
    let parts: Vec<&str> = image.split(":").collect();

    if parts.len() != 2 {
        return Err(DockerError::InvalidImage);
    }
    let (image_name, image_version) = (parts[0], parts[1]);

    let token = get_auth_token(image_name)?;

    let manifest = get_manifest(&token, image_name, image_version)?;

    let layer = get_layer(&token, image_name, &manifest)?;

    download_layer(&token, image_name, &layer, output)?;

    Ok(())
}
