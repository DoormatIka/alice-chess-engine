use chess::ChessMove;

pub struct Uci {
    pub depth_data: Vec<DepthData>,
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
            depth_data: vec![],
        }
    }
}

impl Uci{
    pub fn get_depth_data(&mut self) -> Vec<DepthData> {
        self.depth_data.clone()
    }

    
    pub fn update_depth_data(&mut self, depth: u16, max_depth: u16, best_move: Option<ChessMove>) {
        let reversed_depth = max_depth + 1 - depth;
    
        if let Some(data) = self.depth_data.iter_mut().find(|d| d.depth == reversed_depth) {
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