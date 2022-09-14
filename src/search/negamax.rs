use super::*;
use super::pruning::tt_prune;
use super::sorting::MoveScores;
//use super::sorting::StagedGenerator;
use super::sorting::get_next_move;
use crate::hash::search::Bound;
use kimbo_state::{MoveType, Check, movelist::MoveList};
use std::sync::atomic::Ordering;
use std::cmp::{max, min};

impl Search {
    /// Main search
    /// 
    /// Constant parameters:
    /// ROOT - is this a root (ply = 0) node?
    /// STATS - are debug stats required?
    /// 
    /// Comments:
    /// UCI: implemented for the uci protocol / debug stats
    /// ESSENTIAL: core feature of any engine, also SAFE, no need for ELO testing
    /// SAFE: will not distort search results to incorrect values
    /// UNSAFE: potential to distort search results to be incorrect
    /// JUSTIFICATION: if SAFE, reason why safe, if UNSAFE, reason why included
    pub fn negamax<const ROOT: bool, const STATS: bool>(&mut self, mut alpha: i16, mut beta: i16, depth: u8, ply: u8, pv: &mut Vec<u16>,) -> i16 {
        // UCI: if stop token, abort
        if self.stop.load(Ordering::Relaxed) {
            return 0;
        }

        // UCI: check if nodes or time limits reached
        if self.search_limits_reached() {
            self.stop.store(true, Ordering::Relaxed);
            return 0;
        }

        // UCI: update seldepth (due to extensions)
        if ply > self.stats.seldepth {
            self.stats.seldepth = ply;
        }

        // SAFE: mate distance pruning
        // JUSTIFICATION: only applies when a mate score is returned in the previous
        // child node of the parent node, and the cutoff would be caused later anyway
        alpha = max(alpha, -MAX_SCORE + ply as i16);
        beta = min(beta, MAX_SCORE - ply as i16 - 1);
        if alpha >= beta {
            return alpha
        }

        // ESSENTIAL: depth = 0 quiescence search
        if depth == 0 {
            return self.quiesce::<STATS>(alpha, beta);
        }

        // hash table stuff
        let zobrist = self.position.zobrist;
        let mut hash_move = 0;

        // dictates if hash table will be written to at end of this node
        // default write if no hash entry found
        let mut write_to_hash = true;

        // probing hash table
        if let Some(res) = self.ttable.get(zobrist, ply, self.age) {
            if STATS { self.stats.tt_hits += 1; }

            // hash entry found, only write to hash table if this depth search  
            // is deeper than the depth of the hash entry
            write_to_hash = depth > res.depth;

            // ESSENTIAL: hash move for move ordering
            hash_move = res.best_move;

            // UNSAFE: hash score pruning (no pruning on root)
            // JUSTIFICATION: >99% of hash moves are valid moves on average
            // so unlikely to effect search results too much
            if !ROOT {
                if let Some(score) = tt_prune(res, depth, alpha, beta) {
                    if STATS { self.stats.tt_prunes += 1 }
                    pv.push(hash_move);
                    return score;
                }
            }
        }

        // UCI: now will be generating moves, so this node is counted as visited
        self.stats.node_count += 1;

        // staged move generation: captures first
        let mut king_checked = Check::None;
        let mut moves = MoveList::default();
        self.position.board.gen_moves::<{ MoveType::ALL }>(&mut king_checked, &mut moves);
        let king_in_check = king_checked != Check::None;

        // ESSENTIAL: checking for checkmate/stalemate
        if moves.is_empty() {
            return king_in_check as i16 * (-MAX_SCORE + ply as i16);
        }

        // SAFE: check extensions
        // JUSTIFICATION: not given higher priority than any other searches at this
        // depth (recorded in hash table as same depth as other nodes at this ply)
        // so acts like a more accurate qsearch
        let ext = king_in_check as u8;

        // ESSENTIAL: move scoring for move ordering
        let mut move_hit: bool = false;
        let mut move_scores = MoveScores::default();
        self.position.score_moves(&moves, &mut move_scores, hash_move, &mut move_hit);
        if STATS && move_hit {
            self.stats.tt_move_hits += 1;
        }
        
        // initialising stuff for going through moves
        let mut best_move = 0;
        let mut best_score = -MAX_SCORE;
        let mut bound: u8 = Bound::UPPER;

        // going through moves
        while let Some(m) = get_next_move(&mut moves, &mut move_scores) {

            // making move
            let ctx = self.position.make_move(m);

            // scoring move and getting the pv for it
            let mut sub_pv = Vec::new();
            let score = -self.negamax::<false, STATS>(-beta, -alpha, depth - 1 + ext, ply + 1, &mut sub_pv);

            // unmaking move
            self.position.unmake_move(ctx);

            // ESSENTIAL: alpha improvements
            if score > best_score {
                // update best move and score
                best_score = score;
                best_move = m;

                // improve alpha
                if score > alpha {
                    alpha = score;
                    bound = Bound::EXACT;
                    update_pv(pv, m, &mut sub_pv);
                } 
            }

            // ESSENTIAL: beta pruning
            if score >= beta {
                bound = Bound::LOWER;
                break;
            } 
        }

        // writing to hash table
        if write_to_hash {
            self.ttable.push(zobrist, best_move, depth, self.age, bound, best_score, ply);
        }

        // return best score
        best_score
    }
}
