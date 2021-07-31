mod api;
mod player;
mod team;
mod squad;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let list = api::get_full_sorted_player_list().unwrap();
    for player in list {
        println!("{}", player.to_string());
    }
    Ok(())
}
