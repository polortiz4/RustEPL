use crate::key_poller::KeyPoller;
use crate::key_poller::Keycode;
use crate::Player;
use crate::Squad;

pub struct Logger {
    n_squads: usize,
    last_line: String,
    key_poller: KeyPoller,
    current_squad: Squad,
    full_player_list: Vec<Player>,
}

impl Logger {
    pub fn new(current_squad: Squad, full_player_list: &[Player]) -> Self {
        Logger {
            n_squads: 0,
            last_line: String::from(""),
            key_poller: KeyPoller::new(Keycode::P),
            current_squad: current_squad,
            full_player_list: full_player_list.to_vec(),
        }
    }
    pub fn notify_new_squad(&mut self, squad: &Squad, available_players: &[Player]) {
        let mut line = String::from(format!(
            "Valid squads found: {}, Progress (over {}): ",
            self.n_squads,
            self.full_player_list.len()
        ));
        for player in &squad.players {
            line.push_str(&format!(
                "{:?}, ",
                self.full_player_list
                    .iter()
                    .position(|p| p == player)
                    .unwrap()
            ));
        }
        self.n_squads += 1;
        line.push_str(&format!("{: <1$}\r", "", 30));
        if (self.n_squads - 1) % 100 == 0 {
            print!("\r{}", line);
            // print!("\r{}{: <1$}\r", line, self.last_line.len());
            self.last_line = line;
            if self.key_poller.poll() {
                println!("{}", squad.changed_squad(&self.current_squad));
            }
        }
    }
}
