use actix::prelude::*;
use actix_files as fs;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::collections::HashMap;

use num_traits::FromPrimitive;
use std::time::Instant;

use super::session::Session;
use crate::common::game::Game as NetWalk;
use crate::common::rng::GameSeed;
use crate::common::tile::{RotationDirection, TilePos};
use crate::common::util::{Flag, Pos, Size};

use num_derive::{FromPrimitive, ToPrimitive};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
  pub id: String,
  pub session: Client,
}

#[derive(Serialize, Deserialize, Clone, MessageResponse)]
pub struct Token {
  pub token: String,
}

#[derive(Message)]
#[rtype(result = "ClientRequest")]
pub struct Login {
  pub id: String,
  pub auth_token: Option<String>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewGameResult {
  pub root: TilePos,
  pub seed: String,
  pub tiles: Vec<u8>,
  pub size: Size<i32>,
}

#[derive(Serialize, Deserialize, FromPrimitive, ToPrimitive)]
pub enum ClientMethodKind {
  SetToken = 0,
  SetGame,
  UpdateGameState,
}

#[derive(Serialize, Deserialize, FromPrimitive, ToPrimitive, Debug)]
pub enum ServerMethodKind {
  Login = 0,
  NewGame,
  RotateTile,
}

pub type Method = i32;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerRequestArgs {
  pub method: ServerMethodKind,
  pub token: Option<String>,
  pub data: Option<String>,
}

impl ServerRequestArgs {
  pub fn parse(s: &str) -> Option<Self> {
    let arr: Vec<&str> = s.splitn(3, " ").collect();
    if arr.len() < 3 {
      return None;
    }

    let mut m: HashMap<String, String> = HashMap::new();
    for kv in arr {
      let kv: Vec<&str> = kv.splitn(2, ":").collect();
      m.insert(kv[0].to_owned(), kv[1].to_owned());
    }

    let method: ServerMethodKind = if let Some(method) = m.get("method") {
      match method.parse() {
        Ok(v) => match FromPrimitive::from_i32(v) {
          Some(v) => v,
          None => return None,
        },
        Err(_) => return None,
      }
    } else {
      return None;
    };

    let token = if let Some(token) = m.get("token") {
      Some(token.to_owned())
    } else {
      None
    };

    let data = if let Some(data) = m.get("data") {
      Some(data.to_owned())
    } else {
      None
    };

    Some(Self {
      method: method,
      token: token,
      data: data,
    })
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerRequest<T> {
  pub token: String,
  pub data: T,
}

impl<T> actix::Message for ServerRequest<T>
where
  T: Message,
{
  type Result = T::Result;
}

pub trait ServerRequestDeserialize<T> {
  fn new(token: String, data: Option<String>) -> Option<ServerRequest<T>>
  where
    T: DeserializeOwned;
}

impl<T> ServerRequestDeserialize<T> for ServerRequest<T> {
  fn new(token: String, data: Option<String>) -> Option<Self>
  where
    T: DeserializeOwned,
  {
    let data = if let Some(v) = data {
      match serde_json::from_str(&v) {
        Ok(v) => v,
        Err(_) => return None,
      }
    } else {
      return None;
    };

    Some(Self {
      token: token,
      data: data,
    })
  }
}

#[derive(Serialize, Deserialize, MessageResponse)]
pub struct ClientRequest {
  pub method: Method,
  pub data: String,
}

impl ClientRequest {
  pub fn new<T>(method: ClientMethodKind, v: &T) -> Self
  where
    T: Serialize + 'static,
  {
    let data = match serde_json::to_string(v) {
      Ok(s) => s,
      Err(_) => "".to_owned(),
    };

    Self {
      method: method as Method,
      data: data,
    }
  }
}

#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "Option<ClientRequest>")]
pub struct NewGame {
  pub size: Size<i32>,
  pub seed: Option<String>,
}

#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "Option<ClientRequest>")]
pub struct RotateTile {
  pub direction: RotationDirection,
  pub pos: Pos<i32>,
}

#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "Option<ClientRequest>")]
pub struct UpdateGameState {
  pub pos: Pos<i32>,
  pub flag: Flag,
  pub is_solved: bool,
  pub time: u128,
}

impl Handler<ServerRequest<RotateTile>> for Server {
  type Result = Option<ClientRequest>;

  fn handle(&mut self, req: ServerRequest<RotateTile>, _: &mut Context<Self>) -> Self::Result {
    let t = Instant::now();

    if let Some(client) = self.sessions.get_mut(&req.token) {
      if let Some(game) = &mut client.game {
        let r = game.board.rotate_tile(&req.data.pos, req.data.direction);
        let b = game.board.is_solved();

        if r > 0 {
          return Some(ClientRequest::new(
            ClientMethodKind::UpdateGameState,
            &UpdateGameState {
              pos: req.data.pos,
              flag: r,
              is_solved: b,
              time: t.duration_since(game.board.get_start_time()).as_millis(),
            },
          ));
        }
      }
    }

    None
  }
}

pub struct Client {
  pub game: Option<NetWalk>,
}

pub struct Server {
  sessions: HashMap<String, Client>,
}

impl Default for Server {
  fn default() -> Server {
    Server {
      sessions: HashMap::new(),
    }
  }
}

impl Actor for Server {
  type Context = Context<Self>;
}

impl Handler<Connect> for Server {
  type Result = ();

  fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
    self.sessions.insert(msg.id.clone(), msg.session);
  }
}

impl Handler<Login> for Server {
  type Result = ClientRequest;

  fn handle(&mut self, msg: Login, _: &mut Context<Self>) -> Self::Result {
    // TODO: auth
    // if let Some(token) = msg.auth_token {
    //   token
    // }
    ClientRequest::new(ClientMethodKind::SetToken, &Token { token: msg.id })
  }
}

impl Handler<Disconnect> for Server {
  type Result = ();

  fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
    println!("disconnected: {}", msg.id);

    self.sessions.remove(&msg.id);
  }
}

impl Handler<ServerRequest<NewGame>> for Server {
  type Result = Option<ClientRequest>;

  fn handle(&mut self, req: ServerRequest<NewGame>, _: &mut Context<Self>) -> Self::Result {
    if let Some(session) = self.sessions.get_mut(&req.token) {
      let seed = if let Some(seed) = req.data.seed {
        match GameSeed::from_str(&seed) {
          Ok(seed) => seed,
          Err(_) => return None,
        }
      } else {
        GameSeed::new()
      };

      let mut game = NetWalk::new(req.data.size.height, req.data.size.width, seed);

      let res = NewGameResult {
        root: game.board.get_root(),
        size: game.board.get_size(),
        tiles: game.board.get_directions(),
        seed: game.rng.seed().to_string(),
      };

      game.board.start_timer();

      session.game = Some(game);

      return Some(ClientRequest::new(ClientMethodKind::SetGame, &res));
    }

    None
  }
}

async fn web_socket_index(
  req: HttpRequest,
  stream: web::Payload,
  srv: web::Data<Addr<Server>>,
) -> Result<HttpResponse, Error> {
  ws::start(
    Session {
      id: nanoid::generate(32),
      hb: Instant::now(),
      addr: srv.get_ref().clone(),
    },
    &req,
    stream,
  )
}

pub async fn start_server() -> std::io::Result<()> {
  env_logger::init();

  let server = Server::default().start();

  HttpServer::new(move || {
    App::new()
      .data(server.clone())
      .wrap(middleware::Logger::default())
      .service(web::resource("/ws/").route(web::get().to(web_socket_index)))
      .service(fs::Files::new("/", "static/").index_file("index.html"))
  })
  .bind("127.0.0.1:3030")?
  .run()
  .await
}
