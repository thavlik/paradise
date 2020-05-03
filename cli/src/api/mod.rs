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
        self.outputs.destinations.sort_by(|a, b| a.addr.partial_cmp(&b.addr).unwrap());
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
    let mut diffs: Vec<Difference> = vec![];
    a.devices.iter()
        .for_each(|ad| {
            let current_device = &serde_yaml::to_string(ad).unwrap()[4..];
            let bd = match b.devices.iter()
                .find(|bd| bd.name == ad.name) {
                Some(bd) => bd,
                None => {
                    // The old device is not present in the new config.
                    // The entire config. All lines in the config belonging
                    // to the device should be removed.
                    diffs.extend(current_device.split("\n").map(|line| Difference::Rem(line.to_string())));
                    return;
                },
            };
            // Both ad and bd are present in the config
            let desired_device = &serde_yaml::to_string(bd).unwrap()[4..];
            let mut changes = Changeset::new(current_device, desired_device, "\n");
            diffs.append(&mut changes.diffs);
        });
    b.devices.iter()
        .filter(|bd| {
            a.devices.iter()
                .find(|ad| {
                    ad.name == bd.name
                }).is_none()
        })
        .map(|d| serde_yaml::to_string(d).unwrap())
        .for_each(|yaml| {
            let lines = yaml[4..].split("\n");
            lines.for_each(|line| {
                diffs.push(Difference::Add(String::from(line)));
            });
        });
    diffs.iter_mut().for_each(|diff| match diff {
        Difference::Same(diff) => *diff = prefix_lines("  ", diff),
        Difference::Add(diff) => *diff = prefix_lines("  ", diff),
        Difference::Rem(diff) => *diff = prefix_lines("  ", diff),
    });
    Ok(diffs)
}

fn prefix_lines(prefix: &str, lines: &str) -> String {
    let result = lines.split("\n")
        .map(|line| format!("{}{}\n", prefix, line))
        .fold(String::new(), |p, c| format!("{}{}", p, c));
    String::from(&result[..result.len()-2])
}

fn print_diff(line_prefix: &str, lines: &str, color: term::color::Color) {
    let mut t = term::stdout().unwrap();
    t.fg(color).unwrap();
    writeln!(t, "{}", prefix_lines(line_prefix, lines));
    t.reset().unwrap();
    t.flush().unwrap();
}

fn print_diffs(diffs: &Vec<Difference>) {
    //let mut t = term::stdout().unwrap();
    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => {
                //t.reset().unwrap();
                //writeln!(t, " {}", x);
                print_diff("'", x, term::color::WHITE);
            }
            Difference::Add(ref x) => {
                //t.fg(term::color::GREEN).unwrap();
                //writeln!(t, "+{}", x);
                print_diff("+", x, term::color::GREEN);
            }
            Difference::Rem(ref x) => {
                //t.fg(term::color::RED).unwrap();
                //writeln!(t, "-{}", x);
                print_diff("-", x, term::color::RED);
            }
        }
    }
    //t.reset().unwrap();
    //t.flush().unwrap();
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
    fn test_mutate_device_name() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let current_device: String = serde_yaml::to_string(&current.devices[0]).unwrap();
        let device_lines = serde_yaml::to_string(&current.devices[0]).unwrap().split("\n").collect::<Vec<_>>().len() - 1;
        let mut desired = current.clone();
        desired.devices[0].name = String::from("New Virtual Device");
        let desired_device: String = serde_yaml::to_string(&desired.devices[0]).unwrap();
        let diffs = reconcile(&current, &desired).unwrap();
        assert_eq!(diffs.len(), device_lines * 2);
    }

    #[test]
    fn test_mutate_input_channels() {
    }

    #[test]
    fn test_mutate_listener_addr() {
    }

    #[test]
    fn test_add_listener() {
    }

    #[test]
    fn test_remove_listener() {
    }

    #[test]
    fn test_mutate_output_channels() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].outputs.channels = 4;
        let diffs = reconcile(&current, &desired).unwrap();
        print_diffs(&diffs);
        assert_eq!(diffs.len(), 4);
        assert_eq!(diffs[1], Difference::Rem(String::from("    channels: 2")));
        assert_eq!(diffs[2], Difference::Add(String::from("    channels: 4")));
    }

    #[test]
    fn test_mutate_destination_addr() {
    }

    #[test]
    fn test_add_destination() {
    }

    #[test]
    fn test_remove_destination() {
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