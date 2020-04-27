#[derive(Clone)]
pub struct PortPool {
    reserved: std::sync::Arc<std::sync::Mutex<Vec<u16>>>,
    min: u16,
    max: u16,
}

pub struct Runtime {
    pub rt: std::sync::Mutex<tokio::runtime::Runtime>,
    pub outbound: PortPool,
    pub inbound: PortPool,
}

lazy_static! {
    static ref _RT: std::sync::Mutex<Option<std::sync::Weak<Runtime>>> = std::sync::Mutex::new(None);
}


impl PortPool {
    fn new(min: u16, max: u16) -> Self {
        Self {
            reserved: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            min,
            max,
        }
    }

    pub fn reserve(&self) -> std::io::Result<u16> {
        let mut reserved = self.reserved.lock()
            .unwrap();
        match (self.min..self.max)
            .find(|port| !reserved.contains(port)) {
            Some(port) => {
                reserved.push(port);
                Ok(port)
            },
            None => Err(std::io::ErrorKind::AddrNotAvailable.into())
        }
    }

    pub fn release(&self, port: u16) {
        let mut reserved = self.reserved.lock()
            .unwrap();
        match reserved.iter()
            .position(|p| *p == port) {
            Some(i) => reserved.remove(i),
            None => return,
        };
    }
}

impl Runtime {
    fn new() -> Self {
        let rt = std::sync::Mutex::new(tokio::runtime::Builder::new()
            .threaded_scheduler()
            .build()
            .unwrap());
        Self {
            rt,
            inbound: PortPool::new(31000, 3199),
            outbound: PortPool::new(32000, 32999),
        }
    }

    pub fn get() -> std::sync::Arc<Runtime> {
        unsafe {
            let mut guard = _RT.lock().unwrap();
            match &*guard {
                Some(runtime) => {
                    match runtime.upgrade() {
                        Some(runtime) => return runtime.clone(),
                        None => {}
                    }
                },
                None => {},
            };
            let v = std::sync::Arc::new(Runtime::new());
            *guard = Some(std::sync::Arc::downgrade(&v));
            v
        }
    }
}