use anyhow::Error;
use std::net::{SocketAddr};
use serde::{Serialize, Deserialize};
use difference::{Difference, Changeset};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Outputs {
    pub channels: usize,
    pub destinations: Vec<Destination>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inputs {
    pub channels: usize,
    pub listeners: Vec<Listener>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TLS {
    pub cacert: Option<String>,
    pub cert: String,
    pub key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Listener {
    pub addr: String,
    pub tls: Option<TLS>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Destination {
    pub name: String,
    pub addr: String,
    pub channels: Option<Vec<usize>>,
    pub tls: Option<TLS>,
}

/// Defines a virtual audio device which can later be
/// used by the CLI to stream audio. 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Device {
    pub name: String,
    pub inputs: Inputs,
    pub outputs: Outputs,
}

impl Device {
    pub fn sort(&mut self) {
        self.inputs.listeners.sort_by(|a, b| a.addr.partial_cmp(&b.addr).unwrap());
        self.outputs.destinations.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Upstream {
    pub name: String,
    pub addr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub upstream: Option<Vec<Upstream>>,
    pub devices: Vec<Device>,
}

impl Config {
    /// Creates a config from yaml and sorts the items
    /// so they are in deterministic order.
    pub fn from_yaml(doc: &str) -> Result<Self, serde_yaml::Error> {
        let mut config: Config = serde_yaml::from_str(doc)?;
        config.sort();
        Ok(config)
    }

    pub fn sort(&mut self) {
        if let Some(upstream) = &mut self.upstream {
            upstream.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        }
        self.devices.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        self.devices.iter_mut().for_each(|d| d.sort());
    }
}


fn reconcile(a: &Config, b: &Config) -> Result<Vec<Difference>, Error>{
    let a = serde_yaml::to_string(a)?;
    let b = serde_yaml::to_string(b)?;
    let Changeset { mut diffs, .. } = Changeset::new(&a, &b, "\n");
    diffs.retain(|d| match d {
        Difference::Same(_) => false,
        _ => true,
    });
    Ok(diffs)
}

#[cfg(test)]
mod test {
    use super::*;

    const CONFIG: &'static str = include_str!("../../../config.yaml");

    #[test]
    fn test_load_raw_yaml() {
        let config: Config = serde_yaml::from_str(CONFIG).unwrap();

        assert!(config.upstream.is_some());
    }

    #[test]
    fn test_from_yaml() {
        let config = Config::from_yaml(CONFIG).unwrap();
        assert!(config.upstream.is_some());
    }

    #[test]
    fn test_basic_diff() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].outputs.channels = 4;
        let diffs = reconcile(&current, &desired).unwrap();
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0], Difference::Rem(String::from("      channels: 2")));
        assert_eq!(diffs[1], Difference::Add(String::from("      channels: 4")));
    }

    #[test]
    fn foo() {
        /*
        let a = serde_yaml::to_string(&Device{
            name: String::from("foo"),
            inputs: Inputs{
                channels: 2,
                listeners: vec![],
            },
            outputs: Outputs{
                channels: 2,
                destinations: vec![],
            },
        }).unwrap();
        let b = serde_yaml::to_string(&Device{
            name: String::from("bar"),
            inputs: Inputs{
                channels: 2,
                listeners: vec![],
            },
            outputs: Outputs{
                channels: 2,
                destinations: vec![],
            },
        }).unwrap();

        // Compare both texts, the third parameter defines the split level.
        let Changeset { diffs, .. } = Changeset::new(&a, &b, "\n");

        let mut t = term::stdout().unwrap();

        for i in 0..diffs.len() {
            match diffs[i] {
                Difference::Same(ref x) => {
                    t.reset().unwrap();
                    writeln!(t, " {}", x);
                }
                Difference::Add(ref x) => {
                    t.fg(term::color::GREEN).unwrap();
                    writeln!(t, "+{}", x);
                }
                Difference::Rem(ref x) => {
                    t.fg(term::color::RED).unwrap();
                    writeln!(t, "-{}", x);
                }
            }
        }
        t.reset().unwrap();
        t.flush().unwrap();
        */
    }
}