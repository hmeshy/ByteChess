use crate::{board::{self, BBPiece}, tuner, util::{self, bb_gs_low_bit, Score}};
const KING_SAFETY_TABLE: [i32; 100] = [
    0,   0,   1,   2,   3,   5,   7,   9,  12,  15,
   18,  22,  26,  30,  35,  39,  44,  50,  56,  62,
   68,  75,  82,  85,  89,  97, 105, 113, 122, 131,
  140, 150, 169, 180, 191, 202, 213, 225, 237, 248,
  260, 272, 283, 295, 307, 319, 330, 342, 354, 366,
  377, 389, 401, 412, 424, 436, 448, 459, 471, 483,
  494, 500, 500, 500, 500, 500, 500, 500, 500, 500,
  500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
  500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
  500, 500, 500, 500, 500, 500, 500, 500, 500, 500
];

pub fn evaluate(board: &board::Board, params: &tuner::EngineParams) -> i32 {
    let phase = board.phase;
    let material_score = material_score(board, params); // done!
    let mobility_score = mobility_score(board, params); // todo
    let king_safety_score = king_safety_score(board, params); // done!
    let king_edge_score = king_edge(board, params); // done!
    let pawn_structure_score = pawn_struct_score(board, params); // done!
    return (material_score + mobility_score + king_safety_score + king_edge_score + pawn_structure_score).taper(phase); 
}
fn material_score(board: &board::Board, params: &tuner::EngineParams) -> Score {
    let mut score = Score::new(0, 0);
    
    let white_pieces = board.bitboards[BBPiece::White as usize];
    let black_pieces = board.bitboards[BBPiece::Black as usize];
    
    // Piece types to evaluate (skip Empty and None if they exist in your enum)
    let piece_types = [
        BBPiece::Pawn as usize,
        BBPiece::Knight as usize,
        BBPiece::Bishop as usize,
        BBPiece::Rook as usize,
        BBPiece::Queen as usize,
        BBPiece::King as usize,
    ];
    
    for &piece_type in &piece_types {
        let white_count = (board.bitboards[piece_type] & white_pieces).count_ones() as i32;
        let black_count = (board.bitboards[piece_type] & black_pieces).count_ones() as i32;
        let piece_diff = white_count - black_count;
        
        score += params.piece_values[piece_type] * piece_diff;
    }
    
    score
}
fn mobility_score(board: &board::Board, params: &tuner::EngineParams) -> Score {
        let mut white_mobility = Score::new(0, 0);
        let mut black_mobility = Score::new(0, 0);
        
        // Pre-calculate commonly used values
        let blockers = board.combined([BBPiece::White, BBPiece::Black], false);
        let white_pieces = board.bitboards[BBPiece::White as usize];
        let black_pieces = board.bitboards[BBPiece::Black as usize];
        
        // Process each piece type once
        for piece_type in 3..8 { // Knight=3, Bishop=4, Rook=5, Queen=6, King=7
            let mobility_weight = params.mobility_values[piece_type];
            
            // White pieces
            let mut white_piece_bb = board.bitboards[piece_type] & white_pieces;
            while white_piece_bb != 0 {
                let square = util::bb_gs_low_bit(&mut white_piece_bb);
                let mobility_count = board.get_mobility_count(piece_type, square, blockers, white_pieces);
                white_mobility += mobility_weight * mobility_count as i32;
            }
            
            // Black pieces  
            let mut black_piece_bb = board.bitboards[piece_type] & black_pieces;
            while black_piece_bb != 0 {
                let square = util::bb_gs_low_bit(&mut black_piece_bb);
                let mobility_count = board.get_mobility_count(piece_type, square, blockers, black_pieces);
                black_mobility += mobility_weight * mobility_count as i32;
            }
        }
        
        white_mobility - black_mobility
    }
fn king_edge(board: &board::Board, params: &tuner::EngineParams) -> Score {
    let white_distance = king_distance_to_corner(board, true);
    let black_distance = king_distance_to_corner(board, false);
    // Return difference (closer to corner = higher penalty)
    params.king_center_bonus * (white_distance - black_distance)
}

fn king_distance_to_corner(board: &board::Board, is_white: bool) -> i32 {
    let king_color = if is_white { BBPiece::White } else { BBPiece::Black };
    
    // Find king position
    let king_bb = board.bitboards[BBPiece::King as usize] & board.bitboards[king_color as usize];
    if king_bb == 0 {
        return 0; // No king (shouldn't happen)
    }
    
    let mut king_bb_copy = king_bb;
    let king_square = bb_gs_low_bit(&mut king_bb_copy);
    let king_file = king_square % 8;
    let king_rank = king_square / 8;
    
    // Calculate distance to each corner and return the minimum
    let corners = [
        (0, 0),  // a1
        (0, 7),  // a8  
        (7, 0),  // h1
        (7, 7),  // h8
    ];
    
    let mut min_distance = 3;
    
    for &(corner_file, corner_rank) in &corners {
        // Use Chebyshev distance (king move distance)
        let distance = (king_file as i32 - corner_file).abs().max(
                      (king_rank as i32 - corner_rank).abs());
        min_distance = min_distance.min(distance);
    }
    
    min_distance
}

fn pawn_struct_score(board: &board::Board, params: &tuner::EngineParams) -> Score {
    let white_pawns = board.combined([BBPiece::Pawn, BBPiece::White], true);
    let black_pawns = board.combined([BBPiece::Pawn, BBPiece::Black], true);
    pawn_evaluation(board, white_pawns, black_pawns, true, params) - pawn_evaluation(board, black_pawns, white_pawns, false, params)
}
fn pawn_evaluation(board: &board::Board, pawn_bb: u64, opp_bb: u64, is_white: bool, params: &tuner::EngineParams) -> Score {
    if pawn_bb == 0 {
        return Score::new(0,0);
    }
    
    let mut score = Score::new(0,0);
    let mut pawns_per_file = [0u8; 8];
    let mut pawn_positions = Vec::new();
    let mut bitboard = pawn_bb;
    
    // First pass: count pawns per file and add advancement bonuses
    while bitboard != 0 {
        let square = bb_gs_low_bit(&mut bitboard);
        let rank = (square / 8) as u8;
        let file = (square % 8) as usize;
        
        pawns_per_file[file] += 1;
        pawn_positions.push((square, rank, file));

        // Pawn advancement bonus =
        let advancement = if is_white {
            2_i32.pow((rank as u32).saturating_sub(1))
        } else {
            2_i32.pow((6u32).saturating_sub(rank as u32))
        };
        score += params.pawn_advance_bonus * (advancement as i32);
    }
    
    // Second pass: evaluate pawn structure
    for &(square, rank, file) in &pawn_positions {
        if is_passed_pawn(square, is_white, opp_bb) {
            let passed_pawn_score = evaluate_passed_pawn(
                square, 
                rank, 
                file, 
                is_white, 
                pawn_bb,
                params
            );
            score += passed_pawn_score;
        }
    }

    for file in 0..8 {
        let pawn_count = pawns_per_file[file];
        if pawn_count == 0 {
            continue;
        }
        
        // Doubled/tripled pawn penalty (exponential)
        if pawn_count > 1 {
            score -= params.doubled_pawn_penalty * (pawn_count as i32 - 1) * (pawn_count as i32 - 1);
        }
        
        // Isolated pawn penalty
        let has_support = (file > 0 && pawns_per_file[file - 1] > 0) || 
                         (file < 7 && pawns_per_file[file + 1] > 0);
        if !has_support {
            score -= params.isolated_pawn_penalty;
        }
    }
    score
}
fn is_passed_pawn(square: usize, is_white: bool, enemy_pawns: u64) -> bool {
    let file = square % 8;
    let rank = square / 8;
    
    // Generate mask for squares that would block this pawn
    let mut blocking_mask = 0u64;
    
    if is_white {
        // For white pawns, check ranks ahead (higher ranks)
        for check_rank in (rank + 1)..8 {
            // Same file
            blocking_mask |= 1u64 << (check_rank * 8 + file);
            // Adjacent files
            if file > 0 {
                blocking_mask |= 1u64 << (check_rank * 8 + file - 1);
            }
            if file < 7 {
                blocking_mask |= 1u64 << (check_rank * 8 + file + 1);
            }
        }
    } else {
        // For black pawns, check ranks ahead (lower ranks)
        for check_rank in 0..rank {
            // Same file
            blocking_mask |= 1u64 << (check_rank * 8 + file);
            // Adjacent files
            if file > 0 {
                blocking_mask |= 1u64 << (check_rank * 8 + file - 1);
            }
            if file < 7 {
                blocking_mask |= 1u64 << (check_rank * 8 + file + 1);
            }
        }
    }
    
    // If no enemy pawns can stop this pawn, it's passed
    (enemy_pawns & blocking_mask) == 0
}
fn evaluate_passed_pawn(
    square: usize,
    rank: u8, 
    file: usize,
    is_white: bool,
    own_pawns: u64,
    params: &tuner::EngineParams
) -> Score {
    let mut score = params.passed_pawn_base;
    
    // Rank bonus - more advanced = more valuable
    let pawn_rank = if is_white { rank } else { 7 - rank };
    score += params.passed_pawn_rank_bonus[pawn_rank as usize];
    
    // Check if pawn is blocked
    let next_square = if is_white {
        if rank < 7 { Some((rank + 1) * 8 + file as u8) } else { None }
    } else {
        if rank > 0 { Some((rank - 1) * 8 + file as u8) } else { None }
    };
    
    // Check if pawn is protected by own pawn
    let protection_squares = if is_white {
        let mut protection = 0u64;
        if rank > 0 {
            if file > 0 {
                protection |= 1u64 << ((rank - 1) * 8 + file as u8 - 1);
            }
            if file < 7 {
                protection |= 1u64 << ((rank - 1) * 8 + file as u8 + 1);
            }
        }
        protection
    } else {
        let mut protection = 0u64;
        if rank < 7 {
            if file > 0 {
                protection |= 1u64 << ((rank + 1) * 8 + file as u8 - 1);
            }
            if file < 7 {
                protection |= 1u64 << ((rank + 1) * 8 + file as u8 + 1);
            }
        }
        protection
    };
    
    if (own_pawns & protection_squares) != 0 {
        score += params.protected_passed_pawn_bonus;
    }
    
    score
}
fn king_safety_score(board: &board::Board, params: &tuner::EngineParams) -> Score {
    let white_score = evaluate_king_safety(board, true, params);
    let black_score = evaluate_king_safety(board, false, params);
    white_score - black_score
}

fn evaluate_king_safety(board: &board::Board, is_white: bool, params: &tuner::EngineParams) -> Score {
    let king_color = if is_white { BBPiece::White } else { BBPiece::Black };
    let enemy_color = if is_white { BBPiece::Black } else { BBPiece::White };
    let blockers = board.combined([BBPiece::White, BBPiece::Black], false);
    // Find king position
    let king_bb = board.bitboards[BBPiece::King as usize] & board.bitboards[king_color as usize];
    if king_bb == 0 {
        return Score::new(0,0); // No king (shouldn't happen)
    }
    
    let mut king_bb_copy = king_bb;
    let king_square = bb_gs_low_bit(&mut king_bb_copy);
    
    let mut attack_units = Score::new(0,0);
    let mut attackers = 0;
    
    // Get king zone (king + surrounding squares)
    let king_zone = get_king_zone(king_square);
    
    // Check attacks from enemy pieces
    for piece_type in 3..7 { // Knight, Bishop, Rook, Queen
        let enemy_pieces = board.bitboards[piece_type] & board.bitboards[enemy_color as usize];
        let mut piece_bb = enemy_pieces;
        
        while piece_bb != 0 {
            let piece_square = bb_gs_low_bit(&mut piece_bb);
            let attacks = board.get_piece_attacks(piece_type, piece_square, blockers);
            
            if attacks & king_zone != 0 {
                attackers += 1;
                let zone_attacks = (attacks & king_zone).count_ones() as i32;
                attack_units += params.attack_weights[piece_type] * zone_attacks;
            }
        }
    }
    
    // Bonus for multiple attackers
    if attackers >= 2 {
        attack_units += params.two_attacker_bonus;
    }
    if attackers >= 3 {
        attack_units += params.multiple_attacker_bonus * (attackers - 1); // More attackers, more bonus;
    }
    
    // Pawn shelter bonus/penalty
    let shelter_penalty = evaluate_pawn_shelter(board, king_square, is_white, params);
    attack_units += shelter_penalty;
    
    // Convert attack units to score using safety table
    Score::new(-KING_SAFETY_TABLE[std::cmp::min(attack_units.mg as usize, 99)], -KING_SAFETY_TABLE[std::cmp::min(attack_units.eg as usize, 99)]) // Negative because this is penalty for our king
}

fn get_king_zone(king_square: usize) -> u64 {
    let mut zone = board::KING_ATTACKS[king_square];
    zone |= 1u64 << king_square; // Include king square itself
    zone
}

fn evaluate_pawn_shelter(board: &board::Board, king_square: usize, is_white: bool, params: &tuner::EngineParams) -> Score {
    let king_file = king_square % 8;
    let king_rank = king_square / 8;
    let mut penalty = Score::new(0,0);
    let own_pawns = if is_white {
        board.combined([BBPiece::Pawn, BBPiece::White], true)
    } else {
        board.combined([BBPiece::Pawn, BBPiece::Black], true)
    };
    
    // Check files around king (king file and adjacent files)
    for file_offset in -1i32..=1i32 {
        let file = (king_file as i32 + file_offset) as usize;
        if file >= 8 { continue; }
        
        let mut has_pawn = false;
        let mut closest_pawn_distance = 8;
        
        // Look for pawns in front of king on this file
        for rank in 0..8 {
            let square = rank * 8 + file;
            if own_pawns & (1u64 << square) != 0 {
                has_pawn = true;
                let distance = if is_white {
                    (rank as i32 - king_rank as i32).abs()
                } else {
                    (king_rank as i32 - rank as i32).abs()
                };
                if distance < closest_pawn_distance {
                    closest_pawn_distance = distance;
                }
            }
        }
        
        if !has_pawn {
            penalty += params.no_pawn_shield_penalty; // No pawn shield on this file
        } else if closest_pawn_distance > 2 {
            penalty += params.far_pawn_penalty * (closest_pawn_distance - 2); // Pawn too far away
        }
    }
    penalty
}