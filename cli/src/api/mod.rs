use std::net::{SocketAddr};
use serde::{Serialize, Deserialize};

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
pub enum Protocol {
    TCP,
    UDP
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
    pub fn from_yaml(doc: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(doc)
    }
}

fn reconcile(current: &Device, desired: &Device) -> Result<(), ()> {
    Err(())
}

#[cfg(test)]
mod test {
    use super::*;
    use difference::{Difference, Changeset};

    #[test]
    fn test_load_config() {
        let test = include_str!("../../../config.yaml");
        let config: Config = serde_yaml::from_str(test).unwrap();
        assert!(config.upstream.is_some());
    }

    #[test]
    fn test_diff() {
        let test = include_str!("../../../config.yaml");
        let config: Config = serde_yaml::from_str(test).unwrap();
        assert!(config.upstream.is_some());
    }

    #[test]
    fn foo() {
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
    }
}