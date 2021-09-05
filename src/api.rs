use crate::player::{Player, Position};
use crate::squad::Squad;
use crate::team::Team;
use reqwest::cookie::Cookie;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;

use serde::ser::{SerializeMap, SerializeTuple};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::{Error, ErrorKind};

const FANTASY_API_URL: &str = "https://fantasy.premierleague.com/api/bootstrap-static/";
const LOG_IN_URL: &str = "https://users.premierleague.com/accounts/login/";
const TRANSFER_URL: &str = "https://fantasy.premierleague.com/api/transfers/";

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
    Err(Box::new(Error::new(
        ErrorKind::Other,
        format!("Error logging in: {}", reason),
    )))
}
pub fn log_in(
    client: &reqwest::blocking::Client,
    email: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = [
        ("login", email),
        ("password", password),
        ("redirect_uri", "https://fantasy.premierleague.com/"),
        ("app", "plfpl-web"),
    ];

    let response = client.post(LOG_IN_URL).form(&params).send()?;

    if response.status().is_success() {
        let pairs: HashMap<_, _> = response.url().query_pairs().into_owned().collect();
        match pairs.get("state") {
            Some(state) => match state.as_str() {
                "success" => Ok(()),
                "fail" => match pairs.get("reason") {
                    Some(reason) => log_in_error(reason),
                    None => log_in_error("failed state for unknown reason"),
                },
                _ => log_in_error(&format!("type of state ({}) was not understood", state)),
            },
            None => log_in_error("got a response, but no state"),
        }
    } else {
        log_in_error("failed to get response from website")
    }
}

fn _single_transfer_payload(player_in: &Player, player_out: &Player) -> Transfer {
    Transfer {
        element_in: player_in.id.to_string(),
        element_out: player_out.id.to_string(),
        purchase_price: ((player_in.price * 10.0) as u32).to_string(),
        selling_price: ((player_out.price * 10.0) as u32).to_string(),
    }
}

#[derive(Serialize, Debug)]
struct Transfer {
    element_in: String,
    element_out: String,
    purchase_price: String,
    selling_price: String,
}

fn _transfer_payload(
    players_out: Vec<Player>,
    players_in: Vec<Player>,
    user_id: u32,
    wildcard: bool,
    free_hit: bool,
    gameweek: u8,
) -> TPI {
    let mut payload = TPI {
        confirmed: "false".to_string(),
        event: gameweek.to_string(),
        entry: user_id.to_string(),
        transfers: Vec::new(),
        wildcard: wildcard.to_string(),
        freehit: free_hit.to_string(),
    };
    for (player_in, player_out) in players_in.iter().zip(players_out.iter()) {
        payload
            .transfers
            .push(_single_transfer_payload(player_in, player_out));
    }
    payload
}

#[derive(Serialize, Debug)]
struct TPI {
    confirmed: String,
    event: String,
    entry: String,
    transfers: Vec<Transfer>,
    wildcard: String,
    freehit: String,
}

pub fn transfer(
    client: &reqwest::blocking::Client,
    players_out: Vec<Player>,
    players_in: Vec<Player>,
    user_id: u32,
    wildcard: bool,
    free_hit: bool,
    gameweek: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = _transfer_payload(
        players_out,
        players_in,
        user_id,
        wildcard,
        free_hit,
        gameweek,
    );

    let response = client
        .post(TRANSFER_URL)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("X-Requested-With", "XMLHttpRequest")
        .header(
            "Referer",
            "https://fantasy.premierleague.com/a/squad/transfers",
        )
        .json(&params)
        .send()?;

    if response.status().is_success() {
        Ok(())
    } else {
        Session::transfer_error(&response.text()?)
    }
}

struct Session {
    client: reqwest::blocking::Client,
    user_id: u32,
    last_gameweek: u8,
}

impl Session {
    pub fn new(user_id: u32, gameweek: u8) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Session {
            client: reqwest::blocking::Client::builder()
                .cookie_store(true)
                .build()?,
                user_id: user_id,
                last_gameweek: gameweek,
        })
    }
    pub fn log_in(&self, email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        crate::api::log_in(&self.client, email, password)
    }

    fn transfer_error(reason: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err(Box::new(Error::new(
            ErrorKind::Other,
            format!("Error requesting transfer: {}", reason),
        )))
    }
    pub fn transfer(
        &self,
        players_out: Vec<Player>,
        players_in: Vec<Player>,
        wildcard: bool,
        free_hit: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::api::transfer(
            &self.client,
            players_out,
            players_in,
            self.user_id,
            wildcard,
            free_hit,
            self.last_gameweek,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer() {
        let mut out_squad = Squad::new(100.0);
        let mut in_squad = Squad::new(100.0);

        if false {
            out_squad.try_add_player(&salah_player()).unwrap();
            in_squad.try_add_player(&pogba_player()).unwrap();
        } else {
            in_squad.try_add_player(&salah_player()).unwrap();
            out_squad.try_add_player(&pogba_player()).unwrap();
        }

        let out_copy = out_squad.clone();
        let in_copy = in_squad.clone();

        let session = Session::new(7597109, 4).unwrap();
        session.log_in("polortiz4@hotmail.com", "password").unwrap();

        let res = session.transfer(out_copy.players, in_copy.players, false, false);
        res.unwrap();
    }

    #[test]
    fn test_get_my_squad() {
        let list = get_full_sorted_player_list().unwrap();
        let squad = get_my_squad(2367749, 1, &list).unwrap();
        let copy = squad.clone();
        print!("{}", squad.changed_squad(&copy));
    }

    #[test]
    fn test_log_in() {
        let mut client = reqwest::blocking::Client::new();
        let bad_call_result = log_in(&mut client, "polortiz4@hotmail.com", "pasfsword");
        assert_eq!(
            "Error logging in: credentials",
            bad_call_result.unwrap_err().to_string()
        );
        log_in(&mut client, "polortiz4@hotmail.com", "password").unwrap();
    }

    fn pogba_player() -> Player {
        Player::new(
            1.0,
            1.0,
            7.7,
            String::from("Pogba"),
            Position::MID,
            272,
            Team::new(6),
            1,
            1.0,
        )
    }

    fn salah_player() -> Player {
        Player::new(
            1.0,
            1.0,
            12.5,
            String::from("Salah"),
            Position::MID,
            233,
            Team::new(6),
            1,
            1.0,
        )
    }
}
