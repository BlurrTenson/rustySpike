use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct Node {
    function_table: Vec<bool>, // truth table
    s_t: Option<bool>,         //state at current time step
    s_nt: Option<bool>,        //state at next time step
    pub tbl_size: usize,       //truth table size
    id: u16,                   //id value , needs to be unique at rbn lvl not htis lvl
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
            id: node_id,
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
            id: node_id,
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
