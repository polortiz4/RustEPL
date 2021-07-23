use std::fmt;
#[derive(Debug)]
enum Position {
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

pub struct Player {
    form: f32,
    health: f32,
    price: f32,
    name: String,
    position: Position,
    id: f32,
    team: i32,
}


impl ToString for Player {
    fn to_string(&self) -> String {
        format!(
            "{}, form: {}, price: {}, position: {}, team: {}, id: {}, health: {}",
            self.name, self.form, self.price, self.position, self.team, self.id, self.health
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_player() {
        let player = Player {
            form: 1.0,
            health: 1.0,
            price: 1.1,
            name: String::from("Lampard"),
            position: Position::MID,
            id: 1.0,
            team: 1,
        };

        assert_eq!(player.form, 1.0);
        assert_eq!(player.health, 1.0);
        assert_eq!(player.price, 1.1);
        assert_eq!(player.name, "Lampard");

        assert_eq!(
            player.to_string(),
            "Lampard, form: 1.0, price: 1.1, position: MID, team: None, id: None, health: 1.0"
        );
    }
}
