use anyhow::{Error, Result, anyhow, Context};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::PathBuf;
use std::process::Command;
use quinn::{
    ServerConfig,
    ServerConfigBuilder,
    TransportConfig,
    CertificateChain,
    PrivateKey,
    Certificate,
};
use std::{
    io,
    net::SocketAddr,
    sync::{Arc, mpsc},
    fs,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Endpoint {
    pub name: String,

    pub addr: String,

    pub insecure: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceSpec {
    pub name: String,

    #[serde(rename = "displayName")]
    pub display_name: String,

    pub inputs: u16,

    pub outputs: u16,

    pub endpoints: Vec<Endpoint>,
}

impl DeviceSpec {
    pub fn get_handle(&self) -> Result<cpal::Device> {
        let available_hosts = cpal::available_hosts();
        for host_id in available_hosts {
            let host = cpal::host_from_id(host_id)?;
            for (_, d) in host.devices()?.enumerate() {
                if let Ok(name) = d.name() {
                    if name == self.display_name {
                        return Ok(d);
                    }
                }
            }
        }
        Err(anyhow!("device '{}' not found", &self.name))
    }

    pub fn verify(&self) -> Result<()> {
        let available_hosts = cpal::available_hosts();
        for host_id in available_hosts {
            let host = cpal::host_from_id(host_id)?;
            for (_, d) in host.devices()?.enumerate() {
                if let Ok(name) = d.name() {
                    if name == self.display_name {
                        match (self.inputs > 0, d.default_input_config()) {
                            (true, Ok(conf)) => {
                                if conf.channels() != self.inputs {
                                    return Err(Error::msg(format!(
                                        "mismatch number of input channels (got {}, expected {})",
                                        conf.channels(),
                                        self.inputs
                                    )));
                                }
                            }
                            (true, Err(e)) => {
                                return Err(Error::msg("device is missing input config"))
                            }
                            (false, Ok(_)) => {
                                return Err(Error::msg("device has unexpected input config"))
                            }
                            (false, Err(e)) => match e {
                                cpal::DefaultStreamConfigError::StreamTypeNotSupported => {}
                                _ => return Err(e.into()),
                            },
                        }
                        match (self.outputs > 0, d.default_output_config()) {
                            (true, Ok(conf)) => {
                                if conf.channels() != self.outputs {
                                    return Err(Error::msg(format!(
                                        "mismatch number of output channels (got {}, expected {})",
                                        conf.channels(),
                                        self.outputs
                                    )));
                                }
                            }
                            (true, Err(e)) => {
                                return Err(Error::msg("device is missing output config"))
                            }
                            (false, Ok(_)) => {
                                return Err(Error::msg("device has unexpected output config"))
                            }
                            (false, Err(e)) => match e {
                                cpal::DefaultStreamConfigError::StreamTypeNotSupported => {}
                                _ => return Err(e.into()),
                            },
                        }
                        return Ok(());
                    }
                }
            }
        }
        return Err(Error::msg(format!(
            "device '{}' not loaded by CoreAudio",
            &self.name
        )));
    }
}
