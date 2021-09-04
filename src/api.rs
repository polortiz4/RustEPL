use crate::player::{Player, Position};
use crate::squad::Squad;
use crate::team::Team;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

const FANTASY_API_URL: &str = "https://fantasy.premierleague.com/api/bootstrap-static/";
const LOG_IN_URL: &str = "https://users.premierleague.com/accounts/login/";

#[derive(Deserialize)]
pub struct PlayerResponse {
    pub elements: Vec<APIPlayer>,
}

#[derive(Deserialize, Debug)]
pub struct APIPlayer {
    chance_of_playing_next_round: Option<f32>,
    form: String,
    element_type: u8,
    web_name: String,
    now_cost: f32,
    team: u8,
    id: u16,
    total_points: i32,
    ep_next: String,
}

impl APIPlayer {
    pub fn to_player(&self) -> Player {
        let team = Team::new(self.team);
        let position = match self.element_type {
            1 => Some(Position::GK),
            2 => Some(Position::DEF),
            3 => Some(Position::MID),
            4 => Some(Position::FWD),
            _ => None,
        };

        Player::new(
            self.form.parse::<f32>().unwrap(),
            self.chance_of_playing_next_round.unwrap_or(100.0) / 100.0,
            self.now_cost / 10.0,
            self.web_name.clone(),
            position.unwrap(),
            self.id,
            team,
            self.total_points,
            self.ep_next.parse::<f32>().unwrap(),
        )
    }
}

#[derive(Deserialize, Debug)]
struct APIEntryHistory {
    bank: f32,
}

#[derive(Deserialize, Debug)]
struct APIPick {
    element: u16,
}

#[derive(Deserialize, Debug)]
pub struct APISquad {
    picks: Vec<APIPick>,
    entry_history: APIEntryHistory,
}

pub fn get_full_sorted_player_list() -> Result<Vec<Player>, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(FANTASY_API_URL)?;
    let resp_json: PlayerResponse = serde_json::from_str(&resp.text()?)?;

    let mut result: Vec<Player> = resp_json.elements.iter().map(|p| p.to_player()).collect();
    result.sort_by(|b, a| a.metric().partial_cmp(&b.metric()).unwrap());
    Ok(result)
}

pub fn get_my_squad(
    user_id: u32,
    current_gameweek: u8,
    full_player_list: &Vec<Player>,
) -> Result<Squad, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(format!(
        "https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/",
        user_id, current_gameweek
    ))?;
    let resp_json: APISquad = serde_json::from_str(&resp.text()?)?;
    let mut current_squad = Squad::new(f32::INFINITY);
    for player in full_player_list {
        for pick in &resp_json.picks {
            if pick.element == player.id {
                current_squad
                    .try_add_player(&player)
                    .expect("error adding players from api to squad");
                break;
            }
        }
    }
    current_squad.set_max_cost(resp_json.entry_history.bank / 10.0 + current_squad.total_cost());
    Ok(current_squad)
}

fn log_in_error(reason: &str) -> Result<(), Box<dyn std::error::Error>> {
    Err(Box::new(Error::new(ErrorKind::Other, format!("Error logging in: {}", reason))))
}
pub fn log_in(email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let params = [
        ("login", email),
        ("password", password),
        ("redirect_uri", "https://fantasy.premierleague.com/"),
        ("app", "plfpl-web"),
    ];

    let client = reqwest::blocking::Client::new();
    let response = client.post(LOG_IN_URL).form(&params).send()?;

    if response.status().is_success() {
        let headers = response.url();
        let pairs: HashMap<_, _> = headers.query_pairs().into_owned().collect();
        match pairs.get("state") {
            Some(state) => {
                match state.as_str() {
                    "success" => Ok(()),
                    "fail" => {
                        match pairs.get("reason") {
                            Some(reason) => log_in_error(reason),
                            None => Ok(())
                        }
                    },
                    _ => log_in_error(&format!("type of state {} was not understood", state))
                }
            }
            None => log_in_error("got a response, but no state")
        }
    }
    else{
        log_in_error("failed to get response from URL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_my_squad() {
        let list = get_full_sorted_player_list().unwrap();
        let squad = get_my_squad(2367749, 1, &list).unwrap();
        let copy = squad.clone();
        print!("{}", squad.changed_squad(&copy));
    }

    #[test]
    fn test_log_in() {
        let bad_call_result = log_in("polortiz4@hotmail.com", "pasfsword");
        assert_eq!(
            "Error logging in: credentials",
            bad_call_result.unwrap_err().to_string()
        );
        log_in("polortiz4@hotmail.com", "password").unwrap();
    }
}
