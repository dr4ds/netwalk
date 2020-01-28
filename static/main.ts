const WS_HOST = "ws://127.0.0.1:3030/ws/";

function flag_count(n: number) {
  var c = 0;
  while (n != 0) {
    if ((n & 1) == 1) {
      c += 1;
    }
    n = n >> 1;
  }

  return c;
}

enum TileKind {
  CONNECTOR,
  SERVER,
  TERMINAL
}

interface Tile {
  kind: TileKind;
  flag: number;
  is_powered: boolean;
}

enum DirectionKind {
  UP = 1,
  RIGHT = 2,
  DOWN = 4,
  LEFT = 8
}

interface Direction {
  kind: DirectionKind;
  flag: number;
  opposite: number;
  offset: Pos;
}

const DIRECTIONS: Direction[] = [
  {
    kind: DirectionKind.UP,
    flag: 1,
    opposite: 4,
    offset: { x: 0, y: -1 }
  },
  {
    kind: DirectionKind.RIGHT,
    flag: 2,
    opposite: 8,
    offset: { x: 1, y: 0 }
  },
  {
    kind: DirectionKind.DOWN,
    flag: 4,
    opposite: 1,
    offset: { x: 0, y: 1 }
  },
  {
    kind: DirectionKind.LEFT,
    flag: 8,
    opposite: 2,
    offset: { x: -1, y: 0 }
  }
];

function $(v: string) {
  return document.getElementById(v);
}

function uuid() {
  var dt = new Date().getTime();
  var uuid = "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function(
    c
  ) {
    var r = (dt + Math.random() * 16) % 16 | 0;
    dt = Math.floor(dt / 16);
    return (c == "x" ? r : (r & 0x3) | 0x8).toString(16);
  });
  return uuid;
}

function split_n(s: string, pattern: string, n: number) {
  var arr = s.split(" ");
  var result = arr.splice(0, 2);

  result.push(arr.join(" "));

  return result;
}

enum ClientMethodKind {
  SetToken = 0,
  SetGame,
  UpdateGameState
}

enum ServerMethodKind {
  Login = 0,
  NewGame,
  RotateTile
}

interface Pos {
  x: number;
  y: number;
}

interface Size {
  width: number;
  height: number;
}

enum RotationDirection {
  Right = "Right",
  Left = "Left"
}

interface RotateTile {
  pos: Pos;
  direction: RotationDirection;
}

interface UpdateGameState {
  pos: Pos;
  flag: number;
  is_solved: boolean;
  time: number;
}

interface NewGame {
  size: Size;
  seed: string | undefined;
}

interface Token {
  token: string;
}

interface ClientRequest {
  method: ClientMethodKind;
  data: string;
}

class ServerRequest {
  method: ServerMethodKind;
  token: string | null;
  data: string | null;
  constructor(
    method: ServerMethodKind,
    token: string | null,
    data: any | null
  ) {
    this.method = method;
    this.token = token;
    this.data = data;
  }

  to_string() {
    var s = "";
    s += "method:";
    s += this.method.toString();
    s += " token:";
    if (this.token) {
      s += this.token;
    }
    s += " data:";
    if (this.data) {
      s += JSON.stringify(this.data);
    }
    return s;
  }
}

interface NetWalk {
  root: Pos;
  size: Size;
  seed: string;
  tiles: number[];
}

class Board {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  root: Pos;
  tiles: Tile[];
  width: number;
  height: number;
  selected_tile: Pos | null;
  last_selected_tile: Pos | null;
  scale: number;
  xo: number;
  yo: number;
  on_rotate_tile: (pos: Pos) => void;

  constructor(width: number, height: number, tiles: number[], root: Pos) {
    this.canvas = $("canvas") as HTMLCanvasElement;
    this.ctx = this.canvas.getContext("2d")!;
    this.selected_tile = null;
    this.last_selected_tile = null;
    this.width = width;
    this.height = height;
    this.tiles = Board.process_tile_flags(tiles);
    this.root = root;
    this.on_rotate_tile = () => {};

    this.update_connectivity();

    this.canvas.height = 600;
    this.canvas.width = 600;
    this.canvas.onmousemove = event => {
      let mouse_pos: Pos = { x: event.clientX, y: event.clientY };
      let tile_pos = this.get_tile_pos_from_mouse_pos(mouse_pos);

      this.select_tile(tile_pos);
    };

    this.canvas.onmousedown = () => {
      this.rotate_selected_tile();
    };

    // TODO: math floor this if selected tile precision is bad
    this.scale = Math.min(
      ((this.canvas.width - 10) * 1.0) / this.width,
      ((this.canvas.height - 10) * 1.0) / this.height
    );
    this.xo =
      Math.floor((this.canvas.width - this.width * this.scale) / 2) + 0.5;
    this.yo =
      Math.floor((this.canvas.height - this.height * this.scale) / 2) + 0.5;
  }

  set_tile_flag(pos: Pos, n: number) {
    this.get_tile(pos).flag = n;
    this.update_connectivity();
    this.draw();
  }

  static process_tile_flags(data: number[]) {
    let tiles: Tile[] = [];
    for (let i = 0; i < data.length; i++) {
      const n = data[i];

      let tile_kind = TileKind.CONNECTOR;
      if (flag_count(n) == 1) {
        tile_kind = TileKind.TERMINAL;
      }
      tiles.push({ kind: tile_kind, flag: n, is_powered: false });
    }

    return tiles;
  }

  in_bounds(pos: Pos) {
    return (
      pos.x >= 0 && pos.y >= 0 && pos.x < this.width && pos.y < this.height
    );
  }

  rotate_selected_tile() {
    if (this.selected_tile && this.in_bounds(this.selected_tile)) {
      this.on_rotate_tile(this.selected_tile);
    }
  }

  solve_walk(pos: Pos) {
    var tile = this.get_tile(pos);
    tile.is_powered = true;

    for (let d = 0; d < DIRECTIONS.length; d++) {
      const dir = DIRECTIONS[d];
      if ((dir.flag & tile.flag) != 0) {
        var neighbour_pos: Pos = {
          x: pos.x + dir.offset.x,
          y: pos.y + dir.offset.y
        };

        if (this.in_bounds(neighbour_pos)) {
          let neighbour_tile = this.get_tile(neighbour_pos);
          if (
            (dir.opposite & neighbour_tile.flag) != 0 &&
            !neighbour_tile.is_powered
          ) {
            this.solve_walk(neighbour_pos);
          }
        }
      }
    }
  }

  update_connectivity() {
    for (let i = 0; i < this.width; i++) {
      for (let j = 0; j < this.height; j++) {
        this.get_tile({ x: i, y: j }).is_powered = false;
      }
    }

    this.solve_walk(this.root);
  }

  clear() {
    this.canvas.width = this.canvas.width;
  }

  clear_last_selected_tile() {
    if (!this.last_selected_tile) {
      return;
    }

    this.ctx.fillStyle = "#fff";
    this.ctx.fillRect(
      this.last_selected_tile.x * this.scale,
      this.last_selected_tile.y * this.scale,
      this.scale,
      this.scale
    );
  }

  select_tile(pos: Pos) {
    let tile = this.get_tile(pos);
    if (tile) {
      this.last_selected_tile = this.selected_tile;
      this.selected_tile = pos;
      // this.clear_last_selected_tile();
      // this.draw_selected_tile();
    }
  }

  draw_selected_tile() {
    if (!this.selected_tile) {
      return;
    }

    this.ctx.fillStyle = "#f00";
    this.ctx.fillRect(
      this.selected_tile.x * this.scale,
      this.selected_tile.y * this.scale,
      this.scale,
      this.scale
    );
  }

  draw_end_nodes() {
    for (var i = 0; i < this.width; ++i) {
      for (var j = 0; j < this.height; ++j) {
        const x = this.xo + i * this.scale;
        const y = this.yo + j * this.scale;
        const tile = this.get_tile({ x: i, y: j });

        if (tile.kind == TileKind.TERMINAL) {
          if (tile.is_powered) {
            this.ctx.fillStyle = "#03fc8c";
          } else {
            this.ctx.fillStyle = "#e46464";
          }

          this.ctx.fillRect(
            x + this.scale / 4,
            y + this.scale / 4,
            this.scale / 2,
            this.scale / 2
          );
        }
      }
    }
  }

  draw_root() {
    this.ctx.fillStyle = "#7d32a8";
    let x = this.xo + this.root.x * this.scale;
    let y = this.yo + this.root.y * this.scale;
    this.ctx.fillRect(
      x + this.scale / 4,
      y + this.scale / 4,
      this.scale / 2,
      this.scale / 2
    );
  }

  draw_lines() {
    this.ctx.strokeStyle = "#aaa";
    for (var i = 0; i <= this.width; ++i) {
      this.ctx.moveTo(this.xo + i * this.scale, this.yo);
      this.ctx.lineTo(
        this.xo + i * this.scale,
        this.yo + this.height * this.scale
      );
      this.ctx.stroke();
    }
    for (var j = 0; j <= this.width; ++j) {
      this.ctx.moveTo(this.xo, this.yo + j * this.scale);
      this.ctx.lineTo(
        this.xo + this.height * this.scale,
        this.yo + j * this.scale
      );
      this.ctx.stroke();
    }
  }

  draw_tiles() {
    for (var i = 0; i < this.width; ++i) {
      for (var j = 0; j < this.height; ++j) {
        const x = this.xo + i * this.scale;
        const y = this.yo + j * this.scale;
        for (var d in DIRECTIONS) {
          const dir = DIRECTIONS[d];
          const tile = this.get_tile({ x: i, y: j });
          if (dir.flag & tile.flag) {
            if (tile.is_powered) {
              this.ctx.fillStyle = "#32a852";
            } else {
              this.ctx.fillStyle = "#3b3b3b";
            }

            this.ctx.fillRect(
              x + (this.scale * (Math.min(dir.offset.x, 0) + 1)) / 3,
              y + (this.scale * (Math.min(dir.offset.y, 0) + 1)) / 3,
              (this.scale * (Math.abs(dir.offset.x) + 1)) / 3,
              (this.scale * (Math.abs(dir.offset.y) + 1)) / 3
            );
          }
        }
      }
    }
  }

  draw() {
    this.clear();

    this.draw_lines();
    this.draw_tiles();
    this.draw_end_nodes();
    this.draw_root();
  }

  get_tile(pos: Pos) {
    return this.tiles[pos.x + pos.y * this.width];
  }

  get_tile_pos_from_mouse_pos(mouse_pos: Pos) {
    var rect = this.canvas.getBoundingClientRect();
    var m_pos: Pos = {
      x: mouse_pos.x - rect.left,
      y: mouse_pos.y - rect.top
    };

    var pos = {
      x: Math.floor(m_pos.x / this.scale),
      y: Math.floor(m_pos.y / this.scale)
    };

    return pos;
  }
}

class Game {
  board: Board;
  seed: string;
  start_time: number;
  current_time: number;
  is_finished: boolean;
  finish_time: number;
  constructor(nw: NetWalk) {
    this.seed = nw.seed;
    this.board = new Board(nw.size.height, nw.size.width, nw.tiles, nw.root);
    this.is_finished = false;
    this.finish_time = 0;
    this.current_time = 0;
    this.draw();

    this.start_time = Date.now();
  }

  draw() {
    if (!this.board) {
      return;
    }

    this.board.draw();
  }
}

class App {
  ws: WebSocket;
  token: string | null;
  game: Game | null;
  time_element: HTMLParagraphElement;

  constructor() {
    this.game = null;
    this.ws = new WebSocket(WS_HOST);
    this.token = null;
    this.time_element = $("time")! as HTMLParagraphElement;

    this.init_listeners();
    this.init_socket();
  }

  init_socket() {
    this.ws.onmessage = event => this.handle_message(event.data);

    this.ws.onopen = () => {
      this.send(ServerMethodKind.Login, this.token);
    };
  }

  handle_message(msg: any) {
    let req: ClientRequest = JSON.parse(msg);

    switch (req.method) {
      case ClientMethodKind.SetToken:
        let token: Token = JSON.parse(req.data);
        this.token = token.token;
        break;
      case ClientMethodKind.SetGame:
        let nw: NetWalk = JSON.parse(req.data);
        this.game = new Game(nw);
        this.game.board.on_rotate_tile = pos => {
          let req: RotateTile = {
            pos: pos,
            direction: RotationDirection.Right
          };
          this.send(ServerMethodKind.RotateTile, req);
        };
        break;

      case ClientMethodKind.UpdateGameState:
        let data: UpdateGameState = JSON.parse(req.data);
        if (this.game && data) {
          this.game.board.set_tile_flag(data.pos, data.flag);

          if (data.is_solved && !this.game.is_finished) {
            this.game.is_finished = true;
            this.game.finish_time = data.time;
          }
        }
        break;

      default:
        break;
    }
  }

  send(method: ServerMethodKind, data: any | null) {
    let req = new ServerRequest(method, this.token, data);

    this.ws.send(req.to_string());
  }

  init_listeners() {
    $("5x5")!.onclick = () => {
      this.new_game({ height: 5, width: 5 });
    };

    $("7x7")!.onclick = () => {
      this.new_game({ height: 7, width: 7 });
    };

    $("10x10")!.onclick = () => {
      this.new_game({ height: 10, width: 10 });
    };

    $("15x15")!.onclick = () => {
      this.new_game({ height: 15, width: 15 });
    };

    $("20x20")!.onclick = () => {
      this.new_game({ height: 20, width: 20 });
    };

    setInterval(() => {
      if (this.game) {
        var t = 0;
        if (this.game.is_finished) {
          t = this.game.finish_time;
        } else {
          this.game.current_time = Date.now() - this.game.start_time;
          t = this.game.current_time;
        }

        const seconds = Math.floor((t / 1000) % 60);
        const minutes = Math.floor((t / 1000 / 60) % 60);

        t -= seconds * 1000;
        t -= minutes * 1000;

        let time_s = "time: ";
        if (minutes) {
          time_s += minutes.toString() + ":";
        }
        time_s += seconds.toString() + ".";
        time_s += t.toString();

        this.time_element.textContent = time_s;
      }
    }, 20);
  }

  new_game(size: Size, seed?: string) {
    let ng: NewGame = { size: size, seed: seed };
    this.send(ServerMethodKind.NewGame, ng);
  }
}

let app = new App();
