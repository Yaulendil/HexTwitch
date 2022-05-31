use std::time::{Duration, Instant};
use hexchat::EatMode;
use parking_lot::Mutex;


safe_static! {
    static lazy CALLBACK: Mutex<Option<Callback>> = Default::default();
}


#[allow(dead_code)]
pub fn register(callback: impl FnOnce(&String) -> EatMode + Send + 'static) {
    *CALLBACK.lock() = Some(Callback::new(callback));
}


pub fn run(input: &String) -> EatMode {
    match CALLBACK.lock().take() {
        Some(cb) if cb.is_valid() => cb.run(input),
        _ => EatMode::None,
    }
}


struct Callback {
    callback: Box<dyn FnOnce(&String) -> EatMode + Send + 'static>,
    registered: Instant,
}

impl Callback {
    const TIMEOUT: Duration = Duration::from_secs(5);

    fn new(callback: impl FnOnce(&String) -> EatMode + Send + 'static) -> Self {
        Self {
            callback: Box::new(callback),
            registered: Instant::now(),
        }
    }

    fn is_valid(&self) -> bool {
        self.registered.elapsed() <= Self::TIMEOUT
    }

    fn run(self, input: &String) -> EatMode {
        (self.callback)(input)
    }
}
