use crate::player::{Player, Position};
use crate::team::Team;
extern crate ordered_float;
use ordered_float::OrderedFloat;

use std::fmt;

const N_GK: usize = 2;
const N_DEF: usize = 5;
const N_MID: usize = 5;
const N_FWD: usize = 3;
const EPSILON: f32 = 1e-4;
const MAX_PLAYERS_PER_TEAM: usize = 3;

#[derive(Debug, PartialEq)]
enum AddPlayerError {
    TooExpensiveError(String),
    PositionFull(String),
    TeamsSpotFull(String),
    DuplicatePlayer(String),
}

struct SquadNotFull;

#[derive(Debug)]
pub struct Squad {
    max_cost: f32,
    goalkeepers: Vec<Player>,
    defenders: Vec<Player>,
    midfielders: Vec<Player>,
    strikers: Vec<Player>,

    players: Vec<Player>,
}

impl PartialEq for Squad {
    fn eq(&self, other: &Squad) -> bool {
        self.organized_players() == other.organized_players()
    }
}
impl Clone for Squad {
    fn clone(&self) -> Squad {
        let mut copy = Squad::new(self.max_cost());
        for player in &self.players {
            copy.try_add_player(&player).unwrap();
        }
        copy
    }
}
impl Squad {
    pub fn new(max_cost: f32) -> Squad {
        Squad::new_with_size(max_cost, N_GK, N_DEF, N_MID, N_FWD)
    }

    #[allow(dead_code)]
    pub fn new_with_size(
        max_cost: f32,
        n_gk: usize,
        n_def: usize,
        n_mid: usize,
        n_fwd: usize,
    ) -> Squad {
        Squad {
            max_cost: max_cost,
            goalkeepers: Vec::with_capacity(n_gk),
            defenders: Vec::with_capacity(n_def),
            midfielders: Vec::with_capacity(n_mid),
            strikers: Vec::with_capacity(n_fwd),
            players: Vec::with_capacity(n_gk + n_def + n_mid + n_fwd),
        }
    }
    pub fn max_cost(&self) -> f32 {
        self.max_cost
    }
    pub fn set_max_cost(&mut self, max_cost: f32) {
        self.max_cost = max_cost + EPSILON;
    }
    pub fn total_cost(&self) -> f32 {
        self.players.iter().map(|p| p.price).sum()
    }

    pub fn leftover_money(&self) -> f32 {
        self.max_cost() - self.total_cost()
    }

    fn sort_players(&mut self) {
        self.goalkeepers
            .sort_by(|a, b| b.metric().partial_cmp(&a.metric()).unwrap());
        self.defenders
            .sort_by(|a, b| b.metric().partial_cmp(&a.metric()).unwrap());
        self.midfielders
            .sort_by(|a, b| b.metric().partial_cmp(&a.metric()).unwrap());
        self.strikers
            .sort_by(|a, b| b.metric().partial_cmp(&a.metric()).unwrap());
    }

    pub fn remove_player(&mut self, player: &Player) {
        if self.has_player(&player) {
            match player.position {
                Position::GK => self.goalkeepers.retain(|p| p != player),
                Position::DEF => self.defenders.retain(|p| p != player),
                Position::MID => self.midfielders.retain(|p| p != player),
                Position::FWD => self.strikers.retain(|p| p != player),
            }
            self.players.retain(|p| p != player);
        }
    }

    pub fn captain(&self) -> Player {
        self.players
            .iter()
            .max_by_key(|p| OrderedFloat(p.metric()))
            .unwrap()
            .clone()
    }

    fn players_from_team(&self, team: Team) -> usize {
        self.players.iter().filter(|&p| p.team == team).count()
    }

    fn has_player(&self, player: &Player) -> bool {
        self.players.iter().filter(|&p| p == player).count() > 0
    }

    pub fn sort_and_organized_players(&mut self) -> Vec<Player> {
        self.sort_players();
        self.organized_players()
    }
    pub fn organized_players(&self) -> Vec<Player> {
        let mut s_copy = self.clone();
        s_copy.sort_players();
        s_copy.midfielders.append(&mut s_copy.strikers);
        s_copy.defenders.append(&mut s_copy.midfielders);
        s_copy.goalkeepers.append(&mut s_copy.defenders);
        s_copy.goalkeepers
    }

    pub fn number_of_changes(&self, other: &Squad) -> usize {
        self.players
            .iter()
            .filter(|&p| !other.has_player(p))
            .count()
    }
    pub fn changes_from(&self, other: &Squad) -> String{
        let mut unique_from_self = Squad::new(1000.0);
        let mut unique_from_other = Squad::new(1000.0);
        for (my_player, other_player) in self.players.iter().zip(other.players.iter()){
            if !other.has_player(my_player) {
                unique_from_self.try_add_player(my_player).unwrap();
            }
            if !self.has_player(other_player) {
                unique_from_other.try_add_player(other_player).unwrap();
            }
        }
        
        let mut result: String = String::new();
        for (mine, other) in unique_from_self.organized_players().iter().zip(unique_from_other.organized_players().iter()){
            assert!(mine.position == other.position);
            result.push_str(&format!("Out: {:?} <-----------> In: {:?}\n", other, mine));
        }
        result
    }
    // Return a list of the starters from a given list
    pub fn position_starters(&self, position: Position, n_starters: usize) -> Vec<Player> {
        let player_list = match position {
            Position::GK => &self.goalkeepers,
            Position::DEF => &self.defenders,
            Position::MID => &self.midfielders,
            Position::FWD => &self.strikers,
        };
        let mut ans = player_list.clone();
        ans.sort_by(|a, b| b.metric().partial_cmp(&a.metric()).unwrap());
        ans[..n_starters].to_vec()
    }
    pub fn positions_full(&self) -> bool {
        self.players.len()
            == self.goalkeepers.capacity()
                + self.defenders.capacity()
                + self.midfielders.capacity()
                + self.strikers.capacity()
    }
    pub fn total_metric(&self, captain_multiplier: f32) -> f32 {
        self.players
            .iter()
            .map(|p| {
                if *p == self.captain() {
                    captain_multiplier * p.metric()
                } else {
                    p.metric()
                }
            })
            .sum()
    }

    pub fn try_add_player(&mut self, player: &Player) -> Result<(), AddPlayerError> {
        // Check team capacity
        if self.players_from_team(player.team) >= MAX_PLAYERS_PER_TEAM {
            return Err(AddPlayerError::TeamsSpotFull(format!(
                "Too many players from team: {}. Already have: {}",
                player.team.to_string(),
                self.players_from_team(player.team)
            )));
        }

        // Check for duplication
        if self.has_player(&player) {
            return Err(AddPlayerError::DuplicatePlayer(format!(
                "Player {} is already in the squad",
                player.name
            )));
        }

        // Check for funds
        if player.price + self.total_cost() > self.max_cost() {
            return Err(AddPlayerError::TooExpensiveError(format!(
                "Not enough funds to add {} who costs {} and you have {:.2} available.",
                player.name,
                player.price,
                self.max_cost() - self.total_cost()
            )));
        }

        // Check position capacity
        let player_list = match player.position {
            Position::GK => &self.goalkeepers,
            Position::DEF => &self.defenders,
            Position::MID => &self.midfielders,
            Position::FWD => &self.strikers,
        };
        if player_list.len() > player_list.capacity() {
            panic!()
        }
        if player_list.len() == player_list.capacity() {
            return Err(AddPlayerError::PositionFull(format!(
                "Cannot add {}, as there are too many {}: {}",
                player.name,
                player.position,
                player_list.len()
            )));
        }

        // Add Player
        self.force_add_player(player);
        Ok(())
    }
    fn force_add_player(&mut self, player: &Player) {
        self.players.push(player.clone());
        match player.position {
            Position::GK => self.goalkeepers.push(player.clone()),
            Position::DEF => self.defenders.push(player.clone()),
            Position::MID => self.midfielders.push(player.clone()),
            Position::FWD => self.strikers.push(player.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hazard_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Hazard"),
            Position::MID,
            10,
            Team::new(6),
            5,
        )
    }
    fn gerrard_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Gerrard"),
            Position::MID,
            11,
            Team::new(9),
            1,
        )
    }
    fn scholes_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Scholes"),
            Position::MID,
            12,
            Team::new(10),
            6,
        )
    }
    fn lampard_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Lampard"),
            Position::MID,
            1,
            Team::new(6),
            5,
        )
    }
    fn pablo_player() -> Player {
        Player::new(
            7.1,
            0.8,
            1.0,
            String::from("Ortiz"),
            Position::MID,
            3,
            Team::new(6),
            4,
        )
    }
    fn buffon_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Buffon"),
            Position::GK,
            13,
            Team::new(12),
            6,
        )
    }
    fn maldini_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Maldini"),
            Position::DEF,
            14,
            Team::new(13),
            6,
        )
    }
    fn drogba_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Drogba"),
            Position::FWD,
            15,
            Team::new(6),
            7,
        )
    }
    fn rooney_player() -> Player {
        Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Rooney"),
            Position::FWD,
            16,
            Team::new(9),
            5,
        )
    }

    fn six_p_squad() -> Squad {
        let mut squad = Squad::new(100.0);
        squad.try_add_player(&drogba_player()).unwrap();
        squad.try_add_player(&lampard_player()).unwrap();
        squad.try_add_player(&scholes_player()).unwrap();
        squad.try_add_player(&maldini_player()).unwrap();
        squad.try_add_player(&buffon_player()).unwrap();
        squad.try_add_player(&gerrard_player()).unwrap();
        squad
    }

    #[test]
    fn test_total_metric(){
        let six_squad = six_p_squad();
        assert_eq!(drogba_player(), six_squad.captain());
        assert_eq!(38.0, six_squad.total_metric(2.0));
    }
    #[test]
    fn test_n_changes() {
        let six_squad = six_p_squad();
        let mut alt_squad = Squad::new(100.0);
        alt_squad.try_add_player(&rooney_player()).unwrap();
        alt_squad.try_add_player(&lampard_player()).unwrap();
        alt_squad.try_add_player(&scholes_player()).unwrap();
        alt_squad.try_add_player(&maldini_player()).unwrap();
        alt_squad.try_add_player(&buffon_player()).unwrap();
        alt_squad.try_add_player(&pablo_player()).unwrap();
        assert_eq!(2, alt_squad.number_of_changes(&six_squad));
        assert_eq!(2, six_squad.number_of_changes(&alt_squad));

        let expected = String::from("Out: Gerrard <-----------> In: Ortiz\nOut: Drogba <-----------> In: Rooney\n");
        assert_eq!(expected, alt_squad.changes_from(&six_squad));
    }
    #[test]
    fn test_copy() {
        let squad = six_p_squad();
        let s_copy = squad.clone();
        assert_eq!(squad, s_copy);
    }
    #[test]
    fn test_organized_players() {
        let mut squad = Squad::new(100.0);
        squad.try_add_player(&drogba_player()).unwrap();
        squad.try_add_player(&lampard_player()).unwrap();
        squad.try_add_player(&scholes_player()).unwrap();
        squad.try_add_player(&maldini_player()).unwrap();
        squad.try_add_player(&buffon_player()).unwrap();
        squad.try_add_player(&gerrard_player()).unwrap();
        let expected = vec![
            buffon_player(),
            maldini_player(),
            scholes_player(),
            lampard_player(),
            gerrard_player(),
            drogba_player(),
        ];
        assert_eq!(expected, squad.organized_players());
    }
    #[test]
    fn test_mid_starters() {
        let mut squad = Squad::new(100.0);
        squad.try_add_player(&lampard_player()).unwrap();
        squad.try_add_player(&gerrard_player()).unwrap();
        squad.try_add_player(&hazard_player()).unwrap();
        squad.try_add_player(&scholes_player()).unwrap();
        let expected = vec![scholes_player(), lampard_player(), hazard_player()];
        assert_eq!(expected, squad.position_starters(Position::MID, 3));
    }
    #[test]
    fn test_captain() {
        let mut squad = Squad::new(100.0);
        let player = lampard_player();
        squad.force_add_player(&player);
        let player = pablo_player();
        squad.force_add_player(&player);
        assert_eq!(squad.captain(), lampard_player());
    }
    #[test]
    fn test_max_cost() {
        let mut squad = Squad::new(100.0);
        assert_eq!(100.0, squad.max_cost());
        squad.set_max_cost(95.0);
        assert_eq!(95.0 + EPSILON, squad.max_cost());
    }

    #[test]
    fn test_remove_player() {
        let mut squad = Squad::new(100.0);
        let player = lampard_player();
        squad.try_add_player(&player).unwrap();
        let player = pablo_player();
        assert_eq!(squad.players_from_team(Team::new(6)), 1);
        squad.try_add_player(&player).unwrap();
        assert_eq!(squad.players_from_team(Team::new(6)), 2);
        let player = Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Lampard"),
            Position::MID,
            1,
            Team::new(6),
            5,
        );
        squad.remove_player(&player);
        assert_eq!(squad.players_from_team(Team::new(6)), 1);
    }
    #[test]
    fn test_team_full() {
        let mut squad = Squad::new(100.0);
        let player = lampard_player();
        squad.force_add_player(&player);
        assert!(matches!(
            squad.try_add_player(&player),
            Err(AddPlayerError::DuplicatePlayer { .. })
        ));

        squad.force_add_player(&player);
        squad.force_add_player(&player);
        squad.force_add_player(&player);
        squad.force_add_player(&player);

        assert!(matches!(
            squad.try_add_player(&player),
            Err(AddPlayerError::TeamsSpotFull { .. })
        ));
    }
    #[test]
    fn test_position_capacity() {
        let mut squad = Squad::new(100.0);
        let player = lampard_player();
        squad.force_add_player(&player);
        squad.force_add_player(&player);
        squad.force_add_player(&player);

        let player = Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Ortiz"),
            Position::MID,
            3,
            Team::new(7),
            5,
        );
        squad.force_add_player(&player);
        squad.force_add_player(&player);
        let player = Player::new(
            7.2,
            0.8,
            0.1,
            String::from("Sibley"),
            Position::MID,
            5,
            Team::new(8),
            5,
        );
        assert!(matches!(
            squad.try_add_player(&player),
            Err(AddPlayerError::PositionFull { .. })
        ));
    }
    #[test]
    fn test_funds() {
        let mut squad = Squad::new(1.1);
        let player = lampard_player();
        squad.force_add_player(&player);

        let player = pablo_player();
        assert!(matches!(
            squad.try_add_player(&player),
            Err(AddPlayerError::TooExpensiveError { .. })
        ));

        let player = Player::new(
            7.2,
            0.8,
            0.1,
            String::from("Sibley"),
            Position::MID,
            5,
            Team::new(6),
            5,
        );
        let result = squad.try_add_player(&player);
        match result {
            Ok(_) => (),
            Err(error) => println!("{:?}", error),
        }
        assert!(!matches!(
            squad.try_add_player(&player),
            Err(AddPlayerError::TooExpensiveError { .. })
        ));
    }
    #[test]
    fn test_total_cost() {
        let mut squad = Squad::new(100.0);
        let player = lampard_player();
        squad.force_add_player(&player);
        assert_eq!(squad.total_cost(), 1.0);
        squad.force_add_player(&player);
        assert_eq!(squad.total_cost(), 2.0);
    }
    #[test]
    fn test_players_from_team() {
        let mut squad = Squad::new(100.0);
        let player = lampard_player();
        let player2 = Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Ortiz"),
            Position::MID,
            2,
            Team::new(6),
            5,
        );
        let player3 = Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Sibley"),
            Position::MID,
            3,
            Team::new(7),
            5,
        );
        assert_eq!(squad.players_from_team(Team::new(6)), 0);
        assert_eq!(squad.players_from_team(Team::new(7)), 0);

        squad.force_add_player(&player);
        assert_eq!(squad.players_from_team(Team::new(6)), 1);

        squad.force_add_player(&player2);
        assert_eq!(squad.players_from_team(Team::new(6)), 2);

        squad.force_add_player(&player3);
        assert_eq!(squad.players_from_team(Team::new(6)), 2);
        assert_eq!(squad.players_from_team(Team::new(7)), 1);
    }
}
