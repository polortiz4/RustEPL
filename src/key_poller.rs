pub use device_query::Keycode;
use device_query::{DeviceQuery, DeviceState};

pub struct KeyPoller {
    pub key: Keycode,
    was_pressed: bool,
    device_state: DeviceState,
}
impl KeyPoller {
    fn is_pressed(&self) -> bool {
        self.device_state.get_keys().contains(&self.key)
    }
    pub fn poll(&mut self) -> bool {
        if !self.was_pressed && self.is_pressed() {
            // Rising Edge
            self.was_pressed = true;
            true
        } else if self.was_pressed && !self.is_pressed() {
            // Falling Edge
            self.was_pressed = false;
            false
        } else {
            false
        }
    }
    pub fn new(key: Keycode) -> Self {
        KeyPoller {
            was_pressed: false,
            key: key,
            device_state: DeviceState::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn main_test() {
        let device_state = DeviceState::new();
        loop {
            let keys: Vec<Keycode> = device_state.get_keys();
            for key in keys.iter() {
                println!("Pressed key: {:?}", key);
            }
        }
    }
}
