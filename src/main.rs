mod common;
mod server;

// fn main() {
//   use common::game::Game as NetWalk;
//   use common::rng::GameSeed;

//   let game = NetWalk::new(7, 7, GameSeed::new());
//   println!("{:?}", game.board.get_flags());
// }

#[actix_rt::main]
async fn main() {
  server::server::start_server().await.unwrap();
}
