// use crate::bot::choose_move;
use wasm_bindgen::prelude::*;
use crate::{console_log,log};
use crate::Board;
use crate::GameState;
use crate::bot;



#[wasm_bindgen]
pub struct Game{
    board: Board,
    winner: i8,
    succ_move:bool,
}

#[wasm_bindgen]
impl Game{

    pub fn new()->Game{
        Game{board:Board::new(),winner:-1,succ_move:true}
    }

    pub fn get_data(&self)->js_sys::Uint32Array{
        self.board.get_data()
    }

    pub fn make_move(&mut self,start:usize,end:usize,upgrade:u8)->js_sys::Uint32Array{

        if matches!(self.board.state,GameState::Won(_)){
            return self.board.get_data();
        }

        let counter = self.board.counter;
        self.board = self.board.update(start, end, upgrade);
        if counter == self.board.counter{
            self.succ_move = false
        }else{
            self.succ_move = true
        }
        return self.get_data();

    }
    pub fn respond(&mut self)->js_sys::Uint32Array{


        if ! self.succ_move{
            return self.get_data();
        }

        if self.board.get_legal_moves(5).len() == 0{
            console_log!("GAME OVER");
            return self.get_data();
        }

        match self.board.state{
            GameState::Won(color)=>{
                console_log!("{} won.",color);
                self.winner = color.to_num() as i8;
                return self.get_data()
            }
            _=>{}
        }



        let bot_move = bot::choose_move(self.board);

        // console_log!("making bot move");

        self.board.make_possible_move(&bot_move);

        self.get_data()
    }
}
