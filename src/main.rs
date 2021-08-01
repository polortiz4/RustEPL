use crate::player::{Player, Position};
use crate::squad::Squad;

mod api;
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

fn get_top_n_players(
    full_list: Vec<Player>,
    n_players: usize,
    squad: &Squad,
    min_acceptable_metric: f32,
) -> Vec<Player> {
    let mut result: Vec<Player> = Vec::with_capacity(n_players + squad.players.len());
    squad.players.iter().for_each(|p| result.push(p.clone()));

    for player in full_list {
        if result.capacity() == result.len() {
            return result;
        }
        if player.metric() > min_acceptable_metric && !result.contains(&player) {
            result.push(player.clone());
        }
    }
    result
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let list = api::get_full_sorted_player_list().unwrap();
    for player in list {
        println!("{}", player.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::team::Team;

    #[test]
    fn test_list_filter() {
        let full_list = api::get_full_sorted_player_list().unwrap();
        let custom_squad = custom_squad(&full_list);
        let reduced_list = get_top_n_players(full_list, 10, &custom_squad, 2.0);
        assert_eq!(
            "[Sánchez, Meslier, Dunk, Cresswell, Stones, Targett, Cancelo, Mané, Tielemans, Maddison, Son, Gündogan, Lacazette, Antonio, Maupay, Fernandes, Kane, Salah, Bamford, Vardy, Martínez, Rashford, Dallas, Watkins, Calvert-Lewin]",
            format!("{:?}", reduced_list)
        );
        assert!(custom_squad.positions_full());
    }
}
