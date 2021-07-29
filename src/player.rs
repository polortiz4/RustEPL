use crate::team::Team;
use std::fmt;

#[derive(Debug)]
pub enum Position {
    GK,
    DEF,
    MID,
    FWD,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Position::GK => write!(f, "GK"),
            Position::DEF => write!(f, "DEF"),
            Position::MID => write!(f, "MID"),
            Position::FWD => write!(f, "FWD"),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    form: f32,
    health: f32,
    price: f32,
    name: String,
    position: Position,
    id: u16,
    team: Team,
    metric: f32,
    total_points: u32,
}

impl Player {
    pub fn metric(&self) -> f32 {
        self.metric
    }
    pub fn update_metric(&mut self) {
        self.metric = self.total_points as f32;
        // self.metric = self.form * self.health;
    }
    pub fn new(
        form: f32,
        health: f32,
        price: f32,
        name: String,
        position: Position,
        id: u16,
        team: Team,
        points: u32,
    ) -> Player {
        let mut player = Player {
            form: form,
            health: health,
            price: price,
            name: name,
            position: position,
            id: id,
            team: team,
            metric: 0.0,
            total_points: points,
        };
        player.update_metric();
        player
    }
}
impl PartialEq for Player {
    fn eq(&self, other: &Player) -> bool {
        self.id == other.id
    }
}
impl Eq for Player {}

impl ToString for Player {
    fn to_string(&self) -> String {
        format!(
            "{}, form: {:.2}, price: {:.2}, position: {}, team: {}, id: {}, health: {:.2}, points: {}, metric: {:.2}",
            self.name,
            self.form,
            self.price,
            self.position,
            self.team.to_string(),
            self.id,
            self.health,
            self.metric,
            self.total_points,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player() {
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

        assert_eq!(player.form, 7.2);
        assert_eq!(player.health, 0.8);
        assert_eq!(player.price, 1.0);
        assert_eq!(player.name, "Lampard");
        assert_eq!(player.id, 1);
        assert_eq!(player.metric(), 5.0);
        assert_eq!(player.metric, 5.0);

        assert_eq!(
            player.to_string(),
            "Lampard, form: 7.20, price: 1.00, position: MID, team: Chelsea, id: 1, health: 0.80, points: 5, metric: 5"
        );

        let same_id_player = Player::new(
            7.2,
            1.0,
            1.1,
            String::from("Terry"),
            Position::DEF,
            1,
            Team::new(6),
            0,
        );
        assert_eq!(player, same_id_player);
    }
}
