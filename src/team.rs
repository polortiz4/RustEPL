#[derive(Debug)]
pub struct Team {
    idx: u8,
}

impl Team {
    pub fn new(idx: u8) -> Team {
        Team { idx: idx }
    }
}
impl ToString for Team {
    fn to_string(&self) -> String {
        match self.idx {
            1 => String::from("Arsenal"),
            2 => String::from("Aston Villa"),
            3 => String::from("Brentford"),
            4 => String::from("Brighton & Hove Albion"),
            5 => String::from("Burnley"),
            6 => String::from("Chelsea"),
            7 => String::from("Crystal Palace"),
            8 => String::from("Everton"),
            9 => String::from("Leeds United"),
            10 => String::from("Leicester City"),
            11 => String::from("Liverpool"),
            12 => String::from("Man City"),
            13 => String::from("Man Utd"),
            14 => String::from("Newcastle"),
            15 => String::from("Norwich City"),
            16 => String::from("Southampton"),
            17 => String::from("Tottenham"),
            18 => String::from("Watford"),
            19 => String::from("West Ham Utd"),
            20 => String::from("Wolves"),
            _ => String::from("Unknown"),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teams() {
        let team = Team::new(5);
        assert_eq!(team.to_string(), "Burnley");
        assert_eq!(Team::new(1).to_string(), "Arsenal");
    }
}
