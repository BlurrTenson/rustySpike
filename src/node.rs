use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug)]
pub struct Node {
    function_table: Vec<bool>,     // truth table
    s_t: Option<bool>,             //state at current time step
    s_nt: Option<bool>,            //state at next time step
    pub tbl_size: usize,           //truth table size
    trans_liveliness: Option<i32>, // liveliness of the node
    cycle_liveliness: Option<i32>, // liveliness of the node
    id: u16,                       //id value , needs to be unique at rbn lvl not htis lvl
    pub inputs: Vec<Rc<RefCell<Node>>>,
    influence: Option<u16>,
}

impl Node {
    /// New Node with randome boolean table of size 2^<no_in>
    pub fn new(no_in: u8, node_id: u16) -> Node {
        let tbl_sz;
        match 1u8.checked_shl(no_in.into()) {
            //table has 2^no_in entries
            Some(shift) => tbl_sz = shift,
            None => tbl_sz = 0,
        };

        if tbl_sz == 0 {
            panic!(
                "Node with too many inputs, k = {}, lookup table too large\n",
                no_in
            );
        }
        let mut table = Vec::new();
        for _x in 0..tbl_sz {
            table.push(rand::random::<bool>()); // generate bool function
        }

        let sz = table.len();
        if sz.count_ones() != 1 {
            panic! {"Table size not factor of 2, invalid Node"};
        }

        Node {
            function_table: table, // Node
            s_t: Some(true),
            s_nt: Some(false),
            tbl_size: sz,
            trans_liveliness: None,
            cycle_liveliness: None,
            id: node_id,
            inputs: vec![],
            influence: None,
        }
    }

    /// New Node with specific truth table <tbl>
    pub fn new_with_tbl(tbl: Vec<bool>, node_id: u16) -> Node {
        let sz = tbl.len();
        if sz.count_ones() != 1 {
            panic! {"Table size not factor of 2, invalid Node"};
        }
        Node {
            function_table: tbl,
            s_t: Some(true),
            s_nt: Some(true),
            tbl_size: sz,
            trans_liveliness: None,
            cycle_liveliness: None,
            id: node_id,
            inputs: vec![],
            influence: None,
        }
    }

    /// Get Node state by <in_sum> which is combination of input Nodes
    /// TODO Remove this it's only really for testing.
    pub fn get_state(&mut self, in_sum: usize) -> bool {
        if self.function_table.len() < in_sum {
            panic! {"Node lookup out of range", }
        }
        self.s_t = Some(self.function_table[in_sum]);
        return self.function_table[in_sum];
    }

    /// Calculates next state by <in_sum> which is combination of input Nodes
    pub fn calc_next_state(&mut self, in_sum: usize) -> bool {
        if self.function_table.len() < in_sum {
            panic! {"Node lookup out of range", }
        }
        self.s_nt = Some(self.function_table[in_sum]);
        return self.function_table[in_sum];
    }

    /// Updates the current state with the new calculated resets the next state
    pub fn update_state(&mut self) {
        self.s_t = self.s_nt;
        self.s_nt = None;
    }

    /// Gets the current timestep state
    pub fn get_current_state(&self) -> bool {
        return match self.s_t {
            Some(thing) => thing,
            None => panic!("There is no current state , I'm confused"),
        };
    }
    pub fn set_current_state(&mut self, state: bool) {
        self.s_t = Some(state);
        self.s_nt = None;
    }
    pub fn get_function_table(&self) -> &Vec<bool> {
        return &self.function_table;
    }

    pub fn get_id(&self) -> u16 {
        return self.id;
    }
    pub fn reset_liveliness(&mut self) {
        self.trans_liveliness = None;
        self.cycle_liveliness = None;
    }
    /// Updates the liveliness counter based on current state
    pub fn update_trans_liveliness(&mut self) {
        match self.s_t {
            Some(state) => {
                if state {
                    self.trans_liveliness = Some(self.trans_liveliness.unwrap_or(0) + 1);
                } else {
                    self.trans_liveliness = Some(self.trans_liveliness.unwrap_or(0) - 1);
                }
            }
            None => panic!("No current State set"),
        }
    }
    pub fn reset_cycle_liveliness(&mut self) {
        self.cycle_liveliness = None;
    }
    /// Updates the liveliness counter based on current state
    pub fn update_cycle_liveliness(&mut self) {
        if self.s_t.unwrap() {
            self.cycle_liveliness = Some(self.cycle_liveliness.unwrap_or(0) + 1);
        } else {
            self.cycle_liveliness = Some(self.cycle_liveliness.unwrap_or(0) - 1);
        }
    }
    pub fn get_cycle_liveliness(&self) -> i32 {
        return self.cycle_liveliness.unwrap();
    }
    pub fn get_trans_liveliness(&self) -> i32 {
        return self.trans_liveliness.unwrap();
    }
    pub fn get_influence(&self) -> Option<u16> {
        return self.influence;
    }
    //TODO this function should actually sort inputs by influence and ID to give absolute ordering
    //(which is the same as RBN::generate_interaction_groups_inf()) and then it can return
    //first/last or even the hole list. Need this if you want to do k>2
    pub fn get_input_by_inf(&self, least: bool) -> Option<Rc<RefCell<Node>>> {
        if self.inputs.len() < 2 {
            return None;
        }

        if self.inputs[0].borrow().get_influence() <= self.inputs[1].borrow().get_influence() {
            if least {
                return Some(self.inputs[0].clone());
            } else {
                return Some(self.inputs[1].clone());
            }
        } else {
            if least {
                return Some(self.inputs[1].clone());
            } else {
                return Some(self.inputs[0].clone());
            }
        }
    }
    //once the structure is set the influence is set, if it is still none then we set it to 0
    pub fn structure_set(&mut self) {
        match self.influence {
            Some(x) => self.influence = Some(x),
            None => self.influence = Some(0),
        }
    }
    pub fn inc_influence(&mut self) {
        println!("incrementing node influence");
        match self.influence {
            Some(x) => self.influence = Some(x + 1),
            None => self.influence = Some(1),
        }
    }
}
impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fmt_string = String::new();
        let mut n = 0;
        for state in &self.function_table {
            fmt_string.push_str(&n.to_string());
            fmt_string.push_str(": \t");
            fmt_string.push_str(&state.to_string());
            fmt_string.push_str("\n");
            n += 1;
        }
        write!(f, "{}", fmt_string)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn make_node_2() {
        let n = Node::new(2, 1);
        assert_eq!(n.tbl_size, 4);
        assert_eq!(n.get_id(), 1);
    }

    #[test]
    fn make_node_3() {
        let n = Node::new(3, 1);
        assert_eq!(n.tbl_size, 8);
        assert_eq!(n.get_id(), 1);
    }

    #[test]
    fn make_node_7() {
        let n = Node::new(7, 1);
        assert_eq!(n.tbl_size, 128);
        assert_eq!(n.get_id(), 1);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 8, lookup table too large\n")]
    fn make_node_8() {
        let n = Node::new(8, 1);
        assert_eq!(n.tbl_size, 8);
        assert_eq!(n.get_id(), 1);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 255, lookup table too large\n")]
    fn make_node_max() {
        let n = Node::new(std::u8::MAX, 1);
        assert_eq!(n.tbl_size, 8);
        assert_eq!(n.get_id(), 1);
    }
}
