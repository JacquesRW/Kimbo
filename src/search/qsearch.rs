use super::*;
use super::sorting::{MoveScores, get_next_move};
//use crate::engine::EngineMoveContext;
use kimbo_state::{MoveType, Check, movelist::MoveList};

impl Search {
    /// Quiescence search
    /// 
    /// Constant parameters:
    /// STATS - are debug stats required?
    /// 
    /// Comments:
    /// UCI: implemented for the uci protocol / debug stats
    pub fn quiesce<const STATS: bool>(&mut self, mut alpha: i16, beta: i16) -> i16 {
        // static eval
        let stand_pat = self.position.static_eval();

        // beta pruning
        if stand_pat >= beta {
            return beta;
        }

        // delta pruning
        if stand_pat < alpha - 850 {
            return alpha;
        }

        // improving alpha bound
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // UCI: now will be generating moves, so this node is counted as visited
        self.stats.node_count += 1;
        if STATS { self.stats.qnode_count += 1; }

        // generating captures
        let mut king_checked = Check::None;
        let mut captures = MoveList::default();
        self.position.board.gen_moves::<{ MoveType::CAPTURES }>(&mut king_checked, &mut captures);

        // scoring captures
        let mut move_scores = MoveScores::default();
        self.position.score_captures(&captures, &mut move_scores);

        // going through captures
        while let Some(m) = get_next_move(&mut captures, &mut move_scores) {
            // making move
            let ctx = self.position.make_move(m);

            // getting score
            let score = -self.quiesce::<STATS>(-beta, -alpha);

            // unmaking move
            self.position.unmake_move(ctx);

            // beta pruning
            if score >= beta {
                return beta;
            }

            // improve alpha bound
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
}
