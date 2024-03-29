use crate::logger::Logger;
use crate::optimizer::Listener;
use crate::optimizer::Optimizer;
use crate::player::Player;
use crate::squad::Squad;
use crate::top_squad::TopSquad;
use clap::{load_yaml, App};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use std::io;

const CAPTAIN_MULTIPLIER: f32 = 2.0;

mod api;
mod key_poller;
mod logger;
mod optimizer;
mod player;
mod squad;
mod team;
mod top_squad;

#[derive(Debug)]
struct PlayerNotFound(String);
fn add_by_last_name(
    squad: &mut Squad,
    last_name: String,
    full_list: &Vec<Player>,
) -> Result<(), PlayerNotFound> {
    for player in full_list {
        if player.name == last_name {
            squad.try_add_player(&player).expect("Error adding player");
            return Ok(());
        }
    }
    Err(PlayerNotFound(format!(
        "Couldn't find player: {}",
        last_name
    )))
}

fn custom_squad(full_list: &Vec<Player>) -> Squad {
    let money_in_bank = 0.0;
    let mut current_squad = Squad::new(1000.0);
    add_by_last_name(&mut current_squad, "Martínez".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Sánchez".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Robertson".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Cresswell".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Wan-Bissaka".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Dunk".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Targett".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Fernandes".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Son".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Dallas".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Harrison".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Ward-Prowse".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Benteke".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Bamford".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Watkins".to_string(), &full_list).unwrap();
    current_squad.sort_players();
    current_squad.set_max_cost(money_in_bank + current_squad.total_cost());
    current_squad
}

fn get_top_n_players(full_list: Vec<Player>, n_players: usize, squad: &Squad) -> Vec<Player> {
    let mut result: Vec<Player> = Vec::with_capacity(n_players + squad.players.len());
    squad.players.iter().for_each(|p| result.push(p.clone()));

    for player in full_list {
        if result.capacity() == result.len() {
            break;
        }
        if !result.contains(&player) {
            result.push(player.clone());
        }
    }
    result.sort_by(|a, b| {
        b.metric()
            .partial_cmp(&a.metric())
            .expect("Error sorting players")
    });
    result
}

// fn run(){
// let mut poller = KeyPoller::new(Keycode::P);
// loop {
//     if poller.poll() {
//         println!("Pressed key: {:?}", poller.key);
//     }
// }
// }

fn read_gameweek() -> u8 {
    println!("What was the last gameweek?");
    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("failed to read from stdin");

    let trimmed = input_text.trim();
    trimmed.parse::<u8>().unwrap()
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let list = api::get_full_sorted_player_list().unwrap();
    let current_squad = if config.overwrite_pulled_team {
        custom_squad(&list)
    } else {
        let gameweek = config.gameweek.unwrap_or_else(||read_gameweek());
        api::get_my_squad(config.user_id, gameweek, &list)?
    };
    let reduced_list = if let Some(_) = config.min_player_metric {
        panic!("Min_acceptable player metric not implemented yet");
    } else {
        get_top_n_players(
            list,
            config
                .top_n_player
                .expect("expected either a top_n_players value, or a min_acceptable_metric"),
            &current_squad,
        )
    };

    let logger = Rc::new(RefCell::new(Logger::new(&reduced_list)));
    let top_squad_holder = Rc::new(RefCell::new(TopSquad::new(
        current_squad.clone(),
        config.clone(),
    )));
    let mut new_squad = Squad::new(current_squad.max_cost());
    let mut optimizer = Optimizer::new(
        Some(current_squad.clone()),
        config.transfer_cost,
        None,
        Some(config.free_transfers),
        None,
        None,
    );
    optimizer.register(Rc::clone(&logger) as Rc<RefCell<dyn Listener>>);
    optimizer.register(Rc::clone(&top_squad_holder) as Rc<RefCell<dyn Listener>>);
    let _ = optimizer.fill_squad(&mut new_squad, &reduced_list);
    println!(
        "Top Squad:\n{}",
        top_squad_holder.borrow().changes_for_top()
    );
    println!(
        "Number of Squads checked: {}",
        top_squad_holder.borrow().n_squads_checked()
    );
    println!(
        "Top Squad found after {} valid squads",
        top_squad_holder.borrow().top_squad_idx()
    );
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Config {
    pub gameweek: Option<u8>, // Not used yet
    pub password: bool,       // Not used yet
    pub user_id: u32,         // Not used yet
    pub verbose: bool,        // Not used yet
    pub top_n_player: Option<usize>,
    pub free_transfers: usize,
    pub overwrite_pulled_team: bool,
    pub min_player_metric: Option<f32>,
    pub transfer_cost: f32,
    pub bench_point_value: f32, // Not used yet
}
impl Config {
    pub fn parse_cli() -> Config {
        let yaml = load_yaml!("cli.yml");
        let m = App::from(yaml).get_matches();
        let mut top_n_players = Some(
            m.value_of("top_n_players")
                .unwrap()
                .parse::<usize>()
                .unwrap(),
        );
        let min_metric = match m.value_of("min_metric") {
            Some(metric) => {
                top_n_players = None;
                println!(
                    "Choosing from players with metric > {} instead of using a top_n_players",
                    metric.parse::<f32>().unwrap()
                );
                Some(metric.parse::<f32>().unwrap())
            }
            None => None,
        };
        let gameweek = match m.value_of("gameweek") {
            Some(gweek) => Some(gweek.parse::<u8>().unwrap()),
            None => None
        };
        Config {
            gameweek: gameweek,
            password: m.is_present("password"),
            verbose: m.is_present("verbose"),
            overwrite_pulled_team: m.is_present("overwrite"),
            user_id: m.value_of("user_id").unwrap().parse::<u32>().unwrap(),
            top_n_player: top_n_players,
            free_transfers: m
                .value_of("free_transfers")
                .unwrap()
                .parse::<usize>()
                .unwrap(),
            min_player_metric: min_metric,
            transfer_cost: m.value_of("transfer_cost").unwrap().parse::<f32>().unwrap(),
            bench_point_value: m
                .value_of("bench_point_value")
                .unwrap()
                .parse::<f32>()
                .unwrap(),
        }
    }
}
