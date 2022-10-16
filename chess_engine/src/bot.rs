
use std::fmt::Display;
use std::vec;

use crate::Board;
use crate::PossibleMove;

// use wasm_bindgen::prelude::*;
use crate::*;

use wasm_bindgen::prelude::*;


const BETA: f32 = 0.03;


fn eval(board:Board,player:Color) -> f32{

    // let player_idx = player.to_num();

    let mut vals :[f32;2] = [0.,0.];

    
    for i in 0..64{
        let tile = board.data[i];
        let pos = Pos::from_num(i);
        match tile{
            Tile::Taken(color,piece,_)=>{
                let val:f32 = match piece{
                    Piece::Rook=>{
                        let move_options = board.get_possible_moves_for_pos(pos, color, 0).len();
                        5. + move_options as f32 *0.2
                    }
                    Piece::Pawn=>{

                        let progress = if player == Color::White{
                            pos.y 
                        } else{
                            7 - pos.y
                        };

                        1. + progress as f32 * 0.1
                    }
                    Piece::Knight=>{

                        let center_values = [
                            0,0,0,0, 0,0,0,0, 
                            0,1,1,1, 1,1,1,0, 
                            0,1,2,2, 2,2,1,0, 
                            0,1,2,3, 3,2,1,0, 

                            0,1,2,3, 3,2,1,0, 
                            0,1,2,2, 2,2,1,0, 
                            0,1,1,1, 1,1,1,0, 
                            0,0,0,0, 0,0,0,0, 
                        ];

                        let positional_value = center_values[pos.num] as f32 * 0.3;

                        3. + positional_value
                    }
                    Piece::Bishop=>{

                        let move_options = board.get_possible_moves_for_pos(pos, color, 0).len();
                        3. + move_options as f32 * 0.4
                    }
                    Piece::Queen=>{
                        let move_options = board.get_possible_moves_for_pos(pos, color, 0).len();
                        9. + move_options as f32 * 0.1
                    }
                    Piece::King=>{
                        let early_king_values = [
                            1,1,0,0, 0,0,1,1,
                            0,0,0,0, 0,0,0,0,
                            0,0,0,0, 0,0,0,0,
                            0,0,0,0, 0,0,0,0,

                            0,0,0,0, 0,0,0,0,
                            0,0,0,0, 0,0,0,0,
                            0,0,0,0, 0,0,0,0,
                            1,1,0,0, 0,0,1,1,

                        ];
                        if board.counter < 10{
                            early_king_values[pos.num] as f32 * 0.5
                        }else{
                            0.
                        }
                    }
                };
                vals[color.to_num()]+= val;
            }
            Tile::Empty=>{}
        }
    }


    // if vals[0] as i32 != board.value_counts[0]{
    //     console_log!("deviant eval {:?} {:?} ",vals,board.value_counts);
    // }



    // advantage for white from -1 to +1
    let white_advantage:f32 = (vals[0]-vals[1]) / (vals[0]+vals[1]);

    //if we are looking for analysis of black player reverse the result
    let mut result = white_advantage;
    // if player == Color::Black{
    //     result = - result;
    // }    
    result = result * player.get_dir()  as f32;

    // console_log!("eval {} ",result);

    
    let mx = 0.99;
    if result >= mx{
        console_log!("too big eval");
        mx

    }else if result <= -mx{
        console_log!("too smol eval");
        -mx
    }else{
        result
    }


    // return result
    // result


}

#[derive(Debug)]
struct Chain{
    prev: Option<Box<Chain>>,
}


#[wasm_bindgen(start)]
pub fn pre_bot(){
    // let board = Board::from_nums([
    //     0,0,0,7, 11,0,0,0,
    //     0,0,0,0, 0,0,0,0,
    //     0,0,0,0, 0,0,0,0,
    //     0,0,0,0, 0,0,0,0,

    //     0,0,0,0, 0,0,0,0,
    //     0,0,0,0, 0,0,0,0,
    //     0,0,0,0, 0,0,0,0,
    //     0,0,0,2, 5,0,0,0,

    // ]);
    
    let first = Chain {
        prev : Option::None,

    };

    let mut first = Chain{prev: None};

    let second :Chain = Chain{prev :  Option::Some(Box::new(first))};

    first = Chain{ prev : Option::Some(Box::new(second))};

    console_log!("chain start {:?} ",first.prev);

    
    

    // console_log!("{}\n white eval : {}",board,eval(board,Color::White));
}





pub fn choose_move(board:Board)->PossibleMove{


    // let depth = 100000;
    let depth = 1e3 as i32;


    let mut root = SearchNode::new(board,Color::Black);

    for i in 0..depth{
        // if i % (depth/5) == 0{
        //     console_log!("expand {} ",i);
        // } 
        root.expand();
    };



    let (mov, mut future) = root.get_future();

    console_log!("confidence: {} ",root.r/root.n );

    (_,future) = future.get_future();

    console_log!("envisioned response: {}\n n {}\n eval {}",future.data,future.n,eval(future.data,Color::Black));

    return mov.clone();


}


// #[derive(Debug)]
struct SearchNode{

    possible_moves : Vec<PossibleMove>,
    children : Vec<Box <SearchNode>>,
    player:Color,
    data : Board,
    n : f32,
    r : f32,
}


impl SearchNode{

    fn new(board:Board,color:Color)->SearchNode{

        let eval = eval(board,color);
        SearchNode { 
            possible_moves : vec![],
            children: vec![],
            player:color,
            data: board, 
            // eval:eval,
            r : eval,
            n : 1.,
        }
    }

    fn get_future(&self)->(&PossibleMove,&Box<SearchNode>){
        let mut best_eval = 2.;
        let mut best_i = 0;
        for i in 0..self.children.len(){
            let child_eval = self.children[i].r / self.children[i].n;
            if child_eval < best_eval{
                best_eval = child_eval;
                best_i = i;
            }
        }

        return (&self.possible_moves[best_i], &self.children[best_i])
    }

    fn expand(&mut self)->f32{


        let r_delta:f32;

        //is the game over?
        if self.data.state != GameState::Ongoing{
            self.n += 1.;


            r_delta = match self.data.state{
                GameState::Won(winner)=>{
                    if winner == self.player{
                        1.
                    }else{
                        -1.
                    }
                }
                _=>{console_log!("cant see end");
                    panic!()}

            };
            self.r += r_delta;
            return r_delta;
        }


        if self.n == 1. {
            self.possible_moves = self.data.get_legal_moves(5);}

        self.n += 1.;




        if self.possible_moves.len() > self.children.len(){


            let mut new_board = self.data.clone();
            new_board.make_possible_move(&self.possible_moves[self.children.len()]);
            let new_child = SearchNode::new(new_board,self.player.other());
            r_delta = - new_child.r;
            self.children.push(Box::new(new_child));

        }else{

            //find best child to expand

            let count = self.children.len();
            let mut best_ucb = 0.;
            let mut best_i:i32=-1;

            for i in 0..count{
                let child = & self.children[i];
                let mu = child.r/child.n ;
                if mu < -1. || mu > 1.{
                    console_log!("mu error {}",mu)
                }
                let ucb = (1.-mu) + BETA * f32::sqrt(2.*f32::ln(self.n  )/child.n );
                if ucb > best_ucb{
                    best_i = i as i32;
                    best_ucb = ucb;
                }
            }

            //expand best child
            match best_i{
                -1=>{
                    // console_log!("{}",self.children.len());
                    return -1.;
                }
                i=> {

                    let idx = i as usize;
                    let best_child = &mut self.children[idx];

                    r_delta = - best_child.expand();
                }
            }
        }

        self.r += r_delta;

        r_delta

        
    }
}


// #[wasm_bindgen(start)]
// pub fn run(){
//     // let no_node : &Box<SearchNode>;
//     // no_node.expand();
// }

impl Display for SearchNode{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"SN {:?} ",self.data)
    }
}
