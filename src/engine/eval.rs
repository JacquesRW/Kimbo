use crate::hash::pawn::PawnHashTable;
use super::consts::*;
use super::*;

/// Calculating phase
pub fn calculate_phase(pos: &Position) -> i16 {
    let mut phase: i16 = 0;
    for pc in pos.squares {
        phase += PHASE_VALS[pc as usize];
    }
    phase
}

/// Calculating material scores from scratch
pub fn calc_material<const MG: bool>(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for (i, side) in pos.pieces.iter().enumerate() {
        let mut score = 0;
        for (j, piece_val) in (if MG {MG_PC_VALS} else {EG_PC_VALS}).iter().enumerate() {
            let mut piece = side[j];
            while piece > 0 {
                score += piece_val;
                piece &= piece - 1
            }
        }
        scores[i] = score;
    }
    scores
}


/// Calculate midgame piece-square tables from scratch
pub fn calc_pst<const MG: bool>(pos: &Position) -> [i16; 2] {
    let mut scores = [0; 2];
    for (i, side) in pos.pieces.iter().enumerate() {
        let mut score = 0;
        for (j, &pc) in side.iter().enumerate() {
            let mut piece = pc;
            while piece > 0 {
                let idx = ls1b_scan(piece) as usize;
                score += get_weight::<MG>(idx, i, j);
                piece &= piece - 1
            }
        }
        scores[i] = score;
    }
    scores
}

const SIDE_FACTOR: [i16; 3] = [1, -1, 0];

#[inline(always)]
fn taper(phase: i32, mg: i16, eg: i16) -> i16 {
    ((phase * mg as i32 + (TOTALPHASE - phase) * eg as i32) / TOTALPHASE) as i16
}

#[inline(always)]
fn eval_factor(phase: i32, mg: [i16; 2], eg: [i16; 2]) -> i16 {
    let eval_mg = mg[0] - mg[1];
    let eval_eg = eg[0] - eg[1];
    taper(phase, eval_mg, eval_eg)
}

impl Engine {
    /// static evaluation of position
    pub fn static_eval<const STATS: bool>(&mut self, table: Arc<PawnHashTable>) -> i16 {
        let mut phase = self.phase as i32;
        if phase > TOTALPHASE {
            phase = TOTALPHASE
        };
        let mat = eval_factor(phase, self.mat_mg, self.mat_eg);
        let pst = eval_factor(phase, self.pst_mg, self.pst_eg);
        let pwn: i16;
        if let Some(val) = table.get(self.pawnhash) {
            if STATS { self.stats.pwn_hits += 1 }
            pwn = val.score;
        } else {
            if STATS { self.stats.pwn_misses += 1 }
            pwn = self.side_pawn_score(0, phase) - self.side_pawn_score(1, phase);
            table.push(self.pawnhash, pwn);
        }
        SIDE_FACTOR[self.board.side_to_move] * (mat + pst + pwn)
    }

    fn side_pawn_score(&self, side: usize, phase: i32) -> i16 {
        let mut doubled = 0;
        let mut isolated = 0;
        let mut passed = 0;
        let mut chained = 0;
        let mut pawns = self.board.pieces[side][0];
        // doubled and isolated pawns
        for file in 0..8 {
            let count = (FILES[file] & pawns).count_ones();
            doubled += (count > 1) as i16 * count as i16;
            if count > 0 {
                let rail_count = (RAILS[file] & pawns).count_ones();
                isolated += (rail_count == 0) as i16;
            }
        }
        // chained and passed pawns
        let enemies = self.board.pieces[side ^ 1][0];
        while pawns > 0 {
            let ls1b = pawns & pawns.wrapping_neg();
            let pawn = ls1b_scan(ls1b) as usize;
            chained += (CHAINS[pawn] & self.board.pieces[side][0]).count_ones() as i16;
            let enemies_ahead = (IN_FRONT[side][pawn] & enemies).count_ones();
            passed += (enemies_ahead == 0) as i16;
            pawns &= pawns - 1
        }
        // score
        let mg = doubled * DOUBLED_MG + isolated * ISOLATED_MG + passed * PASSED_MG + chained * CHAINED_MG;
        let eg = doubled * DOUBLED_EG + isolated * ISOLATED_EG + passed * PASSED_EG + chained * CHAINED_EG;
        taper(phase, mg, eg)
    }
}
