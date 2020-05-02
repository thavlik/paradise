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
pub enum Addr {
    Upstream(String),
    IP((SocketAddr, Protocol)),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TLS {
    pub cacert: Option<String>,
    pub cert: String,
    pub key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Listener {
    pub addr: Addr,
    pub tls: Option<TLS>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Destination {
    pub name: String,
    pub addr: Addr,
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

fn reconcile(current: &Device, desired: &Device) -> Result<(), ()> {
    Err(())
}

#[cfg(test)]
mod test {
    use super::*;
    use difference::{Difference, Changeset};

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