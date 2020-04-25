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
#[macro_use] extern crate vst;
#[macro_use] extern crate log;
#[macro_use] extern crate tokio;
extern crate log4rs;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;

mod editor;
mod stream;

// this is a 4-pole filter with resonance, which is why there's 4 states and vouts
#[derive(Clone)]
struct RemoteAudioEffect {
    // Receive streams
    rx: Vec<std::sync::Arc<stream::rx::RxStream<stream::rx::locking::LockingRxBuffer>>>,

    // Send streams
    tx: Vec<std::sync::Arc<stream::tx::TxStream<stream::tx::locking::LockingTxBuffer>>>,

    rt: std::sync::Arc<tokio::runtime::Runtime>,

    // Store a handle to the plugin's parameter object.
    params: Arc<LadderParameters>,
    // the output of the different filter stages
    vout: [f32; 4],
    // s is the "state" parameter. In an IIR it would be the last value from the filter
    // In this we find it by trapezoidal integration to avoid the unit delay
    s: [f32; 4],
}

struct LadderParameters {
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

impl Default for LadderParameters {
    fn default() -> LadderParameters {
        START.call_once(|| entrypoint());
        LadderParameters {
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
// member methods for the struct
impl RemoteAudioEffect {
    fn running(&mut self) -> bool {
        false
    }
    // the state needs to be updated after each process. Found by trapezoidal integration
    fn update_state(&mut self) {
        self.s[0] = 2. * self.vout[0] - self.s[0];
        self.s[1] = 2. * self.vout[1] - self.s[1];
        self.s[2] = 2. * self.vout[2] - self.s[2];
        self.s[3] = 2. * self.vout[3] - self.s[3];
    }
    // performs a complete filter process (mystran's method)
    fn tick_pivotal(&mut self, input: f32) {
        if self.params.drive.get() > 0. {
            self.run_ladder_nonlinear(input * (self.params.drive.get() + 0.7));
        } else {
            //
            self.run_ladder_linear(input);
        }
        self.update_state();
    }
    // nonlinear ladder filter function with distortion.
    fn run_ladder_nonlinear(&mut self, input: f32) {
        let mut a = [1f32; 5];
        let base = [input, self.s[0], self.s[1], self.s[2], self.s[3]];
        // a[n] is the fixed-pivot approximation for tanh()
        for n in 0..base.len() {
            if base[n] != 0. {
                a[n] = base[n].tanh() / base[n];
            } else {
                a[n] = 1.;
            }
        }
        // denominators of solutions of individual stages. Simplifies the math a bit
        let g0 = 1. / (1. + self.params.g.get() * a[1]);
        let g1 = 1. / (1. + self.params.g.get() * a[2]);
        let g2 = 1. / (1. + self.params.g.get() * a[3]);
        let g3 = 1. / (1. + self.params.g.get() * a[4]);
        //  these are just factored out of the feedback solution. Makes the math way easier to read
        let f3 = self.params.g.get() * a[3] * g3;
        let f2 = self.params.g.get() * a[2] * g2 * f3;
        let f1 = self.params.g.get() * a[1] * g1 * f2;
        let f0 = self.params.g.get() * g0 * f1;
        // outputs a 24db filter
        self.vout[3] =
            (f0 * input * a[0] + f1 * g0 * self.s[0] + f2 * g1 * self.s[1] + f3 * g2 * self.s[2] + g3 * self.s[3])
                / (f0 * self.params.res.get() * a[3] + 1.);
        // since we know the feedback, we can solve the remaining outputs:
        self.vout[0] = g0
            * (self.params.g.get() * a[1] * (input * a[0] - self.params.res.get() * a[3] * self.vout[3]) + self.s[0]);
        self.vout[1] = g1 * (self.params.g.get() * a[2] * self.vout[0] + self.s[1]);
        self.vout[2] = g2 * (self.params.g.get() * a[3] * self.vout[1] + self.s[2]);
    }
    // linear version without distortion
    pub fn run_ladder_linear(&mut self, input: f32) {
        // denominators of solutions of individual stages. Simplifies the math a bit
        let g0 = 1. / (1. + self.params.g.get());
        let g1 = self.params.g.get() * g0 * g0;
        let g2 = self.params.g.get() * g1 * g0;
        let g3 = self.params.g.get() * g2 * g0;
        // outputs a 24db filter
        self.vout[3] =
            (g3 * self.params.g.get() * input + g0 * self.s[3] + g1 * self.s[2] + g2 * self.s[1] + g3 * self.s[0])
                / (g3 * self.params.g.get() * self.params.res.get() + 1.);
        // since we know the feedback, we can solve the remaining outputs:
        self.vout[0] = g0 * (self.params.g.get() * (input - self.params.res.get() * self.vout[3]) + self.s[0]);
        self.vout[1] = g0 * (self.params.g.get() * self.vout[0] + self.s[1]);
        self.vout[2] = g0 * (self.params.g.get() * self.vout[1] + self.s[2]);
    }
}
impl LadderParameters {
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
impl PluginParameters for LadderParameters {
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
        let rt = std::sync::Arc::new(tokio::runtime::Builder::new()
            .threaded_scheduler()
            .build()
            .unwrap());
        RemoteAudioEffect {
            vout: [0f32; 4],
            s: [0f32; 4],
            params: Arc::new(LadderParameters::default()),
            rx: vec![],
            tx: vec![],
            rt,
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
        if !self.running() {
            return
        }
        let (inputs, mut outputs) = buffer.split();
        if inputs.len() != self.tx.len() {
            //self.set_error()
            error!("num inputs ({}) does not match num tx streams ({})", inputs.len(), self.tx.len());
            return;
        }
        if outputs.len() != self.rx.len() {
            error!("num outputs ({}) does not match num rx streams ({})", outputs.len(), self.rx.len());
            return;
        }
        inputs.into_iter()
            .zip(self.tx.iter())
            .for_each(|(input, tx)| tx.process(input));
        outputs.into_iter()
            .zip(self.rx.iter())
            .for_each(|(output, rx)| rx.process(output));
    }
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        info!("get_parameter_object()");
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
    fn get_editor(&mut self) -> Option<Box<dyn vst::editor::Editor>> {
        info!("get_editor()");
        Some(Box::new(editor::Editor::new()))
    }
}
plugin_main!(RemoteAudioEffect);