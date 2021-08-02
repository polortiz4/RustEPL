use crate::Player;
use crate::Squad;
use std::f32;

pub trait Listener {
    fn notify_new_squad(&mut self, squad: &Squad);
}
#[derive(Debug, PartialEq)]
pub struct SquadNotFull(String);

pub struct Optimizer {
    transfer_cost: f32,
    squad_max_len: usize,
    observers: Vec<Box<dyn Listener>>,
    cheapest_cost: Option<f32>,
    current_squad: Option<Squad>,
    n_free_transfers: usize,
    min_metric: Option<f32>,
    max_metric: Option<f32>,
    stack_i: usize,
}

impl Optimizer {
    pub fn new(
        current_squad: Option<Squad>,
        transfer_cost: f32,
        squad_max_len: Option<usize>,
        n_free_transfers: Option<usize>,
        min_metric: Option<f32>,
        max_metric: Option<f32>,
    ) -> Self {
        let squad_max_len = squad_max_len.unwrap_or(15);
        let n_free_transfers = n_free_transfers.unwrap_or(squad_max_len);
        Optimizer {
            transfer_cost: transfer_cost,
            squad_max_len: squad_max_len,
            observers: Vec::new(),
            cheapest_cost: None,
            current_squad: current_squad,
            n_free_transfers: n_free_transfers,
            min_metric: min_metric,
            max_metric: max_metric,
            stack_i: 1,
        }
    }
    pub fn register(&mut self, logger: Box<dyn Listener>)
    {
        self.observers.push(logger);
    }
    pub fn trigger_callbacks(&mut self, squad: &Squad) {
        for logger in &mut self.observers {
            logger.notify_new_squad(&squad);
        }
    }

    fn update_max_metric(&mut self, available_players: &[Player]) {
        self.max_metric = Some(
            available_players
                .iter()
                .map(|p| p.metric())
                .fold(-f32::INFINITY, |a, b| a.max(b)),
        );
    }

    fn update_min_metric(&mut self, available_players: &[Player]) {
        self.min_metric = Some(
            available_players
                .iter()
                .map(|p| p.metric())
                .fold(f32::INFINITY, |a, b| a.min(b)),
        );
    }

    fn update_cheapest_cost(&mut self, available_players: &[Player]) {
        self.cheapest_cost = Some(
            available_players
                .iter()
                .map(|p| p.price)
                .fold(f32::INFINITY, |a, b| a.min(b)),
        );
    }

    fn skip_step(
        &self,
        no_new_players: bool,
        squad: &Squad,
        len_players: usize,
        p: &Player,
    ) -> bool {
        if len_players < self.squad_max_len - 1
            && (squad.max_cost() - (squad.total_cost() + p.price))
                / ((self.squad_max_len - 1 - len_players) as f32)
                <= self.cheapest_cost.expect("Cheapest cost is not defined")
        {
            return true;
        }
        if squad.total_cost() + p.price > squad.max_cost() {
            return true;
        }
        if let Some(c_squad) = &self.current_squad {
            if no_new_players && !c_squad.has_player(p) {
                return true;
            }
        }
        false
    }
    pub fn fill_squad(
        &mut self,
        squad: &mut Squad,
        available_players: &[Player],
    ) -> Result<(), SquadNotFull> {
        if self.min_metric.is_none() {
            self.update_min_metric(available_players);
        }
        if self.max_metric.is_none() {
            self.update_max_metric(available_players);
        }
        if self.cheapest_cost.is_none() {
            self.update_cheapest_cost(available_players);
        }
        let mut no_new_players = false;
        if let Some(current_squad) = &self.current_squad {
            let changes_so_far = squad.number_of_changes(&current_squad);
            no_new_players = changes_so_far > self.n_free_transfers
                && self.max_metric.expect("Error: metric not set")
                    - self.min_metric.expect("Error: metric not set")
                    < self.transfer_cost;
        }
        let no_new_players = no_new_players; // Remove mutability

        let len_players = squad.players.len();
        if available_players.len() == 0
            || len_players + available_players.len() < self.squad_max_len
        {
            return Err(SquadNotFull("Not enough Players".to_string()));
        }

        for (i, p) in available_players.iter().enumerate() {
            assert!(!squad.has_player(p));
            if self.skip_step(no_new_players, &squad, len_players, &p) {
                continue;
            }
            if let Err(_) = squad.try_add_player(&p) {
                continue;
            }

            if squad.positions_full() {
                // Valid squad found
                self.trigger_callbacks(squad);
            } else if let Some(next_player) = available_players.get(i + 1) {
                self.max_metric = Some(next_player.metric());
                self.stack_i += 1;
                let _ = self.fill_squad(squad, &available_players[i + 1..]);
            }
            squad.remove_player(&p);
        }
        Err(SquadNotFull(String::from("Squad not full")))
    }
}
