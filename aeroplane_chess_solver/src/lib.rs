use smallvec::SmallVec;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Board {
    hangar: u8,
    flying: SmallVec<[Chess; 4]>,
    done: u8,
}

#[wasm_bindgen]
impl Board {
    pub fn new() -> Self {
        Self {
            hangar: 4,
            flying: SmallVec::new_const(),
            done: 0,
        }
    }

    pub fn hangar(&self) -> u8 {
        self.hangar
    }

    pub fn flying_len(&self) -> usize {
        self.flying.len()
    }

    pub fn flying(&self) -> Box<[Chess]> {
        self.flying.clone().into_boxed_slice()
    }

    pub fn done(&self) -> u8 {
        self.done
    }

    pub fn hangar_empty(&self) -> bool {
        self.hangar == 0
    }
}

#[wasm_bindgen]
impl Board {
    pub fn takeoff(&mut self, n: u8) {
        assert!(self.hangar > 0);

        self.hangar -= 1;

        let mut chess = Chess::new();
        chess.advance(n);
        self.flying.push(chess);
        self.flying.sort_by_key(|it| it.pos);
        self.flying.reverse();
    }

    pub fn advance(&mut self, idx: usize, n: u8) {
        let chess = &mut self.flying[idx];
        chess.advance(n);
        if chess.done() {
            self.flying.remove(idx);
            self.done += 1;
        }
        self.flying.sort_by_key(|it| it.pos);
        self.flying.reverse();
    }

    pub fn ret(&mut self, idx: usize) {
        self.flying.remove(idx);
        self.hangar += 1;
    }

    pub fn preview(&self, n: u8) -> AdvancePreviews {
        fn uniq_step<I: IntoIterator<Item = u8>>(it: I) -> i8 {
            it.into_iter()
                .fold(0_u8, |acc, it| acc | (1 << it))
                .count_ones() as i8
        }

        fn uniq_dist<T: Into<Chess>, I: IntoIterator<Item = T>>(
            it: I,
        ) -> i8 {
            uniq_step(
                it.into_iter()
                    .filter_map(|it| it.into().distance_to_next_jump()),
            )
        }

        fn uniq_done<T: Into<Chess>, I: IntoIterator<Item = T>>(
            it: I,
        ) -> i8 {
            uniq_step(
                it.into_iter()
                    .filter_map(|it| it.into().distance_to_done()),
            )
        }

        macro_rules! chess_with {
            ($flying:expr, $into_iter:expr) => {
                $flying.iter().copied().chain($into_iter.into_iter())
            };
        }

        macro_rules! flying_with {
            ($into_iter:expr) => {
                chess_with!(self.flying, $into_iter)
            };
        }

        let takeoff = if !self.hangar_empty() {
            let mut hangar = Chess::new();

            let prev = uniq_dist(flying_with!([hangar]));

            let will_jump = hangar.total_advance(n) > (n as i8);
            hangar.advance(n);

            let next = if self.hangar > 1 {
                uniq_dist(flying_with!([hangar, Chess::new()]))
            } else {
                uniq_dist(flying_with!([hangar]))
            };

            Some(AdvancePreview {
                pos: n,
                will_jump,
                will_enter_end: hangar.at_end(),
                will_done: hangar.done(),
                jump_change: next - prev,
                done_change: 0,
            })
        } else {
            None
        };

        let prev = if !self.hangar_empty() {
            uniq_dist(flying_with!([Chess::new()]))
        } else {
            uniq_dist(&self.flying)
        };
        let prev_done = uniq_done(&self.flying);

        let flying = self
            .flying
            .iter()
            .copied()
            .enumerate()
            .map(|(idx, mut chess)| {
                let will_jump = chess.total_advance(n) > (n as i8);
                chess.advance(n);

                let mut flying = self.flying.clone();
                flying[idx] = chess;
                let next = if self.hangar_empty() {
                    uniq_dist(&flying)
                } else {
                    uniq_dist(chess_with!(flying, [Chess::new()]))
                };
                let next_done = uniq_done(&flying);

                AdvancePreview {
                    pos: chess.pos,
                    will_jump,
                    will_enter_end: chess.at_end(),
                    will_done: chess.done(),
                    jump_change: if chess
                        .distance_to_next_jump()
                        .is_some()
                    {
                        next - prev
                    } else {
                        0
                    },
                    done_change: next_done - prev_done,
                }
            })
            .collect();

        AdvancePreviews { flying, takeoff }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Chess {
    pos: u8,
}

#[wasm_bindgen]
impl Chess {
    pub fn pos(&self) -> u8 {
        self.pos
    }
}

impl Chess {
    pub fn new() -> Self {
        Self { pos: 0 }
    }

    pub fn total_advance(&self, n: u8) -> i8 {
        assert!(self.pos < 56);

        let mut new_pos = self.pos + n;

        if new_pos > 56 {
            // backtrack
            return 2 * 56 - (new_pos as i8) - self.pos as i8;
        } else if new_pos == 56 {
            return n as i8;
        }

        // not in range where have shortcut/jump
        if !(2..50).contains(&new_pos) {
            return n as i8;
        }

        if new_pos == 18 {
            // shortcut
            new_pos += 30 - 18 + 4;
        } else if (new_pos - 2).is_multiple_of(4) {
            // jump
            new_pos += 4;
            if new_pos == 18 {
                // shortcut
                new_pos += 30 - 18;
            }
        }

        (new_pos - self.pos) as i8
    }

    pub fn advance(&mut self, n: u8) -> i8 {
        let total = self.total_advance(n);
        self.pos = self.pos.wrapping_add_signed(total);
        total
    }

    pub fn at_end(&self) -> bool {
        self.pos >= 51
    }

    pub fn done(&self) -> bool {
        self.pos == 56
    }

    pub fn distance_to_done(&self) -> Option<u8> {
        if self.at_end() {
            Some(56 - self.pos)
        } else {
            None
        }
    }

    pub fn distance_to_next_jump(&self) -> Option<u8> {
        if self.pos < 2 {
            Some(2 - self.pos)
        } else if self.pos < 46 {
            Some(4 - (self.pos - 2) % 4)
        } else {
            None
        }
    }
}

impl Default for Chess {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Chess> for Chess {
    fn from(value: &Chess) -> Self {
        *value
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct AdvancePreviews {
    flying: SmallVec<[AdvancePreview; 4]>,
    takeoff: Option<AdvancePreview>,
}

#[wasm_bindgen]
impl AdvancePreviews {
    #[wasm_bindgen(getter)]
    pub fn flying(&self) -> Box<[AdvancePreview]> {
        self.flying.clone().into_boxed_slice()
    }

    #[wasm_bindgen(getter)]
    pub fn takeoff(&self) -> Option<AdvancePreview> {
        self.takeoff
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct AdvancePreview {
    pub pos: u8,
    pub will_jump: bool,
    pub will_enter_end: bool,
    pub will_done: bool,
    pub jump_change: i8,
    pub done_change: i8,
}
