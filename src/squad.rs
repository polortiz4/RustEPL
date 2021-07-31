use crate::player::{Player, Position};
use crate::team::Team;
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

    fn players_from_team(&self, team: Team) -> usize {
        self.players.iter().filter(|&p| p.team == team).count()
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
        if self.players.iter().filter(|&p| p == player).count() > 0 {
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

    #[test]
    fn test_max_cost() {
        let mut squad = Squad::new(100.0);
        assert_eq!(100.0, squad.max_cost());
        squad.set_max_cost(95.0);
        assert_eq!(95.0 + EPSILON, squad.max_cost());
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

        let player = Player::new(
            7.2,
            0.8,
            1.0,
            String::from("Ortiz"),
            Position::MID,
            3,
            Team::new(6),
            5,
        );
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
