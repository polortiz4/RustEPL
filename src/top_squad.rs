use crate::key_poller::KeyPoller;
use crate::key_poller::Keycode;
use crate::optimizer::Listener;
use crate::Squad;

pub struct TopSquad {
    n_squads: usize,
    top_squad: Squad,
    key_poller: KeyPoller,
    current_squad: Squad,
}
impl TopSquad {
    pub fn new(current_squad: Squad) -> Self {
        TopSquad {
            n_squads: 0,
            key_poller: KeyPoller::new(Keycode::P),
            top_squad: current_squad.clone(),
            current_squad: current_squad,
        }
    }
}
impl Listener for TopSquad {
    fn notify_new_squad(&mut self, squad: &Squad) {
        self.n_squads += 1;
        // if stuff {
        //     self.top_squad = squad.clone();
        // }
        self.top_squad = squad.clone();
        if (self.n_squads - 1) % 1000 == 999 {
            if self.key_poller.poll() {
                println!("{}", self.top_squad.changed_squad(&self.current_squad));
            }
        }
    }
}
