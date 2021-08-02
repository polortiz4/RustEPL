use crate::key_poller::KeyPoller;
use crate::optimizer::Optimizer;
use crate::player::{Player, Position};
use crate::squad::Squad;
use clap::{load_yaml, App, Arg};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::error::Error;

mod api;
mod key_poller;
mod optimizer;
mod player;
mod squad;
mod team;

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
    let money_in_bank = 11.3;
    let mut current_squad = Squad::new(1000.0);
    add_by_last_name(&mut current_squad, "Sánchez".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Meslier".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Dunk".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Cresswell".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Stones".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Targett".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Cancelo".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Mané".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Tielemans".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Maddison".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Son".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Gündogan".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Lacazette".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Antonio".to_string(), &full_list).unwrap();
    add_by_last_name(&mut current_squad, "Maupay".to_string(), &full_list).unwrap();
    current_squad.sort_players();
    current_squad.set_max_cost(money_in_bank + current_squad.total_cost());
    current_squad
}

fn get_top_n_players(full_list: Vec<Player>, n_players: usize, squad: &Squad) -> Vec<Player> {
    let mut result: Vec<Player> = Vec::with_capacity(n_players + squad.players.len());
    squad.players.iter().for_each(|p| result.push(p.clone()));

    for player in full_list {
        if result.capacity() == result.len() {
            return result;
        }
        if !result.contains(&player) {
            result.push(player.clone());
        }
    }
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let list = api::get_full_sorted_player_list().unwrap();
    let current_squad = if config.overwrite_pulled_team {
        custom_squad(&list)
    } else {
        panic!("Team pull not implemented yet. Pass the --overwrite-pulled-team for now");
        // Squad::new(100.0)
    };
    if let Some(_) = config.min_player_metric {
        panic!("Min_acceptable player metric not implemented yet");
    }
    let reduced_list = get_top_n_players(
        list,
        config.top_n_player.expect("expected a top_n_players value"),
        &current_squad,
    );
    let mut new_squad = Squad::new(current_squad.max_cost());
    let mut optimizer = Optimizer::new(
        Some(current_squad),
        config.transfer_cost,
        None,
        Some(config.free_transfers),
        None,
        None,
    );
    optimizer.fill_squad(&mut new_squad, &reduced_list);
    Ok(())
}

#[derive(Debug)]
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
            None => {
                if !m.is_present("overwrite") {
                    panic!("Please provide a gameweek, or overwrite team");
                }
                None
            }
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::team::Team;

    #[test]
    fn test_list_filter() {
        let full_list = api::get_full_sorted_player_list().unwrap();
        let custom_squad = custom_squad(&full_list);
        let reduced_list = get_top_n_players(full_list, 10, &custom_squad);
        assert_eq!(
            "[Sánchez, Meslier, Dunk, Cresswell, Stones, Targett, Cancelo, Mané, Tielemans, Maddison, Son, Gündogan, Lacazette, Antonio, Maupay, Fernandes, Kane, Salah, Bamford, Vardy, Martínez, Rashford, Dallas, Watkins, Calvert-Lewin]",
            format!("{:?}", reduced_list)
        );
        assert!(custom_squad.positions_full());
    }
}
