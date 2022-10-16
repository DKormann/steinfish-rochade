mod bot;
mod game;

pub use game::*;

// use crate::game::console_log

use wasm_bindgen::prelude::*;
use std::{fmt::{self}};
use strum_macros::Display;

#[wasm_bindgen]
extern "C"{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()));
}
pub(crate) use console_log;

const DIM:usize = 64;
const KNIGHT_HOPS:[(i8,i8);8] =  [(1,2),(2,1),(1,-2),(2,-1),(-1,-2),(-2,-1),(-1,2),(-2,1)];
const STRAIGHTS:[(i8,i8);4] = [(1,0),(-1,0),(0,1),(0,-1)];
const DIAGONALS:[(i8,i8);4] = [(1,1),(-1,1),(1,-1),(-1,-1)];

#[derive (Display,Debug,Clone, Copy,PartialEq)]
enum Color{
    White,
    Black,
}

impl Color{
    fn from_num(num:usize)->Color{
        [Color::White,Color::Black][num]
    }
    fn to_num(&self)->usize{
        match self{
            Color::White=>0,
            Color::Black=>1
        }
    }
    fn other(&self)->Color{
        Color::from_num(1-self.to_num())
    }
    fn get_dir(&self)->i8{
        1 - self.to_num()as i8 *2
    }
}

#[derive(Display,Debug,Clone, Copy)]
enum Piece{
    //the bool field represents whether the piece has moved
    Rook,
    King,
    Pawn,
    Knight,
    Bishop,
    Queen,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum PieceInfo{
    None,
    Moved,
    JustDoubleMoved,
}

#[derive(Copy,Clone,Debug)]
enum Tile{
    Empty,
    Taken(Color,Piece,PieceInfo),
}


impl Tile{
    fn to_num(&self) -> u32{
        match self {
            Tile::Empty=>0,
            Tile::Taken(color,piece,_)=>{
                let shift = match color{
                    Color::White => 0,
                    Color::Black => 6
                };
                shift + match piece {
                    Piece::Rook =>1,
                    Piece::Knight=>2,
                    Piece::Bishop=>3,
                    Piece::King=>4,
                    Piece::Queen=>5,
                    Piece::Pawn=>6,
                }
            }
        }
    }

    fn from_num(num:u32) -> Tile{
        match num{
            0=>Tile::Empty,
            mut x=>{
                Tile::Taken(
                    if x > 6 {
                        x -= 6;
                        Color::Black
                    }else{
                        Color::White
                    },
                    match x {
                        1=>Piece::Rook,
                        2=>Piece::Knight,
                        3=>Piece::Bishop,
                        4=>Piece::King,
                        5=>Piece::Queen,
                        6=>Piece::Pawn,
                        _=>{
                            console_log!("cannot find piece for {}",x);
                            panic!()}
                    }, 
                    PieceInfo::None
                )
            }
        }
    }

    fn get_value(&self)->i8{
        match self{
            Tile::Empty=>0,
            Tile::Taken(_,piece,_)=>match piece{
                Piece::Rook=>5,
                Piece::Bishop=>3,
                Piece::Knight=>3,
                Piece::Queen=>9,
                Piece::Pawn=>1,
                Piece::King=>3,
            }

        }
    }

}
impl fmt::Display for Tile{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        // write!(f,"<{} {}>",self.color,self.class)
        match self{
            Tile::Empty=> write!(f,"Empty"),
            Tile::Taken(color,piece,_) => {
                color.fmt(f)?;
                piece.fmt(f)
            }
        }
    }
}

#[derive(Debug)]
struct BoundaryError;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Pos{
    num:usize,
    x:i8,
    y:i8,
}

impl Pos{
    fn from_num(num:usize)->Pos{
        Pos { num: num, x: num as i8 %8, y: (num as i8 /8) }
    }
    fn from_ints(x:i8,y:i8)->Pos{
        Pos{num:x as usize +y as usize*8,x:x,y:y}
    }
    fn step(&self, x:i8,y:i8)->Result<Pos,BoundaryError>{

        let new_x = x + self.x;
        let new_y = y + self.y;
        if new_x < 0 || new_x > 7 ||
        new_y < 0 || new_y > 7 {
            return Err(BoundaryError);
        }else{
            let new_pos = Pos::from_ints(new_x, new_y);
            return Ok(new_pos);
        }
    }
}

impl fmt::Display for Pos{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        write!(f,"({},{})",self.x,self.y)
    }
}

#[derive(Clone,Debug)]
pub struct PossibleMove{
    start:Pos,
    end:Pos,
    extra:Vec< (Pos,Tile) >,
    upgrade:u8,
}

impl fmt::Display for PossibleMove{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        write!(f,"possibleMove: {} to {} upgrade:{}",self.start,self.end,self.upgrade)
    }
}

impl PossibleMove{
    fn basic (start:Pos,end:Pos, upgrade:u8)->PossibleMove{
        PossibleMove { start: start, end: end, extra: vec![] ,upgrade}
    }

    fn en_passante(start:Pos,end:Pos,target:Pos)->PossibleMove{
        PossibleMove { start: start, end: end, extra: vec![(target,Tile::Empty)] ,upgrade:0}
    }
}

#[derive(Display,Clone, Copy,Debug,PartialEq)]
enum GameState {
    Won(Color),
    // Draw,
    Ongoing,
}

#[wasm_bindgen]
#[derive(Clone, Copy,Debug)]
pub struct Board{
    counter: i32,
    data: [Tile;DIM],
    king_positions:[Pos;2],
    value_counts: [i32;2],
    state:GameState,
}

impl Board{

    pub fn new() -> Board{
        Board::from_nums([
            1,2,3,4, 5,3,2,1,
            6,6,6,6, 6,6,6,6,
            0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0,

            0,0,0,0, 0,0,0,0,
            0,0,0,0, 0,0,0,0,
            12,12,12,12, 12,12,12,12,
            7,8,9,10, 11,9,8,7,

        ])
    }

    fn from_nums(nums : [u32;64])->Board{
        let mut data : [Tile;64] = [Tile::Empty;64];

        let mut value_counts:[i32;2] = [0;2];

        let mut king_pos = [Pos::from_ints(3, 0),Pos::from_ints(3,7)];


        for i in 0..64{
            let tile =  Tile::from_num(nums[i]);
            data[i] = tile;
            match tile{
                Tile::Taken(color,piece,_)=>{
                    value_counts[color.to_num()] += tile.get_value() as i32;
                    match piece{
                        Piece::King=>{
                            king_pos[color.to_num()] = Pos::from_num(i);
                        }_=>{}
                    }
                }_=>{}
            }

        }

        return Board { counter: 0, data: data, king_positions: king_pos, value_counts: value_counts, state: GameState::Ongoing };
    }

    fn tile_is_empty(&self,pos:usize)->bool{matches!(self.data[pos],Tile::Empty)}

    fn push_if_tile_empty(&self,start:Pos,end:Pos,res:&mut Vec<PossibleMove>,upgrade:u8)->bool{
        match self.tile_is_empty(end.num){
            true=>{res.push(PossibleMove::basic(start,end,upgrade));
                true},
            false=>false
        }
    }

    fn push_if_tile_killable(&self, start:Pos,end:Pos,my_color:Color,res:&mut Vec<PossibleMove>,upgrade:u8)->bool{
        match self.data[end.num]{

            Tile::Empty=>{
                false
            },
            Tile::Taken(color,_,_)=>{
                match color == my_color{
                    true=>false,
                    false=>{
                        res.push(PossibleMove::basic(start, end, upgrade));
                        true
                    },
                }
            }
        }
    }

    fn try_pawn_takes(&self, start_pos:Pos,x:i8,y:i8,mover_color:Color,result:&mut Vec<PossibleMove>,upgrade:u8){

        match start_pos.step(x,y){
            Ok(target)=>{
                if ! self.push_if_tile_killable(start_pos,target,mover_color, result,upgrade){
                    //check if en passante
                    let target_pos = start_pos.step(x,0);
                    match target_pos{
                        Err(_)=>{},
                        Ok(target_pos)=>{
                            match self.data[target_pos.num]{
                                Tile::Taken(color,_,PieceInfo::JustDoubleMoved)=>{
                                    if color != mover_color {
                                        result.push(PossibleMove::en_passante(start_pos, target, target_pos));
                                    }
                                },
                                _=>{}
                            }
                        }
                    }
                }
            },
            Err(_)=>{
            }
        }

    }

    fn try_direction_step(&self, start_pos:Pos,dir:(i8,i8),color:Color,result:&mut Vec<PossibleMove>){

        let mut pos :Pos = start_pos;

        loop{
            pos = match pos.step(dir.0,dir.1){
                Ok(pos)=>pos,
                Err(_)=>return
            };
            if ! self.push_if_tile_empty(start_pos, pos, result,0){
                self.push_if_tile_killable(start_pos, pos, color, result,0);
                return 
            }
        }
    }

    fn check_safety(&self,x:i8,y:i8,color:Color)->bool{
        let start_pos = Pos::from_ints(x,y);

        let enemy_color = color.other();

        //check knight 
        for mov in KNIGHT_HOPS.iter(){
            match start_pos.step(mov.0,mov.1){
                Ok(pos)=>{
                    match self.data[pos.num]{
                        Tile::Taken(color,Piece::Knight,_)=>if color == enemy_color {return false},
                        _=>{}
                }}
                Err(_)=>{}
            }
        }

        // check bishop
        for dir in DIAGONALS.iter(){
            let mut pos = start_pos;
            loop{
                match pos.step(dir.0,dir.1){
                    Ok(new_pos)=>match self.data[new_pos.num]{
                        Tile::Taken(color,piece,_)=>{
                            if color == enemy_color && (matches!(piece, Piece::Bishop) || matches!(piece, Piece::Queen)){
                                return false
                            }else {
                                break
                            }
                        }
                        Tile::Empty=>{
                            pos = new_pos
                        }
                    }
                    Err(_)=>break
                }
            }
        }

        //check rook
        for dir in STRAIGHTS.iter(){
            let mut pos = start_pos;


            loop{
                match pos.step(dir.0,dir.1){
                    
                    Ok(new_pos)=>match self.data[new_pos.num]{

                        Tile::Taken(color,piece,_)=>{
                            if color == enemy_color && (matches!(piece, Piece::Rook) || matches!(piece,Piece::Queen)){
                                // console_log!("checking for straight rook or queen for {} ",start_pos);
                                // console_log!("hit on {}",new_pos);
                                return false
                            }else{
                                // console_log!("stop on {} {} ",new_pos,piece);
                                break
                            }
                        }
                        Tile::Empty=> pos = new_pos
                    }
                    Err(_)=>{
                        // console_log!("end on {}",pos);
                        break
                    }
        }}}
        
        //check enemy king 
        for mov in [DIAGONALS,STRAIGHTS].concat().iter(){
            match start_pos.step(mov.0,mov.1){
                Ok(pos)=>{
                    match self.data[pos.num]{
                        Tile::Taken(color,Piece::King,_)=>if color == enemy_color {return false},
                        _=>{}
                }}
                Err(_)=>{}
            }
        }

        //check enemy pawns
        {
            let diry = color.get_dir();
            
            match start_pos.step(1,diry){
                Ok(pos)=>{
                    match self.data[pos.num]{
                        Tile::Taken(other_color,Piece::Pawn ,_)=>{
                            if color != other_color{
                                return  false
                            }
                        }
                        _=>{}
                    }
                }
                Err(_)=>{}
            }
            match start_pos.step(-1,diry){
                Ok(pos)=>{
                    match self.data[pos.num]{
                        Tile::Taken(other_color,Piece::Pawn,_)=>{
                            if color != other_color{
                                return false
                            }
                        }
                        _=>{}
                    }
                }
                Err(_)=>{}
            }
        }

        true
    }

    fn check_board_safety(&self)->[bool;2]{
        let mut res = [true;2];
        for i in 0..2{
            let king_pos = self.king_positions[i];
            res[i] = self.check_safety(king_pos.x,king_pos.y, Color::from_num(i));
        }
        res
    }

    fn try_rochade(&self,start_pos:Pos,end_pos:Pos,color:Color,result:&mut Vec<PossibleMove>){

        if ! self.check_safety(start_pos.x, start_pos.y, color){return} ;

        let king_end_x;
        let rook_end_x;
        let king_path;

        match end_pos.x{
            0=>{
                king_end_x = 1;
                king_path = 1..3;
                rook_end_x = 2;
            },
            7=>{
                king_end_x = 5;
                king_path = 4..6;
                rook_end_x = 4;
            }
            _=>{panic!()}
        };

        for x in king_path{


            if ! (matches!(self.data[Pos::from_ints(x, start_pos.y).num],Tile::Empty)&&
            self.check_safety(x,start_pos.y,color)){
                return 
            }
        }

        result.push(PossibleMove { 
            start: start_pos, end: Pos::from_ints(king_end_x,start_pos.y), 
            extra: vec![(end_pos,Tile::Empty),(Pos::from_ints(rook_end_x,start_pos.y),self.data[end_pos.num] )],
            upgrade:0,
        })
    }

    fn get_possible_moves_for_pos(&self,pos:Pos,move_color:Color,upgrade:u8)-> Vec<PossibleMove>{
        
        let mover = self.data[pos.num];

        let mut result=vec![];

        match mover{
            Tile::Empty=>return result,
            Tile::Taken(color,piece,info)=>{
                if color != move_color{
                    return result;
                }
                let start_pos: Pos = Pos::from_num(pos.num);

                match piece{
                    Piece::Pawn=>{
                        let dir = color.get_dir();

                        if self.push_if_tile_empty(start_pos, start_pos.step(0,dir).unwrap(), &mut result,upgrade){
                            if info == PieceInfo::None{
                                self.push_if_tile_empty(start_pos, start_pos.step(0,2*dir).unwrap(), &mut result,upgrade);
                            }
                        }
                        self.try_pawn_takes(start_pos, 1, dir, color, &mut result,upgrade);
                        self.try_pawn_takes(start_pos, -1, dir, color, &mut result,upgrade);

                        // console_log!("pawn moves found: {}",result.last().unwrap());

                    }
                    
                    Piece::Knight=>{
                        for hop in KNIGHT_HOPS.iter(){
                            match start_pos.step(hop.0,hop.1){
                                Err(_)=>{}
                                Ok(target)=>{
                                    if ! self.push_if_tile_empty(start_pos, target, &mut result,0){
                                        self.push_if_tile_killable(start_pos, target, color, &mut result,0);
                                    }
                                }
                            }
                        }

                    }
                    Piece::Bishop=>{
                        for dir in DIAGONALS.iter(){
                            self.try_direction_step(start_pos, *dir, color, &mut result)
                        }
                    }
                    Piece::Rook=>{
                        for dir in STRAIGHTS.iter(){
                            self.try_direction_step(start_pos, * dir, color,&mut result);
                        }
                    }
                    Piece::Queen=>{
                        for dir in [STRAIGHTS,DIAGONALS].concat().iter(){
                            self.try_direction_step(start_pos, * dir, color, &mut result)
                        }
                    }
                    Piece::King=>{
                        let moves = [DIAGONALS,STRAIGHTS].concat();
                        for mov in moves.iter(){
                            match start_pos.step(mov.0,mov.1){
                                Ok(target)=>{
                                    if ! self.push_if_tile_empty(start_pos, target, &mut result,0){
                                        self.push_if_tile_killable(start_pos, target, color, &mut result,0);
                                    }
                                }
                                Err(_)=>{}
                            }
                        }
    
                        if info == PieceInfo::None{
                            //try rochade
    
                            //check 0 rook
                            let rook_pos = Pos::from_ints(0, start_pos.y);
                            match self.data[rook_pos.num]{
                                Tile::Taken(_,Piece::Rook,PieceInfo::None)=>{
                                    self.try_rochade(start_pos,rook_pos,color,&mut result);
                                }
                                _=>{}
                            }
                            let rook_pos7:Pos= Pos::from_ints(7,start_pos.y);
                            match self.data[rook_pos7.num]{
                                Tile::Taken(_,Piece::Rook,PieceInfo::None)=>{
                                    // rochade
                                    self.try_rochade(start_pos,rook_pos7,color,&mut result);
    
                                }
                                _=>{}
                            }
                        }
                    }
                }
            }
        }


        result


    }

    fn get_possible_moves(&self,std_upgrade:u8)->Vec<PossibleMove>{
        
        let mut result = vec![];

        let move_color = Color::from_num(self.counter as usize %2);
        
        for (i,tile) in self.data.iter().enumerate(){
            match tile{
                Tile::Empty=>{}
                Tile::Taken(_,_,_)=>{
                    let pos = Pos::from_num(i);
                    result.append(&mut self.get_possible_moves_for_pos(pos,move_color,std_upgrade));
                }
            }
        }
        result
    }

    fn get_legal_moves(&mut self,std_upgrade:u8)->Vec<PossibleMove>{

        let mut options = self.get_possible_moves(std_upgrade);

        // let options = self.get_possible_moves(std_upgrade);
        let mover = Color::from_num(self.counter as usize %2).other();

        let mut i = 0;

        // for i in 0..options.len(){
        while i < options.len() {
            let mov = &options[i];
            
            let mut new_board = self.clone();
            new_board.make_possible_move(&mov);

            let safe = new_board.check_board_safety()[1-mover.to_num()];
            // console_log!("cheked board safety for {}: {}",mov,safe);
            if ! safe{
                options.remove(i);
            }else{
                i += 1;
            }
        }
        if options.len() == 0{
            self.state = GameState::Won(mover.other());
        }
        options
    }

    pub fn update(&self, start:usize, end:usize, upgrade:u8)->Board{

        let mut cp = self.clone();

        cp.make_move(start, end, upgrade);

        let idx = (cp.counter as usize - 1)%2;



        let king_pos = cp.king_positions[idx];
        let king_color = [Color::White,Color::Black][idx];


        let res = if cp.check_safety(king_pos.x, king_pos.y, king_color){
            cp
        }else{
            // console_log!("safety_check failed for {} at {}",king_color,king_pos);
            *self
        };
        res
    }

    fn make_move(&mut self, start: usize, end:usize, upgrade:u8)-> bool{

        let start_tile :Tile = self.data[start];

        let next_player_idx = self.counter as usize %2;

        let next_player_color = Color::from_num(next_player_idx);


        // is move at all possible?
        match start_tile{
            Tile::Empty=>{
                return false
            },
            Tile::Taken(color,_,_)=>{
                if color != next_player_color{
                    return false
                }
            }
        }

        let pos:Pos = Pos::from_num(start);

        let possible_moves:Vec<PossibleMove> = self.get_possible_moves_for_pos(pos,next_player_color,upgrade);

        let mut move_is_allowed : bool = false;
        
        for (_,mov) in possible_moves.iter().enumerate(){
            if mov.start.num == start && mov.end.num == end{

                let new_move;
                if upgrade != 0{
                    new_move = PossibleMove{start:mov.start,end:mov.end,extra:vec![],upgrade};
                    self.make_possible_move(&new_move);
                }else{
                    self.make_possible_move(mov);
                }

                move_is_allowed = true;
            }
        };
        move_is_allowed
    }

    fn make_possible_move(&mut self, mov: &PossibleMove){

        let  start = mov.start.num;
        let end  = mov.end.num;

        let next_player_idx = self.counter as usize %2;
        let last_player_idx = 1- next_player_idx;



        self.counter += 1;
        let mover:Tile = self.data[start];

        let mut lost_value = self.data[end].get_value() as i32;

        match self.data[end]{
            Tile::Taken(_,Piece::King,_)=>{
                self.value_counts = [0,0];
                self.value_counts[next_player_idx] = 1;
                self.state = GameState::Won(Color::from_num(next_player_idx));
            }
            _=>{
                self.value_counts[last_player_idx] -= lost_value;
            }
        }

        self.data[end] = match mover{

            Tile::Taken(color,Piece::King,_)=>{
                self.king_positions[next_player_idx] = mov.end;
                Tile::Taken(color,Piece::King,PieceInfo::Moved)
            }
            Tile::Taken(color,Piece::Rook,_)=>Tile::Taken(color,Piece::Rook,PieceInfo::Moved),
            Tile::Taken(color,Piece::Pawn,_)=>{

                //promtion ?
                if mov.end.y == 0 || mov.end.y == 7{

                    let new_class = match mov.upgrade{
                        1=> Piece::Rook,
                        2=> Piece::Knight,
                        3=> Piece::Bishop,
                        5=> Piece::Queen,
                        _=> panic!("pniccc"),
                    };


                    let res = Tile::Taken(color,new_class,PieceInfo::Moved);
                    self.value_counts[next_player_idx]+= res.get_value() as i32 -1;
                    res

                }else{
                    let info = if(start as i32 - end as i32).abs() == 16{
                        PieceInfo::JustDoubleMoved
                    }else{
                        PieceInfo::Moved
                    };
                    Tile::Taken(color,Piece::Pawn,info)
                }
            }
            any => any

        };

        self.data[start] = Tile::Empty;

        for item in mov.extra.iter(){

            lost_value  = self.data[item.0.num].get_value() as i32;

            match self.data[item.0.num]{
                Tile::Taken(color,_,_)=>{
                    if color == Color::Black{
                        self.value_counts[1] -= lost_value;
                    }else{
                        self.value_counts[0] -= lost_value;
                    }
                }
                _=>{}
            }

            self.data[item.0.num] = item.1;
            let gained_value = item.1.get_value() as i32;
            self.value_counts[next_player_idx] += gained_value ;
        }
    }

    pub fn get_data(&self)-> js_sys::Uint32Array{

        let mut array:[u32;DIM] = [0;DIM];
 
        for x in 0..DIM {
            array[x] = self.data[x].to_num();
        }

        let r = &array[..];
        js_sys::Uint32Array::from(r)
    }

}

impl fmt::Display for Board{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{


        let mut data_repr = "".to_string();
        for y in 0..8{
            data_repr += "\n|";

            for x in 0..8{

                let i =  y*8+x;
                let tile = self.data[i];
                data_repr+=[

                    // " ","♜","♞","♝","♚","♛","♟","♖","♘","♗","♔","♕","♙",
                    " ","R","N","B","K","Q","P","r","n","b","k","q","p",
                    ][tile.to_num()as usize];
                data_repr += " ";

            }
            data_repr += "|";

        }


        write!(f,"{} \n{}",data_repr,self.counter)
    }
}