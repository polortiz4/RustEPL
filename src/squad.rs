use crate::CAPTAIN_MULTIPLIER;
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
const POSSIBLE_LINEUPS: [&'static [usize; 4]; 8] = [
    &[1, 3, 4, 3],
    &[1, 4, 3, 3],
    &[1, 5, 2, 3],
    &[1, 3, 5, 2],
    &[1, 4, 4, 2],
    &[1, 5, 3, 2],
    &[1, 5, 4, 1],
    &[1, 4, 5, 1],
];

#[derive(Debug, PartialEq)]
pub enum AddPlayerError {
    TooExpensiveError(String),
    PositionFull(String),
    TeamsSpotFull(String),
    DuplicatePlayer(String),
}

#[derive(Debug)]
pub struct Squad {
    max_cost: f32,
    goalkeepers: Vec<Player>,
    defenders: Vec<Player>,
    midfielders: Vec<Player>,
    strikers: Vec<Player>,

    pub players: Vec<Player>,
}

impl fmt::Display for Squad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Squad with Metric: {:.2}, cost: {:.2}",
            self.total_metric(CAPTAIN_MULTIPLIER),
            self.total_cost()
        )
    }
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
            copy.force_add_player(&player);
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

    pub fn sort_players(&mut self) {
        self.goalkeepers.sort_by(|a, b| {
            b.metric()
                .partial_cmp(&a.metric())
                .expect("Error sorting players")
        });
        self.defenders.sort_by(|a, b| {
            b.metric()
                .partial_cmp(&a.metric())
                .expect("Error sorting players")
        });
        self.midfielders.sort_by(|a, b| {
            b.metric()
                .partial_cmp(&a.metric())
                .expect("Error sorting players")
        });
        self.strikers.sort_by(|a, b| {
            b.metric()
                .partial_cmp(&a.metric())
                .expect("Error sorting players")
        });
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
            .expect("Failed to sort the metrics for a captain")
            .clone()
    }

    fn players_from_team(&self, team: Team) -> usize {
        self.players.iter().filter(|&p| p.team == team).count()
    }

    pub fn has_player(&self, player: &Player) -> bool {
        self.players.contains(player)
        // self.players.iter().filter(|&p| p == player).count() > 0
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

    pub fn changed_squad(&self, current_squad: &Squad) -> String {
        let mut result: String = String::from("\n\nChanged Squad:\n  Lineup:  ");
        for player in self.best_starter_lineup().players {
            result.push_str(&format!(" {:?}", player));
        }
        result.push_str("\n  Bench:   ");
        for player in self.bench().players {
            result.push_str(&format!(" {:?}", player));
        }
        result.push_str(&format!(
            "\n  Captain:  {:?}, metric: {}",
            self.captain(),
            self.captain().metric()
        ));
        result.push_str("\n\n  Changes needed:\n");
        result.push_str(&format!("{}\n", self.changes_from(current_squad)));
        result
    }
    pub fn number_of_changes(&self, other: &Squad) -> usize {
        self.players
            .iter()
            .filter(|&p| !other.has_player(p))
            .count()
    }
    pub fn changes_from(&self, other: &Squad) -> String {
        let mut unique_from_self = Squad::new(1000.0);
        let mut unique_from_other = Squad::new(1000.0);
        for (my_player, other_player) in self.players.iter().zip(other.players.iter()) {
            if !other.has_player(my_player) {
                unique_from_self.force_add_player(my_player);
            }
            if !self.has_player(other_player) {
                unique_from_other.force_add_player(other_player);
            }
        }
        let mut result: String = String::new();
        for (mine, other) in unique_from_self
            .sort_and_organized_players()
            .iter()
            .zip(unique_from_other.sort_and_organized_players().iter())
        {
            assert!(mine.position == other.position);
            result.push_str(&format!("    Out: {:?}, {} <-----------> In: {:?}, {}\n", other, other.metric(), mine, mine.metric()));
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
        ans.sort_by(|a, b| {
            b.metric()
                .partial_cmp(&a.metric())
                .expect("player metrics could not be compared")
        });
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

    pub fn bench(&self) -> Squad {
        let mut bench = Squad::new(self.max_cost());
        let starters = self.best_starter_lineup();
        self.players
            .iter()
            .filter(|p| !starters.has_player(p))
            .for_each(|p| bench.force_add_player(&p));
        bench
    }
    pub fn best_starter_lineup(&self) -> Squad {
        assert_eq!(
            vec![
                self.goalkeepers.capacity(),
                self.defenders.capacity(),
                self.midfielders.capacity(),
                self.strikers.capacity()
            ],
            vec![N_GK, N_DEF, N_MID, N_FWD]
        );
        let mut best_lineup: Option<Squad> = None;
        let mut best_metric = 0.0;
        for lineup in &POSSIBLE_LINEUPS {
            let mut starting_squad = Squad::new(self.max_cost());

            let gks = self.position_starters(Position::GK, lineup[0]);
            let defs = self.position_starters(Position::DEF, lineup[1]);
            let mids = self.position_starters(Position::MID, lineup[2]);
            let fwds = self.position_starters(Position::FWD, lineup[3]);

            gks.iter()
                .for_each(|gk| starting_squad.force_add_player(&gk));
            defs.iter()
                .for_each(|def| starting_squad.force_add_player(&def));
            mids.iter()
                .for_each(|mid| starting_squad.force_add_player(&mid));
            fwds.iter()
                .for_each(|fwd| starting_squad.force_add_player(&fwd));

            if best_lineup.is_none()
                || starting_squad.total_metric(CAPTAIN_MULTIPLIER) > best_metric
            {
                best_metric = starting_squad.total_metric(CAPTAIN_MULTIPLIER);
                best_lineup = Some(starting_squad);
            }
        }
        best_lineup.expect("best_lineup failed to be created")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pablo_player() -> Player {
        Player::new(
            1.0,
            1.0,
            1.0,
            String::from("Ortiz"),
            Position::MID,
            3,
            Team::new(6),
            1,
        )
    }

    fn buffon_player() -> Player {
        Player::new(
            2.0,
            1.0,
            1.0,
            String::from("Buffon"),
            Position::GK,
            13,
            Team::new(12),
            2,
        )
    }
    fn karius_player() -> Player {
        Player::new(
            3.0,
            1.0,
            1.0,
            String::from("Karius"),
            Position::GK,
            21,
            Team::new(12),
            3,
        )
    }

    fn maldini_player() -> Player {
        Player::new(
            4.0,
            1.0,
            1.0,
            String::from("Maldini"),
            Position::DEF,
            14,
            Team::new(13),
            4,
        )
    }
    fn terry_player() -> Player {
        Player::new(
            5.0,
            1.0,
            1.0,
            String::from("Terry"),
            Position::DEF,
            18,
            Team::new(6),
            5,
        )
    }
    fn vidic_player() -> Player {
        Player::new(
            6.0,
            1.0,
            1.0,
            String::from("Vidic"),
            Position::DEF,
            19,
            Team::new(16),
            6,
        )
    }
    fn johnson_player() -> Player {
        Player::new(
            7.0,
            1.0,
            1.0,
            String::from("Johnson"),
            Position::DEF,
            29,
            Team::new(17),
            7,
        )
    }
    fn cahill_player() -> Player {
        Player::new(
            8.0,
            1.0,
            1.0,
            String::from("Cahill"),
            Position::DEF,
            20,
            Team::new(17),
            8,
        )
    }

    fn hazard_player() -> Player {
        Player::new(
            9.0,
            1.0,
            1.0,
            String::from("Hazard"),
            Position::MID,
            10,
            Team::new(6),
            9,
        )
    }
    fn gerrard_player() -> Player {
        Player::new(
            10.0,
            1.0,
            1.0,
            String::from("Gerrard"),
            Position::MID,
            11,
            Team::new(9),
            10,
        )
    }
    fn scholes_player() -> Player {
        Player::new(
            11.0,
            1.0,
            1.0,
            String::from("Scholes"),
            Position::MID,
            12,
            Team::new(10),
            11,
        )
    }
    fn lampard_player() -> Player {
        Player::new(
            12.0,
            1.0,
            1.0,
            String::from("Lampard"),
            Position::MID,
            1,
            Team::new(6),
            12,
        )
    }
    fn fabregas_player() -> Player {
        Player::new(
            13.0,
            1.0,
            1.0,
            String::from("Fabregas"),
            Position::MID,
            31,
            Team::new(1),
            13,
        )
    }
    fn bale_player() -> Player {
        Player::new(
            14.0,
            1.0,
            1.0,
            String::from("Bale"),
            Position::MID,
            22,
            Team::new(3),
            14,
        )
    }
    fn drogba_player() -> Player {
        Player::new(
            15.0,
            1.0,
            1.0,
            String::from("Drogba"),
            Position::FWD,
            15,
            Team::new(6),
            15,
        )
    }
    fn rooney_player() -> Player {
        Player::new(
            16.0,
            1.0,
            1.0,
            String::from("Rooney"),
            Position::FWD,
            16,
            Team::new(9),
            16,
        )
    }
    fn suarez_player() -> Player {
        Player::new(
            17.0,
            1.0,
            1.0,
            String::from("Suarez"),
            Position::FWD,
            17,
            Team::new(10),
            17,
        )
    }
    fn adebayor_player() -> Player {
        Player::new(
            17.0,
            1.0,
            1.0,
            String::from("Adebayor"),
            Position::FWD,
            40,
            Team::new(9),
            17,
        )
    }
    fn full_squad() -> Squad {
        let mut squad = Squad::new(100.0);
        squad.try_add_player(&maldini_player()).unwrap();
        squad.try_add_player(&karius_player()).unwrap();
        squad.try_add_player(&suarez_player()).unwrap();
        squad.try_add_player(&cahill_player()).unwrap();
        squad.try_add_player(&drogba_player()).unwrap();
        squad.try_add_player(&buffon_player()).unwrap();
        squad.try_add_player(&terry_player()).unwrap();
        squad.try_add_player(&gerrard_player()).unwrap();
        squad.try_add_player(&fabregas_player()).unwrap();
        squad.try_add_player(&scholes_player()).unwrap();
        squad.try_add_player(&bale_player()).unwrap();
        squad.try_add_player(&lampard_player()).unwrap();
        squad.try_add_player(&vidic_player()).unwrap();
        squad.try_add_player(&johnson_player()).unwrap();
        squad.try_add_player(&rooney_player()).unwrap();
        squad
    }
    fn full_squad_starters() -> Squad {
        let mut squad = Squad::new(100.0);
        squad.try_add_player(&karius_player()).unwrap();
        squad.try_add_player(&suarez_player()).unwrap();
        squad.try_add_player(&rooney_player()).unwrap();
        squad.try_add_player(&drogba_player()).unwrap();
        squad.try_add_player(&bale_player()).unwrap();
        squad.try_add_player(&fabregas_player()).unwrap();
        squad.try_add_player(&lampard_player()).unwrap();
        squad.try_add_player(&scholes_player()).unwrap();
        squad.try_add_player(&cahill_player()).unwrap();
        squad.try_add_player(&johnson_player()).unwrap();
        squad.try_add_player(&vidic_player()).unwrap();
        squad
    }
    fn full_squad_bench() -> Squad {
        let mut squad = Squad::new(100.0);
        squad.try_add_player(&buffon_player()).unwrap();
        squad.try_add_player(&maldini_player()).unwrap();
        squad.try_add_player(&terry_player()).unwrap();
        squad.try_add_player(&gerrard_player()).unwrap();
        squad
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
    fn test_best_starting_lineup_and_bench() {
        let full_squad = full_squad();
        let expected_starters = full_squad_starters();
        let expected_bench = full_squad_bench();
        assert_eq!(full_squad.best_starter_lineup(), expected_starters);
        assert_eq!(full_squad.bench(), expected_bench);
    }
    #[test]
    fn test_total_metric() {
        let six_squad = six_p_squad();
        assert_eq!(drogba_player(), six_squad.captain());
        assert_eq!(69.0, six_squad.total_metric(CAPTAIN_MULTIPLIER));
    }
    #[test]
    fn test_changes() {
        let full_squad = full_squad();
        let mut alt_squad = Squad::new(100.0);
        alt_squad.try_add_player(&maldini_player()).unwrap();
        alt_squad.try_add_player(&karius_player()).unwrap();
        alt_squad.try_add_player(&suarez_player()).unwrap();
        alt_squad.try_add_player(&cahill_player()).unwrap();
        alt_squad.try_add_player(&adebayor_player()).unwrap();
        alt_squad.try_add_player(&buffon_player()).unwrap();
        alt_squad.try_add_player(&terry_player()).unwrap();
        alt_squad.try_add_player(&pablo_player()).unwrap();
        alt_squad.try_add_player(&fabregas_player()).unwrap();
        alt_squad.try_add_player(&scholes_player()).unwrap();
        alt_squad.try_add_player(&bale_player()).unwrap();
        alt_squad.try_add_player(&lampard_player()).unwrap();
        alt_squad.try_add_player(&vidic_player()).unwrap();
        alt_squad.try_add_player(&johnson_player()).unwrap();
        alt_squad.try_add_player(&rooney_player()).unwrap();
        assert_eq!(2, alt_squad.number_of_changes(&full_squad));
        assert_eq!(2, full_squad.number_of_changes(&alt_squad));

        let expected = String::from(
            "Out: Gerrard <-----------> In: Ortiz\nOut: Drogba <-----------> In: Adebayor\n",
        );
        assert_eq!(expected, alt_squad.changes_from(&full_squad));
        let expected = String::from("\nChanged Squad Lineup: Karius Cahill Johnson Vidic Bale Fabregas Lampard Scholes Suarez Adebayor Rooney\nChanged Squad Bench: Maldini Buffon Terry Ortiz\nCaptain: Adebayor, metric: 17\nChanges needed for Changes Squad:\nOut: Gerrard <-----------> In: Ortiz\nOut: Drogba <-----------> In: Adebayor\n\n");
        print!("{}", expected);
        assert_eq!(expected, alt_squad.changed_squad(&full_squad));
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
            lampard_player(),
            scholes_player(),
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
        let expected = vec![lampard_player(), scholes_player(), gerrard_player()];
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
            1.0,
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
            1.0,
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
            1.0,
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
            1.0,
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
        assert_eq!(squad.total_cost(), CAPTAIN_MULTIPLIER);
    }
    #[test]
    fn test_players_from_team() {
        let mut squad = Squad::new(100.0);
        let player = lampard_player();
        let player2 = Player::new(
            7.2,
            1.0,
            1.0,
            String::from("Ortiz"),
            Position::MID,
            2,
            Team::new(6),
            5,
        );
        let player3 = Player::new(
            7.2,
            1.0,
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
