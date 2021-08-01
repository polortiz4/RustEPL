use crate::player::{Player, Position};
use crate::squad::Squad;
use crate::team::Team;
use serde::Deserialize;

const FANTASY_API_URL: &str = "https://fantasy.premierleague.com/api/bootstrap-static/";

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
    total_points: u32,
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
            self.chance_of_playing_next_round.unwrap_or(0.0) / 100.0,
            self.now_cost / 10.0,
            self.web_name.clone(),
            position.unwrap(),
            self.id,
            team,
            self.total_points,
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
    result.sort_by(|a, b| a.metric().partial_cmp(&b.metric()).unwrap());
    Ok(result)
}

pub fn get_my_squad(
    user_id: usize,
    current_gameweek: usize,
    full_player_list: Vec<Player>,
) -> Result<Squad, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(format!(
        "https://fantasy.premierleague.com/api/entry/{}/event/{}/picks/",
        user_id, current_gameweek
    ))?;
    let resp_json: APISquad = serde_json::from_str(&resp.text()?)?;
    let mut current_squad = Squad::new(100.0);
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
