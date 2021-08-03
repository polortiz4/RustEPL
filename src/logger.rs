use crate::optimizer::Listener;
use crate::Player;
use crate::Squad;

pub struct Logger {
    n_squads: usize,
    last_line: String,
    full_player_list: Vec<Player>,
}

impl Logger {
    pub fn new(full_player_list: &[Player]) -> Self {
        Logger {
            n_squads: 0,
            last_line: String::from(""),
            full_player_list: full_player_list.to_vec(),
        }
    }
}
impl Listener for Logger {
    fn notify_new_squad(&mut self, squad: &Squad) {
        self.n_squads += 1;
        if self.n_squads % 500 == 0 {
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
            line.push_str(&format!("{: <1$}\r", "", 30));
            print!("\r{}", line);
            self.last_line = line;
        }
    }
}
