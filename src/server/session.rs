use std::time::{Duration, Instant};

use super::server;
use super::server::{ServerMethodKind, ServerRequest, ServerRequestDeserialize};
use actix::prelude::*;
use actix_web_actors::ws;
use serde::Serialize;

use num_traits::FromPrimitive;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Session {
  pub id: String,
  pub hb: Instant,
  pub addr: Addr<server::Server>,
}

impl Session {
  fn hb(&self, ctx: &mut <Self as Actor>::Context) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
      if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
        act.addr.do_send(server::Disconnect { id: act.id.clone() });

        ctx.stop();
        return;
      }

      ctx.ping(b"");
    });
  }

  fn send<M>(&self, msg: M, ctx: &mut <Self as Actor>::Context)
  where
    M: Message + Send + 'static,
    M::Result: Send + Serialize,
    server::Server: actix::Handler<M>,
  {
    self
      .addr
      .send(msg)
      .into_actor(self)
      .then(|res, _, ctx| {
        match res {
          Ok(res) => ctx.json(&res),
          _ => {}
        }
        fut::ready(())
      })
      .wait(ctx);
  }
}

impl Actor for Session {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    self.hb(ctx);

    self.addr.do_send(server::Connect {
      id: self.id.clone(),
      session: server::Client { game: None },
    });
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    // println!("[{}] WS: {:?}", self.id, msg);

    match msg {
      Ok(ws::Message::Ping(msg)) => {
        self.hb = Instant::now();
        ctx.pong(&msg);
      }
      Ok(ws::Message::Pong(_)) => {
        self.hb = Instant::now();
      }
      Ok(ws::Message::Text(text)) => {
        let req = if let Some(req) = server::ServerRequestArgs::parse(&text) {
          req
        } else {
          return;
        };

        // println!("req: {:?}", req);

        match req.method {
          ServerMethodKind::Login => self.send(
            server::Login {
              id: self.id.clone(),
              auth_token: req.data.clone(),
            },
            ctx,
          ),
          ServerMethodKind::NewGame => {
            let req: ServerRequest<server::NewGame> =
              match ServerRequest::new(self.id.clone(), req.data) {
                Some(req) => req,
                None => return,
              };

            self.send(req, ctx);
          }
          ServerMethodKind::RotateTile=>{
            let req: ServerRequest<server::RotateTile> =
            match ServerRequest::new(self.id.clone(), req.data) {
              Some(req) => req,
              None => return,
            };

            self.send(req, ctx);
          }
          //self.send(
          //   server::NewGame {
          //     id: self.id.clone(),
          //     auth_token: req.data.clone(),
          //   },
          //   ctx,
          // ),
        }

        // match args.method.as_str() {
        //   "new_game" => {
        //     self.addr.send(server::NewGame {
        //       id: self.id.clone(),
        //     });
        //   }
        //   _ => {}
        // }
      }
      Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
      Ok(ws::Message::Close(_)) => {
        ctx.stop();
      }
      _ => ctx.stop(),
    }
  }
}

trait WebsocketContextUtils {
  fn json<T: Serialize>(&mut self, obj: &T);
}

impl<A> WebsocketContextUtils for ws::WebsocketContext<A>
where
  A: Actor<Context = Self>,
{
  fn json<T: Serialize>(&mut self, obj: &T) {
    let s = serde_json::to_string(obj).unwrap();
    self.text(s)
  }
}
