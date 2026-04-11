// Texel Tuning Implementation for Byte
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::util::{self, Score};
use crate::tunereval::{self, evaluate};

// Global engine parameters that the evaluation function will use
static mut CURRENT_ENGINE_PARAMS: Option<EngineParams> = None;
// Training position with known result
#[derive(Debug, Clone)]
pub struct TrainingPosition {
    pub fen: String,
    pub result: f64, // 1.0 = white win, 0.5 = draw, 0.0 = black loss
}
// All parameters we want to tune
#[derive(Debug, Clone)]
pub struct TunableParams {
    // Piece values (mg, eg)
    pub pawn_mg: i32,
    pub pawn_eg: i32,
    pub knight_mg: i32,
    pub knight_eg: i32,
    pub bishop_mg: i32,
    pub bishop_eg: i32,
    pub rook_mg: i32,
    pub rook_eg: i32,
    pub queen_mg: i32,
    pub queen_eg: i32,
    
    // Mobility weights
    pub knight_mobility_mg: i32,
    pub knight_mobility_eg: i32,
    pub bishop_mobility_mg: i32,
    pub bishop_mobility_eg: i32,
    pub rook_mobility_mg: i32,
    pub rook_mobility_eg: i32,
    pub queen_mobility_mg: i32,
    pub queen_mobility_eg: i32,
    pub king_mobility_mg: i32,
    pub king_mobility_eg: i32,
    
    // Positional factors
    pub king_center_mg: i32,
    pub king_center_eg: i32,
    pub doubled_pawn_penalty_mg: i32,
    pub doubled_pawn_penalty_eg: i32,
    pub isolated_pawn_penalty_mg: i32,
    pub isolated_pawn_penalty_eg: i32,
    pub pawn_advance_bonus_mg: i32,
    pub pawn_advance_bonus_eg: i32,
    pub passed_pawn_mg: i32,
    pub passed_pawn_eg: i32,
    pub pp_rank_2_mg: i32,
    pub pp_rank_2_eg: i32,
    pub pp_rank_3_mg: i32, 
    pub pp_rank_3_eg: i32,
    pub pp_rank_4_mg: i32,
    pub pp_rank_4_eg: i32,
    pub pp_rank_5_mg: i32,
    pub pp_rank_5_eg: i32,
    pub pp_rank_6_mg: i32,
    pub pp_rank_6_eg: i32,
    pub pp_rank_7_mg: i32,
    pub pp_rank_7_eg: i32,
    pub protected_passed_pawn_mg: i32,
    pub protected_passed_pawn_eg: i32,
    pub two_attackers_bonus_mg: i32,
    pub two_attackers_bonus_eg: i32,
    pub multiple_attackers_bonus_mg: i32,
    pub multiple_attackers_bonus_eg: i32,
    pub bishop_attack_bonus_mg: i32,
    pub bishop_attack_bonus_eg: i32,
    pub knight_attack_bonus_mg: i32,
    pub knight_attack_bonus_eg: i32,
    pub rook_attack_bonus_mg: i32,
    pub rook_attack_bonus_eg: i32,
    pub queen_attack_bonus_mg: i32,
    pub queen_attack_bonus_eg: i32,
    pub no_pawn_shield_penalty_mg: i32,
    pub no_pawn_shield_penalty_eg: i32,
    pub far_pawn_penalty_mg: i32,
    pub far_pawn_penalty_eg: i32,
    pub king_safety_table: [i32; 100],
}
impl TunableParams {
    // Initialize with current engine values
    pub fn baseline() -> Self {
        TunableParams {
            pawn_mg: 73, pawn_eg: 109,
            knight_mg: 306, knight_eg: 314,
            bishop_mg: 362, bishop_eg: 325,
            rook_mg: 457, rook_eg: 625,
            queen_mg: 1100, queen_eg: 1010,
            
            knight_mobility_mg: 9, knight_mobility_eg: 11,
            bishop_mobility_mg: 4, bishop_mobility_eg: 10,
            rook_mobility_mg: 4, rook_mobility_eg: 6,
            queen_mobility_mg: -1, queen_mobility_eg: 12,
            king_mobility_mg: -11, king_mobility_eg: 13,
            
            king_center_mg: -18, king_center_eg: 19,            
            doubled_pawn_penalty_mg: 3, doubled_pawn_penalty_eg: 24,
            isolated_pawn_penalty_mg: 12, isolated_pawn_penalty_eg: 16,
            pawn_advance_bonus_mg: 2, pawn_advance_bonus_eg: 1,
            passed_pawn_mg: -19, passed_pawn_eg: 59,

            pp_rank_2_mg: -2, pp_rank_2_eg: -38,
            pp_rank_3_mg: -8, pp_rank_3_eg: -42,
            pp_rank_4_mg: -3, pp_rank_4_eg: -18,
            pp_rank_5_mg: 18, pp_rank_5_eg: 15,
            pp_rank_6_mg: 35, pp_rank_6_eg: 92,
            pp_rank_7_mg: 19, pp_rank_7_eg: 155,

            protected_passed_pawn_mg: 26, protected_passed_pawn_eg: -1,
            two_attackers_bonus_mg: 3, two_attackers_bonus_eg: 1,
            multiple_attackers_bonus_mg: 3, multiple_attackers_bonus_eg: 0,
            bishop_attack_bonus_mg: 5, bishop_attack_bonus_eg: 0,
            knight_attack_bonus_mg: 3, knight_attack_bonus_eg: 0,
            rook_attack_bonus_mg: 3, rook_attack_bonus_eg: 0,
            queen_attack_bonus_mg: 4, queen_attack_bonus_eg: 3,
            no_pawn_shield_penalty_mg: 8, no_pawn_shield_penalty_eg: 0,
            far_pawn_penalty_mg: 2, far_pawn_penalty_eg: 3,
            king_safety_table: [
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
            ],
        }
    }
    pub fn to_engine_params(&self) -> EngineParams {
        EngineParams {
            piece_values: [
                Score::new(0,0),      // Empty
                Score::new(0,0),      // None  
                Score::new(self.pawn_mg, self.pawn_eg),
                Score::new(self.knight_mg, self.knight_eg),
                Score::new(self.bishop_mg, self.bishop_eg),
                Score::new(self.rook_mg, self.rook_eg),
                Score::new(self.queen_mg, self.queen_eg),
                Score::new(100000,100000) // King (never changes)
            ],
            mobility_values: [
                Score::new(0,0), Score::new(0,0), Score::new(0,0),
                Score::new(self.knight_mobility_mg, self.knight_mobility_eg),
                Score::new(self.bishop_mobility_mg, self.bishop_mobility_eg),
                Score::new(self.rook_mobility_mg, self.rook_mobility_eg),
                Score::new(self.queen_mobility_mg,self.queen_mobility_eg), // Queen mobility (not tuned)
                Score::new(self.king_mobility_mg, self.king_mobility_eg)
            ],
            king_center_bonus: Score::new(self.king_center_mg, self.king_center_eg),
            doubled_pawn_penalty: Score::new(self.doubled_pawn_penalty_mg, self.doubled_pawn_penalty_eg),
            isolated_pawn_penalty: Score::new(self.isolated_pawn_penalty_mg, self.isolated_pawn_penalty_eg),
            pawn_advance_bonus: Score::new(self.pawn_advance_bonus_mg, self.pawn_advance_bonus_eg),
            passed_pawn_base: Score::new(self.passed_pawn_mg, self.passed_pawn_eg),
            passed_pawn_rank_bonus: [
                Score::new(0, 0),
                Score::new(self.pp_rank_2_mg, self.pp_rank_2_eg),
                Score::new(self.pp_rank_3_mg, self.pp_rank_3_eg),
                Score::new(self.pp_rank_4_mg, self.pp_rank_4_eg),
                Score::new(self.pp_rank_5_mg, self.pp_rank_5_eg),
                Score::new(self.pp_rank_6_mg, self.pp_rank_6_eg),
                Score::new(self.pp_rank_7_mg, self.pp_rank_7_eg),
                Score::new(0, 0),
            ],
            protected_passed_pawn_bonus: Score::new(self.protected_passed_pawn_mg, self.protected_passed_pawn_eg),
            two_attacker_bonus: Score::new(self.two_attackers_bonus_mg, self.two_attackers_bonus_eg),
            multiple_attacker_bonus: Score::new(self.multiple_attackers_bonus_mg, self.multiple_attackers_bonus_eg),
            attack_weights: [
                Score::from_single(0), Score::from_single(0), Score::from_single(0),
                Score::new(self.knight_attack_bonus_mg, self.knight_attack_bonus_eg),
                Score::new(self.bishop_attack_bonus_mg, self.bishop_attack_bonus_eg),
                Score::new(self.rook_attack_bonus_mg, self.rook_attack_bonus_eg),
                Score::new(self.queen_attack_bonus_mg, self.queen_attack_bonus_eg),
                Score::from_single(0)
            ],
            no_pawn_shield_penalty: Score::new(self.no_pawn_shield_penalty_mg, self.no_pawn_shield_penalty_eg),
            far_pawn_penalty: Score::new(self.far_pawn_penalty_mg, self.far_pawn_penalty_eg),
            king_safety_table: self.king_safety_table,
        }
    }
}
pub struct TexelTuner {
    pub positions: Vec<TrainingPosition>,
    pub params: TunableParams,
    pub learning_rate: f64,
    pub k: f64, // Scaling factor for sigmoid
}
impl TexelTuner {
    pub fn new(positions_file: &str, max_positions: Option<usize>) -> Result<Self, Box<dyn std::error::Error>> {
        let positions = Self::load_positions(positions_file, max_positions.unwrap_or(10_000))?;
        let k = Self::find_optimal_k(&positions)?;
        
        Ok(TexelTuner {
            positions,
            params: TunableParams::baseline(),
            learning_rate: 0.1,
            k,
        })
    }

    fn load_positions(filename: &str, max_positions: usize) -> Result<Vec<TrainingPosition>, Box<dyn std::error::Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut positions = Vec::new();
        let mut count = 0;
        for line in reader.lines() {
            if count >= max_positions {
                break;
            }
            
            let line = line?;
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Try different formats:
            
            // Format 1: "fen [result]" (your current format)
            if let Some(bracket_pos) = line.find('[') {
                let fen = line[..bracket_pos].trim().to_string();
                if let Some(end_bracket) = line[bracket_pos + 1..].find(']') {
                    let result_str = &line[bracket_pos + 1..bracket_pos + 1 + end_bracket];
                    if let Ok(result) = result_str.trim().parse::<f64>() {
                        positions.push(TrainingPosition { fen, result });
                        count += 1;
                    }
                }
                continue;
            }
        }

        println!("Loaded {} training positions from {}", positions.len(), filename);
        Ok(positions)
    }

    // Find optimal K value for sigmoid function
    fn find_optimal_k(positions: &[TrainingPosition]) -> Result<f64, Box<dyn std::error::Error>> {
        // Test different K values to minimize error with current evaluation
        let mut best_k = 1.4;
        let mut best_error = f64::INFINITY;
        
        for k_test in [0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1.0,1.1,1.2,1.3,1.4,1.5,1.6,1.7,1.8,1.9,2.0] {
            let mut total_error = 0.0;
            let mut count = 0;
            
            for pos in positions.iter() { // Sample for speed
                if let Ok(eval) = evaluate_fen(TunableParams::baseline().to_engine_params(), &pos.fen) {
                    let predicted = sigmoid(k_test * eval as f64);
                    let error = (predicted - pos.result).powi(2);
                    total_error += error;
                    count += 1;
                }
            }
            
            if count > 0 {
                let avg_error = total_error / count as f64;
                println!("Avg error {:.6}", avg_error);
                if avg_error < best_error {
                    best_error = avg_error;
                    best_k = k_test;
                }
            }
        }
        
        println!("Optimal K value: {:.3} (error: {:.6})", best_k, best_error);
        Ok(best_k)
    }
    // Main tuning loop
    pub fn tune(&mut self, epochs: usize) {
        println!("Starting Texel tuning for {} epochs", epochs);
        
        for epoch in 0..epochs {
            let initial_error = self.compute_error();
            
            // Compute gradients and update parameters
            self.gradient_descent_step();
            
            let final_error = self.compute_error();            
            if epoch % 10 == 0 {
                println!("Epoch {}: Error {:.6} -> {:.6} (improvement: {:.6})", 
                    epoch, initial_error, final_error, initial_error - final_error);
                
                // Print some current parameter values
                if epoch % 50 == 0 {
                self.print_results(false);
                }
            }
            
            // Early stopping if improvement is minimal
            if (initial_error - final_error).abs() == 0.0 {
                println!("Converged at epoch {}", epoch);
                        break;
            }
            // If error increased (worse performance), reduce learning rate
            if final_error > initial_error {
                self.learning_rate *= 0.9; // Reduce learning rate by 10%
                println!("Error increased, reducing learning rate to {:.4}", self.learning_rate);
            }
        }
        
        println!("Final tuned parameters:");
        self.print_results(true);
    }
    // Compute mean squared error across all positions
    fn compute_error(&self) -> f64 {
        let mut total_error = 0.0;
        let mut count = 0;
        
        // Set engine to use our current parameters
        let engine_params = self.params.to_engine_params();
        set_engine_params(engine_params);
        
        for pos in &self.positions {
            if let Ok(eval) = evaluate_fen(engine_params, &pos.fen) {
                let predicted = sigmoid(self.k * eval as f64);
                let error = (predicted - pos.result).powi(2);
                total_error += error;
                count += 1;
            }
        }
        
        if count > 0 {
            total_error / count as f64
        } else {
            f64::INFINITY
        }
    }
   // Perform one gradient descent step
    fn gradient_descent_step(&mut self) {
        let base_error = self.compute_error();
        
        // Material values
        self.update_param_by_field("pawn_mg", base_error);
        self.update_param_by_field("pawn_eg", base_error);
        self.update_param_by_field("knight_mg", base_error);
        self.update_param_by_field("knight_eg", base_error);
        self.update_param_by_field("bishop_mg", base_error);
        self.update_param_by_field("bishop_eg", base_error);
        self.update_param_by_field("rook_mg", base_error);
        self.update_param_by_field("rook_eg", base_error);
        self.update_param_by_field("queen_mg", base_error);
        self.update_param_by_field("queen_eg", base_error);

        // Mobility weights
        /*self.update_param_by_field("knight_mobility_mg", base_error);
        self.update_param_by_field("knight_mobility_eg", base_error);
        self.update_param_by_field("bishop_mobility_mg", base_error);
        self.update_param_by_field("bishop_mobility_eg", base_error);
        self.update_param_by_field("rook_mobility_mg", base_error);
        self.update_param_by_field("rook_mobility_eg", base_error);
        self.update_param_by_field("queen_mobility_mg", base_error);
        self.update_param_by_field("queen_mobility_eg", base_error);
        self.update_param_by_field("king_mobility_mg", base_error);
        self.update_param_by_field("king_mobility_eg", base_error);

        // Positional factors
        self.update_param_by_field("king_center_mg", base_error);
        self.update_param_by_field("king_center_eg", base_error);
        self.update_param_by_field("doubled_pawn_penalty_mg", base_error);
        self.update_param_by_field("doubled_pawn_penalty_eg", base_error);
        self.update_param_by_field("isolated_pawn_penalty_mg", base_error);
        self.update_param_by_field("isolated_pawn_penalty_eg", base_error);
        self.update_param_by_field("pawn_advance_bonus_mg", base_error);
        self.update_param_by_field("pawn_advance_bonus_eg", base_error);
        self.update_param_by_field("passed_pawn_mg", base_error);
        self.update_param_by_field("passed_pawn_eg", base_error);
        self.update_param_by_field("pp_rank_2_mg", base_error);
        self.update_param_by_field("pp_rank_2_eg", base_error);
        self.update_param_by_field("pp_rank_3_mg", base_error); 
        self.update_param_by_field("pp_rank_3_eg", base_error);
        self.update_param_by_field("pp_rank_4_mg", base_error);
        self.update_param_by_field("pp_rank_4_eg", base_error);
        self.update_param_by_field("pp_rank_5_mg", base_error);
        self.update_param_by_field("pp_rank_5_eg", base_error);
        self.update_param_by_field("pp_rank_6_mg", base_error);
        self.update_param_by_field("pp_rank_6_eg", base_error);
        self.update_param_by_field("pp_rank_7_mg", base_error);
        self.update_param_by_field("pp_rank_7_eg", base_error);
        self.update_param_by_field("protected_passed_pawn_mg", base_error);
        self.update_param_by_field("protected_passed_pawn_eg", base_error);
        self.update_param_by_field("two_attackers_bonus_mg", base_error);
        self.update_param_by_field("two_attackers_bonus_eg", base_error);
        self.update_param_by_field("multiple_attackers_bonus_mg", base_error);
        self.update_param_by_field("multiple_attackers_bonus_eg", base_error);
        self.update_param_by_field("bishop_attack_bonus_mg", base_error);
        self.update_param_by_field("bishop_attack_bonus_eg", base_error);
        self.update_param_by_field("knight_attack_bonus_mg", base_error);
        self.update_param_by_field("knight_attack_bonus_eg", base_error);
        self.update_param_by_field("rook_attack_bonus_mg", base_error);
        self.update_param_by_field("rook_attack_bonus_eg", base_error);
        self.update_param_by_field("queen_attack_bonus_mg", base_error);
        self.update_param_by_field("queen_attack_bonus_eg", base_error);
        self.update_param_by_field("no_pawn_shield_penalty_mg", base_error);
        self.update_param_by_field("no_pawn_shield_penalty_eg", base_error);
        self.update_param_by_field("far_pawn_penalty_mg", base_error);
        self.update_param_by_field("far_pawn_penalty_eg", base_error);
        
        // King safety table (tune first 60 values)
        for i in 0..60 {
            let field_name = format!("king_safety_table_{}", i);
            self.update_param_by_field(&field_name, base_error);
        }
        */
    }

    // Update single parameter by field name
    fn update_param_by_field(&mut self, field_name: &str, base_error: f64) {
        let original_value = self.get_param_value(field_name);
        
        // Use adaptive delta based on parameter value for better numerical stability
        let adaptive_delta = if original_value.abs() > 10 {
            (original_value.abs() as f64 * 0.01).max(1.0) as i32
        } else {
            1 // Default delta for small values
        };
        
        // Test positive change
        self.set_param_value(field_name, original_value + adaptive_delta);
        let pos_error = self.compute_error();
        
        // Test negative change
        self.set_param_value(field_name, original_value - adaptive_delta);
        let neg_error = self.compute_error();
        
        // Compute gradient
        let gradient = (pos_error - neg_error) / (2.0 * adaptive_delta as f64);
        
        // Reset to original value
        self.set_param_value(field_name, original_value);
        
        // Update parameter using gradient descent with adaptive step size
        let step_size = if original_value.abs() > 100 {
            (gradient.abs() * self.learning_rate * 10.0) as i32
        } else if original_value.abs() > 10 {
            (gradient.abs() * self.learning_rate * 5.0) as i32
        } else {
            (gradient.abs() * self.learning_rate) as i32
        };
        
        let mut new_value = original_value;
        if gradient > 0.0 && neg_error < pos_error {
            new_value -= step_size.max(1);
        } else if gradient < 0.0 && pos_error < neg_error {
            new_value += step_size.max(1);
        }
        
        self.set_param_value(field_name, new_value);
    }

    // Helper function to get parameter value by field name
    fn get_param_value(&self, field_name: &str) -> i32 {
        if let Some(index_str) = field_name.strip_prefix("king_safety_table_") {
            if let Ok(index) = index_str.parse::<usize>() {
                if index < 100 {
                    return self.params.king_safety_table[index];
                }
            }
        }
        match field_name {
            "pawn_mg" => self.params.pawn_mg,
            "pawn_eg" => self.params.pawn_eg,
            "knight_mg" => self.params.knight_mg,
            "knight_eg" => self.params.knight_eg,
            "bishop_mg" => self.params.bishop_mg,
            "bishop_eg" => self.params.bishop_eg,
            "rook_mg" => self.params.rook_mg,
            "rook_eg" => self.params.rook_eg,
            "queen_mg" => self.params.queen_mg,
            "queen_eg" => self.params.queen_eg,
            "knight_mobility_mg" => self.params.knight_mobility_mg,
            "knight_mobility_eg" => self.params.knight_mobility_eg,
            "bishop_mobility_mg" => self.params.bishop_mobility_mg,
            "bishop_mobility_eg" => self.params.bishop_mobility_eg,
            "rook_mobility_mg" => self.params.rook_mobility_mg,
            "rook_mobility_eg" => self.params.rook_mobility_eg,
            "queen_mobility_mg" => self.params.queen_mobility_mg,
            "queen_mobility_eg" => self.params.queen_mobility_eg,
            "king_mobility_mg" => self.params.king_mobility_mg,
            "king_mobility_eg" => self.params.king_mobility_eg,
            "king_center_mg" => self.params.king_center_mg,
            "king_center_eg" => self.params.king_center_eg,
            "doubled_pawn_penalty_mg" => self.params.doubled_pawn_penalty_mg,
            "doubled_pawn_penalty_eg" => self.params.doubled_pawn_penalty_eg,
            "isolated_pawn_penalty_mg" => self.params.isolated_pawn_penalty_mg,
            "isolated_pawn_penalty_eg" => self.params.isolated_pawn_penalty_eg,
            "pawn_advance_bonus_mg" => self.params.pawn_advance_bonus_mg,
            "pawn_advance_bonus_eg" => self.params.pawn_advance_bonus_eg,
            "passed_pawn_mg" => self.params.passed_pawn_mg,
            "passed_pawn_eg" => self.params.passed_pawn_eg,
            "pp_rank_2_mg" => self.params.pp_rank_2_mg,
            "pp_rank_2_eg" => self.params.pp_rank_2_eg,
            "pp_rank_3_mg" => self.params.pp_rank_3_mg,
            "pp_rank_3_eg" => self.params.pp_rank_3_eg,
            "pp_rank_4_mg" => self.params.pp_rank_4_mg,
            "pp_rank_4_eg" => self.params.pp_rank_4_eg,
            "pp_rank_5_mg" => self.params.pp_rank_5_mg,
            "pp_rank_5_eg" => self.params.pp_rank_5_eg,
            "pp_rank_6_mg" => self.params.pp_rank_6_mg,
            "pp_rank_6_eg" => self.params.pp_rank_6_eg,
            "pp_rank_7_mg" => self.params.pp_rank_7_mg,
            "pp_rank_7_eg" => self.params.pp_rank_7_eg,
            "protected_passed_pawn_mg" => self.params.protected_passed_pawn_mg,
            "protected_passed_pawn_eg" => self.params.protected_passed_pawn_eg,
            "two_attackers_bonus_mg" => self.params.two_attackers_bonus_mg,
            "two_attackers_bonus_eg" => self.params.two_attackers_bonus_eg,
            "multiple_attackers_bonus_mg" => self.params.multiple_attackers_bonus_mg,
            "multiple_attackers_bonus_eg" => self.params.multiple_attackers_bonus_eg,
            "bishop_attack_bonus_mg" => self.params.bishop_attack_bonus_mg,
            "bishop_attack_bonus_eg" => self.params.bishop_attack_bonus_eg,
            "knight_attack_bonus_mg" => self.params.knight_attack_bonus_mg,
            "knight_attack_bonus_eg" => self.params.knight_attack_bonus_eg,
            "rook_attack_bonus_mg" => self.params.rook_attack_bonus_mg,
            "rook_attack_bonus_eg" => self.params.rook_attack_bonus_eg,
            "queen_attack_bonus_mg" => self.params.queen_attack_bonus_mg,
            "queen_attack_bonus_eg" => self.params.queen_attack_bonus_eg,
            "no_pawn_shield_penalty_mg" => self.params.no_pawn_shield_penalty_mg,
            "no_pawn_shield_penalty_eg" => self.params.no_pawn_shield_penalty_eg,
            "far_pawn_penalty_mg" => self.params.far_pawn_penalty_mg,
            "far_pawn_penalty_eg" => self.params.far_pawn_penalty_eg,
            _ => panic!("Unknown parameter: {}", field_name),
        }
    }

    // Helper function to set parameter value by field name
    fn set_param_value(&mut self, field_name: &str, value: i32) {
        if let Some(index_str) = field_name.strip_prefix("king_safety_table_") {
            if let Ok(index) = index_str.parse::<usize>() {
                if index < 100 {
                    self.params.king_safety_table[index] = value;
                    return;
                }
            }
        }
        match field_name {
            "pawn_mg" => self.params.pawn_mg = value,
            "pawn_eg" => self.params.pawn_eg = value,
            "knight_mg" => self.params.knight_mg = value,
            "knight_eg" => self.params.knight_eg = value,
            "bishop_mg" => self.params.bishop_mg = value,
            "bishop_eg" => self.params.bishop_eg = value,
            "rook_mg" => self.params.rook_mg = value,
            "rook_eg" => self.params.rook_eg = value,
            "queen_mg" => self.params.queen_mg = value,
            "queen_eg" => self.params.queen_eg = value,
            "knight_mobility_mg" => self.params.knight_mobility_mg = value,
            "knight_mobility_eg" => self.params.knight_mobility_eg = value,
            "bishop_mobility_mg" => self.params.bishop_mobility_mg = value,
            "bishop_mobility_eg" => self.params.bishop_mobility_eg = value,
            "rook_mobility_mg" => self.params.rook_mobility_mg = value,
            "rook_mobility_eg" => self.params.rook_mobility_eg = value,
            "queen_mobility_mg" => self.params.queen_mobility_mg = value,
            "queen_mobility_eg" => self.params.queen_mobility_eg = value,
            "king_mobility_mg" => self.params.king_mobility_mg = value,
            "king_mobility_eg" => self.params.king_mobility_eg = value,
            "king_center_mg" => self.params.king_center_mg = value,
            "king_center_eg" => self.params.king_center_eg = value,
            "doubled_pawn_penalty_mg" => self.params.doubled_pawn_penalty_mg = value,
            "doubled_pawn_penalty_eg" => self.params.doubled_pawn_penalty_eg = value,
            "isolated_pawn_penalty_mg" => self.params.isolated_pawn_penalty_mg = value,
            "isolated_pawn_penalty_eg" => self.params.isolated_pawn_penalty_eg = value,
            "pawn_advance_bonus_mg" => self.params.pawn_advance_bonus_mg = value,
            "pawn_advance_bonus_eg" => self.params.pawn_advance_bonus_eg = value,
            "passed_pawn_mg" => self.params.passed_pawn_mg = value,
            "passed_pawn_eg" => self.params.passed_pawn_eg = value,
            "pp_rank_2_mg" => self.params.pp_rank_2_mg = value,
            "pp_rank_2_eg" => self.params.pp_rank_2_eg = value,
            "pp_rank_3_mg" => self.params.pp_rank_3_mg = value,
            "pp_rank_3_eg" => self.params.pp_rank_3_eg = value,
            "pp_rank_4_mg" => self.params.pp_rank_4_mg = value,
            "pp_rank_4_eg" => self.params.pp_rank_4_eg = value,
            "pp_rank_5_mg" => self.params.pp_rank_5_mg = value,
            "pp_rank_5_eg" => self.params.pp_rank_5_eg = value,
            "pp_rank_6_mg" => self.params.pp_rank_6_mg = value,
            "pp_rank_6_eg" => self.params.pp_rank_6_eg = value,
            "pp_rank_7_mg" => self.params.pp_rank_7_mg = value,
            "pp_rank_7_eg" => self.params.pp_rank_7_eg = value,
            "protected_passed_pawn_mg" => self.params.protected_passed_pawn_mg = value,
            "protected_passed_pawn_eg" => self.params.protected_passed_pawn_eg = value,
            "two_attackers_bonus_mg" => self.params.two_attackers_bonus_mg = value,
            "two_attackers_bonus_eg" => self.params.two_attackers_bonus_eg = value,
            "multiple_attackers_bonus_mg" => self.params.multiple_attackers_bonus_mg = value,
            "multiple_attackers_bonus_eg" => self.params.multiple_attackers_bonus_eg = value,
            "bishop_attack_bonus_mg" => self.params.bishop_attack_bonus_mg = value,
            "bishop_attack_bonus_eg" => self.params.bishop_attack_bonus_eg = value,
            "knight_attack_bonus_mg" => self.params.knight_attack_bonus_mg = value,
            "knight_attack_bonus_eg" => self.params.knight_attack_bonus_eg = value,
            "rook_attack_bonus_mg" => self.params.rook_attack_bonus_mg = value,
            "rook_attack_bonus_eg" => self.params.rook_attack_bonus_eg = value,
            "queen_attack_bonus_mg" => self.params.queen_attack_bonus_mg = value,
            "queen_attack_bonus_eg" => self.params.queen_attack_bonus_eg = value,
            "no_pawn_shield_penalty_mg" => self.params.no_pawn_shield_penalty_mg = value,
            "no_pawn_shield_penalty_eg" => self.params.no_pawn_shield_penalty_eg = value,
            "far_pawn_penalty_mg" => self.params.far_pawn_penalty_mg = value,
            "far_pawn_penalty_eg" => self.params.far_pawn_penalty_eg = value,
            _ => panic!("Unknown parameter: {}", field_name),
        }
    }
    fn print_results(&self, done: bool) {
        println!("pub const PIECE_VALUES: [Score; 8] = [");
        println!("    Score::new(0,0), // Empty");
        println!("    Score::new(0,0), // None");
        println!("    Score::new({}, {}), // Pawn", self.params.pawn_mg, self.params.pawn_eg);
        println!("    Score::new({}, {}), // Knight", self.params.knight_mg, self.params.knight_eg);
        println!("    Score::new({}, {}), // Bishop", self.params.bishop_mg, self.params.bishop_eg);
        println!("    Score::new({}, {}), // Rook", self.params.rook_mg, self.params.rook_eg);
        println!("    Score::new({}, {}), // Queen", self.params.queen_mg, self.params.queen_eg);
        println!("    Score::new(100000, 100000) // King");
        println!("];");

        println!("\n// === Mobility Weights ===");
        println!("pub const MOBILITY_VALUES: [Score; 8] = [");
        println!("    Score::new(0,0), Score::new(0,0), Score::new(0,0),");
        println!("    Score::new({}, {}), // Knight", self.params.knight_mobility_mg, self.params.knight_mobility_eg);
        println!("    Score::new({}, {}), // Bishop", self.params.bishop_mobility_mg, self.params.bishop_mobility_eg);
        println!("    Score::new({}, {}), // Rook", self.params.rook_mobility_mg, self.params.rook_mobility_eg);
        println!("    Score::new({}, {}), // Queen", self.params.queen_mobility_mg, self.params.queen_mobility_eg);
        println!("    Score::new({}, {}), // King", self.params.king_mobility_mg, self.params.king_mobility_eg);
        println!("];");
       
            println!("\n// === Positional Factors ===");
            println!("const DOUBLED_PAWN_PENALTY: Score = Score::new({}, {});", self.params.doubled_pawn_penalty_mg, self.params.doubled_pawn_penalty_eg);
            println!("const ISOLATED_PAWN_PENALTY: Score = Score::new({}, {});", self.params.isolated_pawn_penalty_mg, self.params.isolated_pawn_penalty_eg);
            println!("const PAWN_ADVANCE_BONUS: Score = Score::new({}, {});", self.params.pawn_advance_bonus_mg, self.params.pawn_advance_bonus_eg);
            println!("const PASSED_PAWN_BASE: Score = Score::new({}, {});", self.params.passed_pawn_mg, self.params.passed_pawn_eg);
            println!("const PROTECTED_PASSED_PAWN_BONUS: Score = Score::new({}, {});", self.params.protected_passed_pawn_mg, self.params.protected_passed_pawn_eg);


            println!("\n// === Passed Pawn Rank Bonuses ===");
            println!("const PASSED_PAWN_RANK_BONUS: [Score; 8] = [");
            println!("    Score::new(0, 0),");
            println!("    Score::new({}, {}), // Rank 2", self.params.pp_rank_2_mg, self.params.pp_rank_2_eg);
            println!("    Score::new({}, {}), // Rank 3", self.params.pp_rank_3_mg, self.params.pp_rank_3_eg);
            println!("    Score::new({}, {}), // Rank 4", self.params.pp_rank_4_mg, self.params.pp_rank_4_eg);
            println!("    Score::new({}, {}), // Rank 5", self.params.pp_rank_5_mg, self.params.pp_rank_5_eg);
            println!("    Score::new({}, {}), // Rank 6", self.params.pp_rank_6_mg, self.params.pp_rank_6_eg);
            println!("    Score::new({}, {}), // Rank 7", self.params.pp_rank_7_mg, self.params.pp_rank_7_eg);
            println!("    Score::new(0, 0) // Rank 8");
            println!("];");
            println!("\n// === Other Bonuses ===");
            println!("const KING_CENTER_BONUS: Score = Score::new({}, {});", self.params.king_center_mg, self.params.king_center_eg);
            println!("const TWO_ATTACKER_BONUS: Score = Score::new({}, {});", self.params.two_attackers_bonus_mg, self.params.two_attackers_bonus_eg);
            println!("const MULTIPLE_ATTACKER_BONUS: Score = Score::new({}, {});", self.params.multiple_attackers_bonus_mg, self.params.multiple_attackers_bonus_eg);
            println!("const ATTACK_WEIGHTS: [Score; 8] = [");
            println!("    Score::new(0,0), Score::new(0,0), Score::new(0,0),");
            println!("    Score::new({}, {}), // Bishop", self.params.bishop_attack_bonus_mg, self.params.bishop_attack_bonus_eg);
            println!("    Score::new({}, {}), // Knight", self.params.knight_attack_bonus_mg, self.params.knight_attack_bonus_eg);
            println!("    Score::new({}, {}), // Rook", self.params.rook_attack_bonus_mg, self.params.rook_attack_bonus_eg);
            println!("    Score::new({}, {}), // Queen", self.params.queen_attack_bonus_mg, self.params.queen_attack_bonus_eg);
            println!("    Score::new(0,0)");
            println!("];");
            println!("const NO_PAWN_SHIELD_PENALTY: Score = Score::new({}, {});", self.params.no_pawn_shield_penalty_mg, self.params.no_pawn_shield_penalty_eg);
            println!("const FAR_PAWN_PENALTY: Score = Score::new({}, {});", self.params.far_pawn_penalty_mg, self.params.far_pawn_penalty_eg);
            
            println!("\n// === King Safety Table ===");
            println!("const KING_SAFETY_TABLE: [i32; 100] = [");
            for chunk in self.params.king_safety_table.chunks(10) {
                print!("   ");
                for (i, &val) in chunk.iter().enumerate() {
                    print!(" {}", val);
                    if i < chunk.len() - 1 {
                        print!(",");
                    }
                }
                println!(",");
            }
            println!("];");
    }
}
// Engine parameters that can be dynamically set
#[derive(Debug, Clone, Copy)]
pub struct EngineParams {
    pub piece_values: [Score; 8],
    pub mobility_values: [Score; 8],
    pub king_center_bonus: Score,
    pub doubled_pawn_penalty: Score,
    pub isolated_pawn_penalty: Score,
    pub pawn_advance_bonus: Score,
    pub passed_pawn_base: Score,
    pub passed_pawn_rank_bonus: [Score; 8],
    pub protected_passed_pawn_bonus: Score,
    pub two_attacker_bonus: Score,
    pub multiple_attacker_bonus: Score,
    pub attack_weights: [Score; 8],
    pub no_pawn_shield_penalty: Score,
    pub far_pawn_penalty: Score,
    pub king_safety_table: [i32; 100],
}

impl EngineParams {
    // Create new EngineParams with default values
    pub fn new() -> Self {
        EngineParams {
            piece_values: [
                Score::new(0,0),      // Empty
                Score::new(0,0),      // None  
                Score::new(73,109),   // Pawn
                Score::new(306,314),  // Knight
                Score::new(362,325),  // Bishop
                Score::new(457,625),  // Rook
                Score::new(1100,1010), // Queen
                Score::new(100000,100000) // King
            ],
            mobility_values: [
                Score::new(0,0),      // Empty slots
                Score::new(0,0), 
                Score::new(0,0),
                Score::new(9,11),     // Knight
                Score::new(4,10),     // Bishop
                Score::new(4,6),      // Rook
                Score::new(-1,12),    // Queen
                Score::new(-11,13)    // King
            ],
            king_center_bonus: Score::new(-18,19),
            doubled_pawn_penalty: Score::new(3,24),
            isolated_pawn_penalty: Score::new(12,16),
            pawn_advance_bonus: Score::new(2,1),
            passed_pawn_base: Score::new(-19,59),
            passed_pawn_rank_bonus: [
                Score::new(0, 0),      // rank 0
                Score::new(-2, -38),   // rank 1  
                Score::new(-8, -42),   // rank 2
                Score::new(-3, -18),   // rank 3
                Score::new(18, 15),    // rank 4
                Score::new(35, 92),    // rank 5
                Score::new(19, 155),   // rank 6
                Score::new(0, 0),      // rank 7
            ],
            protected_passed_pawn_bonus: Score::new(26,-1),
            two_attacker_bonus: Score::new(3,1),
            multiple_attacker_bonus: Score::new(3,0),
            attack_weights: [
                Score::from_single(0), // Empty slots
                Score::from_single(0), 
                Score::from_single(0),
                Score::new(3,0),       // Knight
                Score::new(5,0),       // Bishop  
                Score::new(3,0),       // Rook
                Score::new(4,3),       // Queen
                Score::from_single(0)  // King
            ],
            no_pawn_shield_penalty: Score::new(8,0),
            far_pawn_penalty: Score::new(2,3),
            king_safety_table: [
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
            ],
        }
    }
}
// Function to set engine parameters globally
pub fn set_engine_params(params: EngineParams) {
    unsafe {
        CURRENT_ENGINE_PARAMS = Some(params);
    }
}

// Sigmoid function for converting centipawn evaluation to win probability
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x/200.0).exp())
}
fn evaluate_fen(params: EngineParams, fen: &str) -> Result<i32, String> {
        // Load FEN into your board representation
        // Call your evaluate() function
        // Return centipawn evaluation
        let board = util::board_from_fen(fen);
        let eval = tunereval::evaluate(&board, &params);
        Ok(eval)
    }

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    println!("ByteChess Texel Tuner");
    println!("=====================");

    let start_time = Instant::now();

    // Load training positions (adjust this number based on your dataset size and available RAM)
    // For initial testing, start with 100k-500k positions. For final tuning, use more.
    let positions_file = "positions.txt";
    let max_positions = Some(500_000); // Start with 500k positions for good balance of speed vs accuracy

    println!("Loading up to {} positions from {}...", max_positions.unwrap(), positions_file);
    let mut tuner = TexelTuner::new(positions_file, max_positions)?;
    println!("Loaded {} positions in {:.2}s", tuner.positions.len(), start_time.elapsed().as_secs_f64());

    // Learning rate controls how aggressively parameters are updated
    // Start with 0.1-0.2 for initial tuning, can be reduced as convergence approaches
    tuner.learning_rate = 0.15;
    println!("Learning rate: {}", tuner.learning_rate);

    // Number of epochs to run
    // Each epoch processes all parameters once
    // Start with 100-200 epochs for initial testing, increase for final tuning
    let epochs = 150;
    println!("Running {} epochs...", epochs);

    println!("Starting tuning process...\n");
    let tuning_start = Instant::now();
    tuner.tune(epochs);
    let tuning_duration = tuning_start.elapsed();

    println!("\nTuning completed in {:.2}s!", tuning_duration.as_secs_f64());
    println!("Total time: {:.2}s", start_time.elapsed().as_secs_f64());
    println!("Copy the parameters above into your evaluation function.");

    Ok(())
}