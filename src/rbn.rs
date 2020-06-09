use bit_field::BitField;
use rand::prelude::IteratorRandom;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

use crate::temp::Temperature;
use crate::util::cycle_calc::*;
use particle::{IsSubSymbolic, Stability};

#[derive(Debug)]
pub struct RBNState {
    pattern: Vec<bool>,
}

#[derive(Debug)]
pub struct Node {
    function_table: Vec<bool>, // truth table
    s_t: Option<bool>,         //state at current time step
    s_nt: Option<bool>,        //state at next time step
    pub tbl_size: usize,       //truth table size
    id: u16,                   //id value , needs to be unique at rbn lvl not htis lvl
}

#[derive(Debug)]
pub struct RBN {
    nodes: Vec<Rc<RefCell<Node>>>,
    connections: Vec<(Rc<RefCell<Node>>, Rc<RefCell<Node>>, Rc<RefCell<Node>>)>,
    cycle_len: Option<u64>,
    trans_len: Option<u64>,
}

impl RBN {
    /// Create a new RBN with random structure
    /// k : number of links per Node
    /// n : number of Nodes
    pub fn new(k: u8, n: u16) -> RBN {
        //max rbn size is std::u16::MAX()

        if k != 2 {
            panic!(
                "k has to equal 2, we don't support this k = {} heresy here",
                k
            )
        }

        let mut inv_nodes = Vec::new();
        for x in 0..n {
            inv_nodes.push(Rc::new(RefCell::new(Node::new(k, x))));
        }
        let mut links = Vec::new();
        let mut rnjesus = rand::thread_rng();
        for idx in 0..inv_nodes.len() {
            links.push((
                inv_nodes[idx].clone(),
                inv_nodes
                    .iter()
                    .choose(&mut rnjesus)
                    .expect("Node list empty")
                    .clone(),
                inv_nodes
                    .iter()
                    .choose(&mut rnjesus)
                    .expect("Node list empty")
                    .clone(),
            ))
        }
        RBN {
            connections: links,
            nodes: inv_nodes,
            cycle_len: None,
            trans_len: None,
        }
    }

    ///Creates a new RBN with a predefined structure. Nodes defined by truth tables in <nd_tbls>
    ///links defined by indexes in strct_tbl
    pub fn new_from_def(nd_tbls: Vec<Vec<bool>>, strct_tbl: Vec<(usize, usize)>) -> RBN {
        if nd_tbls.len() != strct_tbl.len() {
            panic!(
                "Length mismatch number of Nodes = {}, structure table lenght = {}\n",
                nd_tbls.len(),
                strct_tbl.len()
            );
        }
        let mut inv_nodes = Vec::new();
        let mut id = 0;
        for n in nd_tbls.into_iter() {
            inv_nodes.push(Rc::new(RefCell::new(Node::new_with_tbl(n, id))));
            id += 1
        }
        let mut links = Vec::new();
        for idx in 0..inv_nodes.len() {
            links.push((
                inv_nodes[idx].clone(),
                inv_nodes[strct_tbl[idx].0].clone(),
                inv_nodes[strct_tbl[idx].1].clone(),
            ))
        }
        RBN {
            connections: links,
            nodes: inv_nodes,
            cycle_len: None,
            trans_len: None,
        }
    }
    fn set_state(&self, state: &RBNState) {
        let mut idx = 0;
        for node in &self.nodes {
            node.borrow_mut().set_current_state(state.pattern[idx]);
            idx += 1;
        }
    }
    pub fn fmt_header(&self) -> String {
        let mut form_string = String::new();
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!("{:>3},", node_idx));
        }
        form_string
    }
    pub fn fmt_state(&self) -> String {
        let mut form_string = String::new();
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!(
                "{:>3},",
                self.nodes[node_idx].borrow().get_current_state() as u8
            ));
        }
        form_string
    }
}

impl From<u16> for RBNState {
    fn from(num: u16) -> Self {
        let mut pat = vec![false; 16];
        for idx in 0..16 {
            pat[idx] = num.get_bit(idx);
        }
        RBNState { pattern: pat }
    }
}
impl From<u32> for RBNState {
    fn from(num: u32) -> Self {
        let mut pat = vec![false; 32];
        for idx in 0..32 {
            pat[idx] = num.get_bit(idx);
        }
        RBNState { pattern: pat }
    }
}
impl From<u64> for RBNState {
    fn from(num: u64) -> Self {
        let mut pat = vec![false; 64];
        for idx in 0..64 {
            pat[idx] = num.get_bit(idx);
        }
        RBNState { pattern: pat }
    }
}
impl PartialEq for RBNState {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl fmt::Display for RBN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut form_string = String::new();
        form_string.push_str("ID\tFunction\tStruct\n");
        for node_idx in 0..self.nodes.len() {
            form_string.push_str(&format!("{},\t", &self.nodes[node_idx].borrow().id));
            for val in &self.nodes[node_idx].borrow().function_table {
                form_string.push_str(&format!("{},", *val as u8));
            }
            form_string.push_str(&format!(
                "\t{},{}\n",
                &self.connections[node_idx].1.borrow().id,
                &self.connections[node_idx].2.borrow().id
            ));
        }
        write!(f, "{}", form_string)
    }
}

impl IsSynchronous for RBN {
    /// Update Nodes for next time step
    fn step(&self) -> RBNState {
        let mut state = RBNState::from(0 as u64);
        let mut idx = 0;
        for nds in &self.connections {
            let l = nds.1.borrow_mut().get_current_state();
            let r = nds.2.borrow_mut().get_current_state();
            let mut sum = 0;
            if l {
                sum += 1;
            }
            if r {
                sum += 2;
            }
            state.pattern[idx] = nds.0.borrow_mut().calc_next_state(sum);
            idx += 1;
        }
        state
    }

    /// Sync all Nodes to the new timestep
    fn sync(&self) {
        for nds in &self.nodes {
            nds.borrow_mut().update_state();
        }
    }
}
impl IsCyclic for RBN {
    fn calculate_cycle_ln(&mut self, init_state: Temperature) -> u64 {
        let mut hare: RBNState;
        let mut tortoise: RBNState;
        let mut cycle_count = 1;
        let mut power = 1;

        tortoise = RBNState::from(init_state);
        self.set_state(&tortoise);
        println!("{}", self.fmt_state());
        hare = self.step();
        self.sync();
        println!("{}", self.fmt_state());
        while tortoise != hare {
            if power == cycle_count {
                tortoise = hare;
                power = power * 2;
                cycle_count = 0;
            }
            hare = self.step();
            self.sync();
            println!("{}", self.fmt_state());
            cycle_count += 1;
        }
        self.cycle_len = Some(cycle_count);
        return cycle_count;
    }

    fn calculate_transient_ln(&mut self, init_state: Temperature) -> u64 {
        let mut cl;
        if self.cycle_len.is_some() {
            cl = self.cycle_len.unwrap();
        } else {
            panic!("Calculating transient with a None cycle lenght");
        }
        let mut hare: RBNState;
        let mut tortoise: RBNState;
        hare = RBNState::from(init_state);
        tortoise = RBNState::from(init_state);
        self.set_state(&hare);
        //Put hare 1 cl away from tortose
        for _idx in 0..cl {
            hare = self.step();
            self.sync();
        }
        let mut mu = 0;
        while tortoise != hare {
            self.set_state(&tortoise);
            tortoise = self.step();
            self.set_state(&hare);
            hare = self.step();
            mu += 1;
        }
        self.trans_len = Some(mu);
        return mu;
    }

    fn calculate_liveliness(&self, _init_state: Temperature) {}
}

impl IsSubSymbolic for RBN {
    fn calculate_particle(&mut self, init_state: Temperature) -> Stability {
        let cl = self.calculate_cycle_ln(init_state);
        let tran = self.calculate_transient_ln(init_state);
        return Stability::Unstable {
            cycle: cl,
            transient: tran,
        };
    }
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
        assert_eq!(n.id, 1);
    }

    #[test]
    fn make_node_3() {
        let n = Node::new(3, 1);
        assert_eq!(n.tbl_size, 8);
        assert_eq!(n.id, 1);
    }

    #[test]
    fn make_node_7() {
        let n = Node::new(7, 1);
        assert_eq!(n.tbl_size, 128);
        assert_eq!(n.id, 1);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 8, lookup table too large\n")]
    fn make_node_8() {
        let n = Node::new(8, 1);
        assert_eq!(n.tbl_size, 8);
        assert_eq!(n.id, 1);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 255, lookup table too large\n")]
    fn make_node_max() {
        let n = Node::new(std::u8::MAX, 1);
        assert_eq!(n.tbl_size, 8);
        assert_eq!(n.id, 1);
    }

    #[test]
    fn make_from_tbl() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(true);
        tbl.push(true);
        tbl.push(true);

        let n = Node::new_with_tbl(tbl, 1);
        assert_eq!(n.tbl_size, 4);
        assert_eq!(n.id, 1);
    }
    #[test]
    fn get_state() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(true);
        tbl.push(false);

        let mut n = Node::new_with_tbl(tbl, 1);
        assert_eq!(n.id, 1);
        assert_eq!(true, n.get_state(0));
        assert_eq!(false, n.get_state(1));
        assert_eq!(true, n.get_state(2));
        assert_eq!(false, n.get_state(3));
    }
    #[test]
    fn test_rbn_calc() {
        let mut nds = Vec::new();
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(false);
        tbl.push(true);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(true);
        tbl.push(false);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(false);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(true);
        tbl.push(true);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(true);
        tbl.push(false);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(false);
        tbl.push(false);
        tbl.push(false);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(true);
        tbl.push(true);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(false);
        tbl.push(true);
        tbl.push(true);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(false);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(false);
        tbl.push(false);
        tbl.push(true);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(false);
        tbl.push(true);
        tbl.push(true);
        tbl.push(false);
        nds.push(tbl);
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(true);
        tbl.push(true);
        nds.push(tbl);

        let mut rbn_struct = Vec::<(usize, usize)>::new();
        rbn_struct.push((4, 5));
        rbn_struct.push((3, 5));
        rbn_struct.push((0, 10));
        rbn_struct.push((1, 4));
        rbn_struct.push((3, 4));
        rbn_struct.push((4, 6));
        rbn_struct.push((4, 8));
        rbn_struct.push((11, 4));
        rbn_struct.push((2, 3));
        rbn_struct.push((2, 11));
        rbn_struct.push((0, 5));
        rbn_struct.push((9, 8));
        let mut newrbn = RBN::new_from_def(nds, rbn_struct);

        println!("{}", newrbn);
        println!("{}", newrbn.fmt_header());
        let _cl1 = newrbn.calculate_cycle_ln(0b000000000101);
        let _mu1 = newrbn.calculate_transient_ln(0b000000000101);
        //This generates a cycle length of 4
        assert_eq!(Some(4), newrbn.cycle_len);
        assert_eq!(Some(5), newrbn.trans_len);
        let expected_struct = "ID\tFunction\tStruct\n0,\t1,0,0,1,\t4,5\n1,\t1,1,0,0,\t3,5\n2,\t1,0,0,0,\t0,10\n3,\t1,0,1,1,\t1,4\n4,\t1,1,0,0,\t3,4\n5,\t0,0,0,0,\t4,6\n6,\t1,1,1,0,\t4,8\n7,\t0,1,1,0,\t11,4\n8,\t1,0,0,0,\t2,3\n9,\t0,0,1,0,\t2,11\n10,\t0,1,1,0,\t0,5\n11,\t1,0,1,1,\t9,8\n";
        assert_eq!(format!("{}", newrbn), expected_struct);
        let mut state_str = String::new();
        state_str.push_str(&newrbn.fmt_header());
        newrbn.step();
        newrbn.sync();
        state_str.push_str(&newrbn.fmt_state());
        newrbn.step();
        newrbn.sync();
        state_str.push_str(&newrbn.fmt_state());
        newrbn.step();
        newrbn.sync();
        state_str.push_str(&newrbn.fmt_state());
        newrbn.step();
        newrbn.sync();
        state_str.push_str(&newrbn.fmt_state());
        let expected_states = " 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,  0,  1,  1,  1,  0,  1,  0,  0,  1,  0,  1,  0,  1,  0,  0,  0,  0,  1,  0,  1,  0,  0,  1,  1,  1,  1,  1,  1,  0,  1,  0,  0,  1,  0,  1,  0,  1,  0,  1,  0,  1,  1,  0,  1,  0,  0,  1,  1,";
        assert_eq!(state_str, expected_states);
    }

    #[test]
    #[should_panic(expected = "Node lookup out of range")]
    fn get_state_oversize() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);

        let mut n = Node::new_with_tbl(tbl, 1);
        assert_eq!(false, n.get_state(3));
    }
}
