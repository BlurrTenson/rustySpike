use bit_field::BitField;
use rand::prelude::IteratorRandom;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::node::Node;
use crate::temp::Temperature;
use crate::util::cycle_calc::*;
use particle::{IsSubSymbolic, Stability};

#[derive(Debug)]
pub struct RBNState {
    pattern: Vec<bool>,
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
        form_string.push_str(&format!("  "));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!("{:>3},", node_idx));
        }
        form_string
    }
    pub fn fmt_state(&self) -> String {
        let mut form_string = String::new();
        form_string.push_str(&format!("  "));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!(
                "{:>3},",
                self.nodes[node_idx].borrow().get_current_state() as u8
            ));
        }
        form_string
    }
    pub fn fmt_cycle_liveliness(&self) -> String {
        let mut form_string = String::new();
        form_string.push_str(&format!("CL"));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!(
                "{:>3},",
                self.nodes[node_idx].borrow().get_cycle_liveliness() as i32
            ));
        }
        form_string
    }
    pub fn fmt_trans_liveliness(&self) -> String {
        let mut form_string = String::new();
        form_string.push_str(&format!("TL"));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!(
                "{:>3},",
                self.nodes[node_idx].borrow().get_trans_liveliness() as i32
            ));
        }
        form_string
    }

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
    fn update_node_trans_liveliness(&self) {
        for node in &self.nodes {
            node.borrow_mut().update_trans_liveliness();
        }
    }
    fn update_node_cycle_liveliness(&self) {
        for node in &self.nodes {
            node.borrow_mut().update_cycle_liveliness();
        }
    }
    fn reset_node_liveliness(&self) {
        for node in &self.nodes {
            node.borrow_mut().reset_liveliness();
        }
    }
    fn calculate_liveliness(&self, init_state: Temperature) {
        let mut cl;
        let mut mu;
        if self.cycle_len.is_some() && self.trans_len.is_some() {
            cl = self.cycle_len.unwrap();
            mu = self.trans_len.unwrap();
        } else {
            panic!("Calculating Liveliness with a None cycle or transient Lenght");
        }
        self.reset_node_liveliness();

        self.set_state(&RBNState::from(init_state));
        self.update_node_trans_liveliness();
        println!("-------------------------- \n Transient");
        println!("{}", self.fmt_state());
        for _idx in 1..mu {
            // starting from 1 because set_state above is the first in transient
            self.step();
            self.sync();
            self.update_node_trans_liveliness();
            println!("{}", self.fmt_state());
        }
        println!("-------------------------- \n Cycle");
        for _idx in 0..cl {
            self.step();
            self.sync();
            self.update_node_cycle_liveliness();
            println!("{}", self.fmt_state());
        }
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

impl IsSubSymbolic for RBN {
    fn calculate_particle(&mut self, init_state: Temperature) -> Stability {
        let cl = self.calculate_cycle_ln(init_state);
        let tran = self.calculate_transient_ln(init_state);
        self.calculate_liveliness(init_state);
        return Stability::Unstable {
            cycle: cl,
            transient: tran,
        };
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
            form_string.push_str(&format!("{},\t", &self.nodes[node_idx].borrow().get_id()));
            for val in self.nodes[node_idx].borrow().get_function_table() {
                form_string.push_str(&format!("{},", *val as u8));
            }
            form_string.push_str(&format!(
                "\t{},{}\n",
                &self.connections[node_idx].1.borrow().get_id(),
                &self.connections[node_idx].2.borrow().get_id()
            ));
        }
        write!(f, "{}", form_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn make_from_tbl() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(true);
        tbl.push(true);
        tbl.push(true);

        let n = Node::new_with_tbl(tbl, 1);
        assert_eq!(n.tbl_size, 4);
        assert_eq!(n.get_id(), 1);
    }
    #[test]
    fn get_state() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(true);
        tbl.push(false);

        let mut n = Node::new_with_tbl(tbl, 1);
        assert_eq!(n.get_id(), 1);
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
        newrbn.calculate_particle(0b000000000101);
        //This generates a cycle length of 4
        assert_eq!(Some(4), newrbn.cycle_len);
        assert_eq!(Some(5), newrbn.trans_len);
        let expected_struct = "ID\tFunction\tStruct\n0,\t1,0,0,1,\t4,5\n1,\t1,1,0,0,\t3,5\n2,\t1,0,0,0,\t0,10\n3,\t1,0,1,1,\t1,4\n4,\t1,1,0,0,\t3,4\n5,\t0,0,0,0,\t4,6\n6,\t1,1,1,0,\t4,8\n7,\t0,1,1,0,\t11,4\n8,\t1,0,0,0,\t2,3\n9,\t0,0,1,0,\t2,11\n10,\t0,1,1,0,\t0,5\n11,\t1,0,1,1,\t9,8\n";
        assert_eq!(format!("{}", newrbn), expected_struct);

        let expected_cl = "CL  2,  0,  2,  0, -2,  4, -4,  0,  0, -4,  4,  0,";
        let mut cl_str = String::new();
        cl_str.push_str(&newrbn.fmt_cycle_liveliness());
        assert_eq!(cl_str, expected_cl);
        let expected_tl = "TL -1,  1, -1, -3, -1,  3, -5, -1,  1, -3,  3,  1,";
        let mut tl_str = String::new();
        tl_str.push_str(&newrbn.fmt_trans_liveliness());
        assert_eq!(tl_str, expected_tl);
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
