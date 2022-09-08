use super::*;
use kimbo_state::MoveType;
use std::sync::atomic::Ordering;
use std::time::Instant;
use crate::io::outputs::uci_info;

impl Search {
    /// iterative deepening search
    pub fn go<const TEST: bool>(&mut self) -> u16 {
        let moves = self.position.board.gen_moves::<{ MoveType::ALL }>();
        // creating the initial scored move list with all scores set to 0
        let mut move_list: Vec<(u16, i16)> = Vec::with_capacity(64);
        for m in moves {
            move_list.push((m, 0));
        }
        // loop of iterative deepening, up to preset max depth
        self.stats.start_time = Instant::now();
        for d in 0..self.max_depth {
            if (self.stats.start_time.elapsed().as_millis() as f64) / (self.max_move_time as f64) >= 0.5 
            || (self.stats.node_count as f64) / (self.max_nodes as f64) >= 0.5 
            {
                break;
            }
            
            let score = self.negamax(-MAX, MAX, d + 1, 0);
            self.best_move = self.ttable.get(self.position.zobrist, &mut false).unwrap().best_move;
            
            if self.stop.load(Ordering::Relaxed) || self.stats.node_count > self.max_nodes {
                break;
            }

            let elapsed = self.stats.start_time.elapsed().as_millis() as u64;
            uci_info(d + 1, self.stats.node_count - self.stats.old_count, elapsed - self.stats.old_time, vec![self.best_move], score, self.ttable.filled.load(Ordering::Relaxed), self.ttable.num_entries as u64);
            if TEST {
                println!("entries: {} hits: {}, cutoff hits: {}, move hits: {}", self.ttable.filled.load(Ordering::Relaxed),self.stats.tt_hits.0, self.stats.tt_hits.1, self.stats.tt_hits.2);
            }
            self.stats.old_time = elapsed;
            self.stats.old_count = self.stats.node_count;

            if score == MAX || score == -MAX {
                break;
            }
        }
        if TEST {
            println!("total nodes: {}, mates: {}", self.stats.node_count, self.stats.mates);
        }
        // resetting counts
        self.stats.reset();
        self.age += 1;
        self.best_move
    }
}