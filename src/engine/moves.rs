use super::{EngineMoveContext, EnginePosition};
use super::pst::*;
use super::eval::*;
use kimbo_state::{MoveFlags, Side};

impl EnginePosition {
    /// Making move, updates engine's scores as well
    pub fn make_move(&mut self, m: u16) -> EngineMoveContext {
        let ctx = self.board.make_move(m);
        let ext_ctx = EngineMoveContext {
            ctx,
            mat_mg: self.mat_mg,
            pst_mg: self.pst_mg,
            pst_eg: self.pst_eg,
            phase: self.phase,
            zobrist: self.zobrist,
        };
        let from_idx = (ctx.m & 63) as usize;
        let to_idx = ((ctx.m >> 6) & 63) as usize;
        let flag = ctx.m & MoveFlags::ALL;
        let moved_pc = ctx.moved_pc as usize;
        // side to move after this
        let opp = self.board.side_to_move;
        // side that just moved above
        let side = opp ^ 1;
        // removing original pst bonus
        self.pst_mg[side] -= get_mg_weight(from_idx, side, moved_pc);
        self.pst_eg[side] -= get_eg_weight(from_idx, side, moved_pc);
        self.zobrist ^= self.zobrist_vals.side_hash();
        self.zobrist ^= self.zobrist_vals.piece_hash(from_idx, side, moved_pc);
        if ctx.en_passant_sq > 0 {
            self.zobrist ^= self
                .zobrist_vals
                .en_passant_hash((ctx.en_passant_sq & 7) as usize);
        }
        match flag {
            MoveFlags::QUIET => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, moved_pc);
            }
            MoveFlags::CAPTURE => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                // updated captured piece psts
                let cap_pc = ctx.captured_pc as usize;
                self.mat_mg[opp] -= PIECE_VALS[cap_pc];
                self.pst_mg[opp] -= get_mg_weight(to_idx, opp, cap_pc);
                self.pst_eg[opp] -= get_eg_weight(to_idx, opp, cap_pc);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, moved_pc);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, opp, cap_pc);
                self.phase -= PHASE_VALS[cap_pc];
            }
            MoveFlags::EN_PASSANT => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                let pwn_idx = match opp {
                    Side::WHITE => to_idx + 8,
                    Side::BLACK => to_idx - 8,
                    _ => panic!("Invalid side!"),
                };
                self.mat_mg[opp] -= PIECE_VALS[0];
                self.pst_mg[opp] -= get_mg_weight(pwn_idx, opp, 0);
                self.pst_eg[opp] -= get_eg_weight(pwn_idx, opp, 0);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, moved_pc);
                self.zobrist ^= self.zobrist_vals.piece_hash(pwn_idx, opp, 0);
                self.phase -= PHASE_VALS[0];
            }
            MoveFlags::DBL_PUSH => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, moved_pc);
                self.zobrist ^= self.zobrist_vals.en_passant_hash(to_idx & 7);
            }
            MoveFlags::QS_CASTLE => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                let (idx1, idx2) = match side {
                    Side::WHITE => (0, 3),
                    Side::BLACK => (56, 59),
                    _ => panic!("Invalid side!"),
                };
                self.pst_mg[side] -= get_mg_weight(idx1, side, 3);
                self.pst_eg[side] -= get_eg_weight(idx1, side, 3);
                self.pst_mg[side] += get_mg_weight(idx2, side, 3);
                self.pst_eg[side] += get_eg_weight(idx2, side, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx1, side, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx2, side, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, moved_pc);
            }
            MoveFlags::KS_CASTLE => {
                self.pst_mg[side] += get_mg_weight(to_idx, side, moved_pc);
                self.pst_eg[side] += get_eg_weight(to_idx, side, moved_pc);
                let (idx1, idx2) = match side {
                    Side::WHITE => (7, 5),
                    Side::BLACK => (63, 61),
                    _ => panic!("Invalid side!"),
                };
                self.pst_mg[side] -= get_mg_weight(idx1, side, 3);
                self.pst_eg[side] -= get_eg_weight(idx1, side, 3);
                self.pst_mg[side] += get_mg_weight(idx2, side, 3);
                self.pst_eg[side] += get_eg_weight(idx2, side, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx1, side, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(idx2, side, 3);
                self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, moved_pc);
            }
            _ => {
                if flag < MoveFlags::KNIGHT_PROMO_CAPTURE {
                    let promo_pc: usize = match flag {
                        MoveFlags::KNIGHT_PROMO => 1,
                        MoveFlags::BISHOP_PROMO => 2,
                        MoveFlags::ROOK_PROMO => 3,
                        MoveFlags::QUEEN_PROMO => 4,
                        _ => panic!("Invalid push promotion!"),
                    };
                    self.pst_mg[side] += get_mg_weight(to_idx, side, promo_pc);
                    self.pst_eg[side] += get_eg_weight(to_idx, side, promo_pc);
                    self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, promo_pc);
                    self.phase += PHASE_VALS[promo_pc];
                } else {
                    let promo_pc: usize = match flag {
                        MoveFlags::KNIGHT_PROMO_CAPTURE => 1,
                        MoveFlags::BISHOP_PROMO_CAPTURE => 2,
                        MoveFlags::ROOK_PROMO_CAPTURE => 3,
                        MoveFlags::QUEEN_PROMO_CAPTURE => 4,
                        _ => panic!("Invalid capture promotion!"),
                    };
                    self.pst_mg[side] += get_mg_weight(to_idx, side, promo_pc);
                    self.pst_eg[side] += get_eg_weight(to_idx, side, promo_pc);
                    let cap_pc = ctx.captured_pc as usize;
                    self.mat_mg[opp] -= PIECE_VALS[cap_pc];
                    self.pst_mg[opp] -= get_mg_weight(to_idx, opp, cap_pc);
                    self.pst_eg[opp] -= get_eg_weight(to_idx, opp, cap_pc);
                    self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, side, promo_pc);
                    self.zobrist ^= self.zobrist_vals.piece_hash(to_idx, opp, cap_pc);
                    self.phase += PHASE_VALS[promo_pc];
                    self.phase -= PHASE_VALS[cap_pc];
                }
            }
        }
        let mut changed_castle = ctx.castle_rights & !self.board.castle_rights;
        while changed_castle > 0 {
            let ls1b = changed_castle & changed_castle.wrapping_neg();
            self.zobrist ^= self.zobrist_vals.castle_hash(ctx.castle_rights, ls1b);
            changed_castle &= changed_castle - 1
        }
        ext_ctx
    }

    /// Unmaking move, updates engine's scores as well
    pub fn unmake_move(&mut self, ext_ctx: EngineMoveContext) {
        let ctx = ext_ctx.ctx;
        self.mat_mg = ext_ctx.mat_mg;
        self.pst_mg = ext_ctx.pst_mg;
        self.pst_eg = ext_ctx.pst_eg;
        self.phase = ext_ctx.phase;
        self.zobrist = ext_ctx.zobrist;
        self.board.unmake_move(ctx);
    }
}