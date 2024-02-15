use chess::ChessMove;

pub struct Uci {
    pub depth_data: Vec<DepthData>,
    pub nodes_total: u64,
    pub ms_passed: u64,
}

#[derive(Clone, Debug)]
pub struct DepthData {
    pub depth: u16,
    pub best_move: Option<ChessMove>,
    pub node_count: u32,
}

impl Default for Uci {
    fn default() -> Uci {
        Uci {
            nodes_total: 0,
            ms_passed: 0,
            depth_data: vec![],
        }
    }
}

impl Uci {
    pub fn get_nodes_per_second(&self) -> f64 {
        self.nodes_total as f64 / (self.ms_passed as f64 / 1000.0)
    }

    pub fn set_ms_passed(&mut self, ms_passed: u64) {
        self.ms_passed = ms_passed;
    }

    pub fn get_depth_data(&self) -> &Vec<DepthData> {
        &self.depth_data
    }

    pub fn update_depth_data(&mut self, depth: u16, max_depth: u16, best_move: Option<ChessMove>) {
        let reversed_depth = max_depth + 1 - depth;

        if let Some(data) = self
            .depth_data
            .iter_mut()
            .find(|d| d.depth == reversed_depth)
        {
            data.best_move = best_move;
            data.node_count += 1;
        } else {
            self.depth_data.push(DepthData {
                depth: reversed_depth,
                best_move,
                node_count: 1,
            });
        }
    }
}
