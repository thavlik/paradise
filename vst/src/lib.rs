//! This zero-delay feedback filter is based on a 4-stage transistor ladder filter.
//! It follows the following equations:
//! x = input - tanh(self.res * self.vout[3])
//! vout[0] = self.params.g.get() * (tanh(x) - tanh(self.vout[0])) + self.s[0]
//! vout[1] = self.params.g.get() * (tanh(self.vout[0]) - tanh(self.vout[1])) + self.s[1]
//! vout[0] = self.params.g.get() * (tanh(self.vout[1]) - tanh(self.vout[2])) + self.s[2]
//! vout[0] = self.params.g.get() * (tanh(self.vout[2]) - tanh(self.vout[3])) + self.s[3]
//! since we can't easily solve a nonlinear equation,
//! Mystran's fixed-pivot method is used to approximate the tanh() parts.
//! Quality can be improved a lot by oversampling a bit.
//! Feedback is clipped independently of the input, so it doesn't disappear at high gains.
#[macro_use]
extern crate vst;
#[macro_use]
extern crate log;
#[macro_use]
extern crate tokio;
#[macro_use]
extern crate crossbeam;
#[macro_use]
extern crate lazy_static;
extern crate log4rs;

use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

mod editor;
mod stream;
mod runtime;

type TxStream = stream::tx::udp::UdpTxStream::<stream::tx::locking::LockingTxBuffer>;
type RxStream = stream::rx::udp::UdpRxStream::<stream::rx::locking::LockingRxBuffer>;

//type TxStream = stream::tx::tcp::TcpTxStream::<stream::tx::locking::LockingTxBuffer>;
//type RxStream = stream::rx::tcp::TcpRxStream::<stream::rx::locking::LockingRxBuffer>;


// this is a 4-pole filter with resonance, which is why there's 4 states and vouts
#[derive(Clone)]
struct RemoteAudioEffect {
    // Receive streams
    rx: Vec<std::sync::Arc<dyn stream::rx::RxStream>>,

    // Send streams
    tx: Vec<std::sync::Arc<dyn stream::tx::TxStream>>,

    // Store a handle to the plugin's parameter object.
    params: Arc<RemoteAudioEffectParameters>,

    latency: std::sync::Arc<std::sync::atomic::AtomicU64>,

    running: std::sync::Arc<std::sync::atomic::AtomicBool>,

    l: std::sync::Arc<std::sync::Mutex<()>>,

    rt: std::sync::Arc<runtime::Runtime>,
}

impl RemoteAudioEffect {

    fn ensure_started(&mut self) -> bool {
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return true;
        }
        let guard = self.l.lock().unwrap();
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return true;
        }
        let rt = runtime::Runtime::get();
        if self.tx.len() == 0 {
            let dest_addr = std::net::SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(127, 0, 0, 1), 30001));
            let send_port = 0; //match rt.outbound.reserve() {
            //    Ok(port) => port,
            //    Err(e) => {
            //        return false;
            //    }
            //};
            let tx = match TxStream::new(dest_addr) {
                Ok(tx) => tx,
                Err(e) => {
                    return false;
                },
            };
            self.tx = vec![tx];
        }
        if self.rx.len() == 0 {
            let receive_port = 30000; //match rt.inbound.reserve() {
            //    Ok(port) => port,
            //    Err(e) => {
            //        return false;
            //    }
            //};
            let rx = match RxStream::new(receive_port) {
                Ok(rx) => rx,
                Err(e) => {
                    return false;
                },
            };
            self.rx = vec![rx];
        }
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        true
    }
}

struct RemoteAudioEffectParameters {
    // the "cutoff" parameter. Determines how heavy filtering is
    cutoff: AtomicFloat,
    g: AtomicFloat,
    // needed to calculate cutoff.
    sample_rate: AtomicFloat,
    // makes a peak at cutoff
    res: AtomicFloat,
    // used to choose where we want our output to be
    poles: AtomicUsize,
    // pole_value is just to be able to use get_parameter on poles
    pole_value: AtomicFloat,
    // a drive parameter. Just used to increase the volume, which results in heavier distortion
    drive: AtomicFloat,
}

static START: std::sync::Once = std::sync::Once::new();

fn entrypoint() {
    env_logger::init();
    let log_path = "/Users/thomashavlik/Repositories/paradise/vst/log.txt";
    let log_file = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new("{l} - {m}\n")))
        .build(log_path)
        .unwrap();
    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("logfile", Box::new(log_file)))
        .build(log4rs::config::Root::builder().appender("logfile").build(log::LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();
    info!("Start successfully");
}

impl Default for RemoteAudioEffectParameters {
    fn default() -> RemoteAudioEffectParameters {
        //START.call_once(|| entrypoint());
        RemoteAudioEffectParameters {
            cutoff: AtomicFloat::new(1000.),
            res: AtomicFloat::new(2.),
            poles: AtomicUsize::new(3),
            pole_value: AtomicFloat::new(1.),
            drive: AtomicFloat::new(0.),
            sample_rate: AtomicFloat::new(44100.),
            g: AtomicFloat::new(0.07135868),
        }
    }
}

impl RemoteAudioEffectParameters {
    pub fn set_cutoff(&self, value: f32) {
        // cutoff formula gives us a natural feeling cutoff knob that spends more time in the low frequencies
        self.cutoff.set(20000. * (1.8f32.powf(10. * value - 10.)));
        // bilinear transformation for g gives us a very accurate cutoff
        self.g.set((PI * self.cutoff.get() / (self.sample_rate.get())).tan());
    }

    // returns the value used to set cutoff. for get_parameter function
    pub fn get_cutoff(&self) -> f32 {
        1. + 0.17012975 * (0.00005 * self.cutoff.get()).ln()
    }

    pub fn set_poles(&self, value: f32) {
        self.pole_value.set(value);
        self.poles.store(((value * 3.).round()) as usize, Ordering::Relaxed);
    }
}

impl PluginParameters for RemoteAudioEffectParameters {
    // get_parameter has to return the value used in set_parameter
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.get_cutoff(),
            1 => self.res.get() / 4.,
            2 => self.pole_value.get(),
            3 => self.drive.get() / 5.,
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.set_cutoff(value),
            1 => self.res.set(value * 4.),
            2 => self.set_poles(value),
            3 => self.drive.set(value * 5.),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "cutoff".to_string(),
            1 => "resonance".to_string(),
            2 => "filter order".to_string(),
            3 => "drive".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_label(&self, index: i32) -> String {
        match index {
            0 => "Hz".to_string(),
            1 => "%".to_string(),
            2 => "poles".to_string(),
            3 => "%".to_string(),
            _ => "".to_string(),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most sense.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{:.0}", self.cutoff.get()),
            1 => format!("{:.3}", self.res.get()),
            2 => format!("{}", self.poles.load(Ordering::Relaxed) + 1),
            3 => format!("{:.3}", self.drive.get()),
            _ => format!(""),
        }
    }
}

impl Default for RemoteAudioEffect {
    fn default() -> RemoteAudioEffect {
        RemoteAudioEffect {
            params: Arc::new(RemoteAudioEffectParameters::default()),
            rx: vec![],
            tx: vec![],
            latency: std::sync::Arc::new(std::default::Default::default()),
            running: std::sync::Arc::new(std::default::Default::default()),
            l: std::sync::Arc::new(std::sync::Mutex::new(())),
            rt: runtime::Runtime::get(),
        }
    }
}

impl Plugin for RemoteAudioEffect {
    fn set_sample_rate(&mut self, rate: f32) {
        info!("set_sample_rate(rate={})", rate);
        self.params.sample_rate.set(rate);
    }

    fn get_info(&self) -> Info {
        info!("get_info()");
        Info {
            name: "RemoteAudioEffect".to_string(),
            unique_id: 9263,
            inputs: 1,
            outputs: 1,
            category: Category::Effect,
            parameters: 4,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (inputs, mut outputs) = buffer.split();
        if !self.ensure_started() {
            outputs.into_iter()
                .for_each(|output| output.iter_mut()
                    .for_each(|v| *v = 0.0));
            return;
        }
        if inputs.len() != self.tx.len() {
            //panic!("num inputs ({}) does not match num tx streams ({})", inputs.len(), self.tx.len());
            return;
        }
        if outputs.len() != self.rx.len() {
            //panic!("num outputs ({}) does not match num rx streams ({})", outputs.len(), self.rx.len());
            return;
        }
        let clock = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
        inputs.into_iter()
            .zip(self.tx.iter())
            .for_each(|(input, tx)| tx.process(input, clock));
        let latency = clock - outputs.into_iter()
            .zip(self.rx.iter())
            .map(|(output, rx)| {
                output.iter_mut()
                    .for_each(|v| *v = 0.0);
                rx.process(output)
            })
            .fold(0, |p, c| p.max(c));
        self.latency.store(latency, std::sync::atomic::Ordering::SeqCst);
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        info!("get_parameter_object()");
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    fn get_editor(&mut self) -> Option<Box<dyn vst::editor::Editor>> {
        info!("get_editor()");
        //Some(Box::new(editor::Editor::new()))
        None
    }

    fn process_events(&mut self, events: &vst::api::Events) {
        events.events().for_each(|event| {

        });
    }
}

plugin_main!(RemoteAudioEffect);