use crate::key_poller::KeyPoller;
use crate::key_poller::Keycode;
use crate::optimizer::Listener;
use crate::Config;
use crate::Squad;
use crate::CAPTAIN_MULTIPLIER;

pub struct TopSquad {
    n_squads: usize,
    top_squad: Squad,
    key_poller: KeyPoller,
    current_squad: Squad,
    config: Config,
    top_adjusted_metric: f32,
    n_tries_for_top: usize,
}
impl TopSquad {
    pub fn new(current_squad: Squad, config: Config) -> Self {
        let mut squad = TopSquad {
            n_squads: 0,
            key_poller: KeyPoller::new(Keycode::P),
            top_squad: current_squad.clone(),
            current_squad: current_squad,
            config: config,
            top_adjusted_metric: 0.0,
            n_tries_for_top: 0,
        };
        squad.top_adjusted_metric = squad.adjusted_metric(&squad.current_squad);
        squad
    }
    pub fn n_squads_checked(&self) -> usize {
        self.n_squads
    }
    pub fn top_squad_idx(&self) -> usize {
        self.n_tries_for_top
    }
    fn adjusted_metric(&self, squad: &Squad) -> f32 {
        let n_changes = squad.number_of_changes(&self.current_squad);
        let max_n = (0 as f32).max(
            (n_changes as i32 - self.config.free_transfers as i32) as f32
                * self.config.transfer_cost,
        );
        squad.best_starter_lineup().total_metric(CAPTAIN_MULTIPLIER) - max_n
    }
    fn set_top_squad(&mut self, squad: &Squad) {
        self.top_squad = squad.clone();
        self.top_adjusted_metric = self.adjusted_metric(&self.top_squad);
        self.n_tries_for_top = self.n_squads;
    }
    pub fn changes_for_top(&self) -> String {
        self.top_squad.changed_squad(&self.current_squad)
    }
}
impl Listener for TopSquad {
    fn notify_new_squad(&mut self, squad: &Squad) {
        self.n_squads += 1;
        let squad_adjusted_metric = self.adjusted_metric(squad);
        if squad_adjusted_metric > self.top_adjusted_metric {
            self.set_top_squad(&squad);
            println!(
                "Found a squad with better metric! Squad #: {}, New metric: {:.2}{: <3$}\n",
                self.n_squads, self.top_adjusted_metric, "", 60
            );
        } else if squad_adjusted_metric == self.top_adjusted_metric {
            let bench_points_required_for_change =
                (squad.total_cost() - self.top_squad.total_cost()) * self.config.bench_point_value; // If positive, I prefer changed_squad unless it has a nice bench
            if squad.bench().total_metric(1.0) - self.top_squad.bench().total_metric(1.0)
                > bench_points_required_for_change
            {
                self.set_top_squad(&squad);
                println!("Found a squad was as good but with better value/bench! Squad #: {}, New metric: {:.2}{: <3$}\n", self.n_squads, self.top_adjusted_metric, "", 60);
            }
        }
        if self.n_squads % 1000 == 0 {
            if self.key_poller.poll() {
                println!("{}", self.changes_for_top());
            }
        }
    }
}
