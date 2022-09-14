use crate::hash::search::{Bound, HashResult};

/// Based on a hash result and given search parameters
/// returns Some(value) if pruning is appropriate, else None
pub fn tt_prune(res: HashResult, depth: u8, alpha: i16, beta: i16) -> Option<i16> {
    // TODO test res.depth >= depth - (res.bound == Bound::EXACT) as u8
    // leads to more pruning but short pv lines because 
    // no line is returned when tt pruning happens
    if res.depth >= depth - (res.bound == Bound::EXACT) as u8 {
        match res.bound {
            Bound::EXACT => {
                return Some(res.score);
            },
            Bound::LOWER => {
                if res.score >= beta {
                    return Some(beta);
                }
            },
            Bound::UPPER => {
                if res.score <= alpha {
                    return Some(alpha);
                }
            },
            _ => ()
        }
    }
    None
}
