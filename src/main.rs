mod book;
mod config;

fn main() {
    let config = config::load_config()
        .unwrap_or_else(|_| panic!("Cannot loading config"));
    let mut conn = config::connect_to_database(config.db());

    let publishers = book::get_publisher_all(&mut conn);
    println!("{:?}", publishers)
}