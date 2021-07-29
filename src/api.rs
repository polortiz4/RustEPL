use crate::player::{Player, Position};
use crate::team::Team;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Response {
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
            self.chance_of_playing_next_round.unwrap_or(0.0),
            self.now_cost,
            self.web_name.clone(),
            position.unwrap(),
            self.id,
            team,
            self.total_points,
        )
    }
}

pub fn get_full_player_list() -> Result<Vec<Player>, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get("https://fantasy.premierleague.com/api/bootstrap-static/")?;
    let resp_json: Response = serde_json::from_str(&resp.text()?)?;

    let mut result: Vec<Player> = resp_json.elements.iter().map(|p| p.to_player()).collect();
    result.sort_by(|a, b| a.metric().partial_cmp(&b.metric()).unwrap());
    Ok(result)
}
