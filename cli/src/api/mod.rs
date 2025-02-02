
use difference::{Changeset, Difference};
use serde::{Deserialize, Serialize};


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
        self.inputs
            .listeners
            .sort_by(|a, b| a.addr.partial_cmp(&b.addr).unwrap());
        self.outputs
            .destinations
            .sort_by(|a, b| a.addr.partial_cmp(&b.addr).unwrap());
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
        self.devices
            .sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        self.devices.iter_mut().for_each(|d| d.sort());
    }

    fn reconcile(a: &Config, b: &Config) -> Vec<Difference> {
        let mut diffs: Vec<Difference> = vec![];
        a.devices.iter().for_each(|ad| {
            let current_device = &serde_yaml::to_string(ad).unwrap()[4..];
            let bd = match b.devices.iter().find(|bd| bd.name == ad.name) {
                Some(bd) => bd,
                None => {
                    // The old device is not present in the new config.
                    // The entire config. All lines in the config belonging
                    // to the device should be removed.
                    diffs.extend(
                        current_device
                            .split("\n")
                            .map(|line| Difference::Rem(line.to_string())),
                    );
                    return;
                }
            };
            // Both ad and bd are present in the config
            let desired_device = &serde_yaml::to_string(bd).unwrap()[4..];
            let mut changes = Changeset::new(current_device, desired_device, "\n");
            diffs.append(&mut changes.diffs);
        });
        b.devices
            .iter()
            .filter(|bd| a.devices.iter().find(|ad| ad.name == bd.name).is_none())
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
        diffs
    }

    pub fn diff(a: Self, b: Self) -> Vec<Difference> {
        let a = a.resolve();
        let b = b.resolve();
        Self::reconcile(&a, &b)
    }

    pub fn resolve(self) -> Self {
        let Self {
            upstream,
            mut devices,
        } = self;
        let mut addrs: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        if let Some(upstream) = upstream {
            upstream.into_iter().for_each(|up| {
                addrs.insert(up.name, up.addr);
            });
        }
        devices.iter_mut().for_each(|d| {
            d.inputs
                .listeners
                .iter_mut()
                .for_each(|input| match addrs.get(&input.addr) {
                    Some(addr) => input.addr = addr.clone(),
                    None => {}
                });
            d.outputs
                .destinations
                .iter_mut()
                .for_each(|output| match addrs.get(&output.addr) {
                    Some(addr) => output.addr = addr.clone(),
                    None => {}
                });
        });
        Self {
            upstream: None,
            devices,
        }
    }
}

fn prefix_lines(prefix: &str, lines: &str) -> String {
    let result = lines
        .split("\n")
        .map(|line| format!("{}{}\n", prefix, line))
        .fold(String::new(), |p, c| format!("{}{}", p, c));
    // Remove extra \n from the end
    String::from(&result[..result.len() - 1])
}

fn print_diff(line_prefix: &str, lines: &str, color: term::color::Color) {
    let mut t = term::stdout().unwrap();
    t.fg(color).unwrap();
    writeln!(t, "{}", prefix_lines(line_prefix, lines));
    t.reset().unwrap();
    t.flush().unwrap();
}

fn print_diffs(diffs: &Vec<Difference>) {
    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => print_diff("'", x, term::color::WHITE),
            Difference::Add(ref x) => print_diff("+", x, term::color::GREEN),
            Difference::Rem(ref x) => print_diff("-", x, term::color::RED),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const CONFIG: &'static str = include_str!("../../../config-v1.yaml");

    fn is_same(diff: &Difference) -> bool {
        match diff {
            Difference::Same(_) => true,
            _ => false,
        }
    }

    fn is_add(diff: &Difference) -> bool {
        match diff {
            Difference::Add(b) => true,
            _ => false,
        }
    }

    fn is_rem(diff: &Difference) -> bool {
        match diff {
            Difference::Rem(b) => true,
            _ => false,
        }
    }

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
        // This results in the device being deleted and an
        // entirely new one being created in its place.
        let current = Config::from_yaml(CONFIG).unwrap();
        let current_device: String = serde_yaml::to_string(&current.devices[0]).unwrap();
        let device_lines = serde_yaml::to_string(&current.devices[0])
            .unwrap()
            .split("\n")
            .collect::<Vec<_>>()
            .len()
            - 1;
        let mut desired = current.clone();
        desired.devices[0].name = String::from("New Virtual Device");
        let desired_device: String = serde_yaml::to_string(&desired.devices[0]).unwrap();
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), device_lines * 2);
    }

    #[test]
    fn test_mutate_input_channels() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].inputs.channels = 4;
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 4);
        assert_eq!(diffs[1], Difference::Rem(String::from("    channels: 2")));
        assert_eq!(diffs[2], Difference::Add(String::from("    channels: 4")));
    }

    #[test]
    fn test_mutate_listener_addr() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].inputs.listeners[0].addr = String::from("127.0.0.1:2000/TCP");
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 4);
        assert_eq!(
            diffs[1],
            Difference::Rem(String::from("      - addr: \"127.0.0.1:20001/UDP\""))
        );
        assert_eq!(
            diffs[2],
            Difference::Add(String::from("      - addr: \"127.0.0.1:2000/TCP\""))
        );
    }

    #[test]
    fn test_add_listener() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].inputs.listeners.push(Listener {
            addr: String::from("127.0.0.1:2000/TCP"),
            tls: None,
        });
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 3);
        assert!(is_add(&diffs[1]));
    }

    #[test]
    fn test_remove_listener() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].inputs.listeners = vec![];
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 4);
        assert!(is_rem(&diffs[1]));
        assert_eq!(diffs[2], Difference::Add(String::from("    listeners: []")));
    }

    #[test]
    fn test_mutate_output_channels() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].outputs.channels = 4;
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 4);
        assert_eq!(diffs[1], Difference::Rem(String::from("    channels: 2")));
        assert_eq!(diffs[2], Difference::Add(String::from("    channels: 4")));
    }

    #[test]
    fn test_mutate_destination_addr() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].outputs.destinations[0].addr = String::from("127.0.0.1:2000/TCP");
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 4);
        assert_eq!(
            diffs[1],
            Difference::Rem(String::from("      - addr: \"127.0.0.1:20001/UDP\""))
        );
        assert_eq!(
            diffs[2],
            Difference::Add(String::from("      - addr: \"127.0.0.1:2000/TCP\""))
        );
    }

    #[test]
    fn test_add_destination() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].outputs.destinations.push(Destination {
            addr: String::from("127.0.0.1:2000/TCP"),
            channels: None,
            tls: None,
        });
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 2);
        assert!(is_add(&diffs[1]));
    }

    #[test]
    fn test_remove_destination() {
        let current = Config::from_yaml(CONFIG).unwrap();
        let mut desired = current.clone();
        desired.devices[0].outputs.destinations = vec![];
        let diffs = Config::diff(current, desired);
        assert_eq!(diffs.len(), 3);
        assert!(is_rem(&diffs[1]));
        assert_eq!(
            diffs[2],
            Difference::Add(String::from("    destinations: []"))
        );
    }
}
