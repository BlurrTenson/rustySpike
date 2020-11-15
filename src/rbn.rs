use crate::node::Node;
use crate::temp::Temperature;
use crate::util::bonding::*;
use crate::util::cycle_calc::*;
use crate::util::formatters::IsFormatable;
use particle::Component;

use bit_field::BitField;

use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::ptr;
use std::rc::Rc;

#[derive(Debug)]
pub struct RBNState {
    pattern: Vec<bool>,
}
//I hate this structure with a passion
//it only exists because I have no way to generate interaction groups withought linking the
//node index to the node pointer (short of exhaustive search which is expensive)
#[derive(Debug, Clone)]
pub struct RBNConnection {
    node: Rc<RefCell<Node>>,
    node_idx: usize,
    source_idx: Vec<usize>,
}

#[derive(Debug)]
pub struct RBN {
    /// List of nodes that are part of this RBN
    nodes: Vec<RBNConnection>,
    /// Node Network showing what inputs to each node are TODO(this only works for k=2)
    //connections: HashMap<usize, usize, usize>,
    /// Influence map showing how many nodes the key node is input to
    /// key is the id of the Node (unique within the RBN but not externally)
    //inf_map: Vec<(u16, u16)>,
    /// Cycle Length store. NOTE this may be inconsistent depending on when the last time you
    /// change something in the RBN instance is and if you recalculated it
    cycle_len: Option<u64>,
    trans_len: Option<u64>,
}
impl Component for RBN {}

impl IsBondable for RBN {
    fn generate_bonding_sites(&mut self) -> Vec<BondingSite> {
        return self.generate_interaction_groups_inf(self.nodes.len() as u16, false);
    }
    /// Returns Bonding Property for a specific &BondingSite
    /// If the BondingSite is not present on the particle returns None
    fn get_bonding_prop(&self, bs: &BondingSite) -> Option<i32> {
        return None;
    }

    /// Returns pointers to all BondingSites on the Particle
    fn get_all_bonding_sites(&self) -> Vec<&BondingSite> {
        return vec![];
    }

    /// Returns pointers to all BondingSites not currently part of a bond on the Particle
    /// If there are no free sites returns None
    fn get_free_bonding_sites(&self) -> Option<Vec<&BondingSite>> {
        return None;
    }

    /// Returns a random bonding site not currently part of a bond
    /// If there are no free sites returns None
    fn get_rand_free_bonding_site(&self) -> Option<&BondingSite> {
        return None;
    }
}
///
impl IsFormatable for RBN {
    fn fmt_header(&self) -> String {
        let mut form_string = String::new();
        form_string.push_str(&format!("  "));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!("{:>3},", node_idx));
        }
        form_string
    }
    fn fmt_state(&self) -> String {
        let mut form_string = String::new();
        form_string.push_str(&format!("  "));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!(
                "{:>3},",
                self.nodes[node_idx].node.borrow().get_current_state() as u8
            ));
        }
        form_string
    }
    fn fmt_cycle_liveliness(&self) -> String {
        let mut form_string = String::new();
        form_string.push_str(&format!("CL"));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!(
                "{:>3},",
                self.nodes[node_idx].node.borrow().get_cycle_liveliness() as i32
            ));
        }
        form_string
    }
    fn fmt_trans_liveliness(&self) -> String {
        let mut form_string = String::new();
        form_string.push_str(&format!("TL"));
        for node_idx in (0..self.nodes.len()).rev() {
            form_string.push_str(&format!(
                "{:>3},",
                self.nodes[node_idx].node.borrow().get_trans_liveliness() as i32
            ));
        }
        form_string
    }
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

        let mut inv_nodes = Vec::<RBNConnection>::new();
        for x in 0..n {
            let connection = RBNConnection {
                node: Rc::new(RefCell::new(Node::new(k, x))),
                node_idx: x as usize,
                source_idx: vec![],
            };
            // make the right number of nodes
            inv_nodes.push(connection);
        }
        let mut rng = thread_rng();
        //let mut links = Vec::new();
        let mut rnjesus = rand::thread_rng();
        for idx in 0..inv_nodes.len() {
            // for each node push the connections
            for _x in 0..k {
                let n: u16 = rng.gen_range(0, n);
                inv_nodes[idx]
                    .node
                    .borrow_mut()
                    .inputs
                    .push(inv_nodes[n as usize].node.clone());
                inv_nodes[idx].source_idx.push(n as usize);
            }
            let mut self_inf = 0;
            //itterate over node inputs and increment the input node's influece map
            //hacky because I can't call inc_influence on myself so self inflece is tallied and
            //then updated in the loop below
            for c in inv_nodes[idx].node.borrow().inputs.iter() {
                if !Rc::ptr_eq(&c, &inv_nodes[idx].node) {
                    c.borrow_mut().inc_influence();
                } else {
                    self_inf += 1;
                }
            }
            for _x in 0..self_inf {
                inv_nodes[idx].node.borrow_mut().inc_influence();
            }
        }
        // we have to inform the nodes that influence calcualtion is complete
        for nd in &inv_nodes {
            nd.node.borrow_mut().structure_set();
        }
        RBN {
            // connections: links,
            nodes: inv_nodes,
            // inf_map: inf_map_temp,
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
        let mut inv_nodes = Vec::<RBNConnection>::new();
        let mut id = 0;
        for tbl in nd_tbls.into_iter() {
            let connection = RBNConnection {
                node: Rc::new(RefCell::new(Node::new_with_tbl(tbl, id))),
                node_idx: id as usize,
                source_idx: vec![],
            };
            inv_nodes.push(connection);
            id += 1
        }
        //let mut links = Vec::new();
        for idx in 0..inv_nodes.len() {
            inv_nodes[idx]
                .node
                .borrow_mut()
                .inputs
                .push(inv_nodes[strct_tbl[idx].0].node.clone());
            inv_nodes[idx]
                .node
                .borrow_mut()
                .inputs
                .push(inv_nodes[strct_tbl[idx].1].node.clone());
            let mut self_inf = 0;
            //itterate over node inputs and increment the input node's influece map
            for c in inv_nodes[idx].node.borrow().inputs.iter() {
                if !Rc::ptr_eq(&c, &inv_nodes[idx].node) {
                    c.borrow_mut().inc_influence();
                } else {
                    self_inf += 1;
                }
            }
            for _x in 0..self_inf {
                inv_nodes[idx].node.borrow_mut().inc_influence();
            }
        }
        // we have to inform the nodes that influence calcualtion is complete
        // TODO , this needs to go away and we need a better way to define the 3 distinct states a
        // node can be in, possibly there is a constructor in rust I can call? Alternatively we can
        // do it with Types I think (change type of the node)
        for nd in &inv_nodes {
            nd.node.borrow_mut().structure_set();
        }
        RBN {
            //connections: links,
            nodes: inv_nodes,
            cycle_len: None,
            trans_len: None,
        }
    }
    fn set_state(&self, state: &RBNState) {
        let mut idx = 0;
        for n in &self.nodes {
            n.node.borrow_mut().set_current_state(state.pattern[idx]);
            idx += 1;
        }
    }
    /// Generates interaction groups based on the influence map.
    /// If is_least_inf is true then least influencial is first
    /// TODO Need to test that influence ordersing are exactly opposite (ie equivelent numbers are
    /// accessed in the same order
    fn generate_interaction_groups_inf(
        &self,
        max_group_size: u16,
        is_least_inf: bool,
    ) -> Vec<BondingSite> {
        // gen a working copy of the node list
        let mut nds_tmp = HashMap::<usize, Rc<RefCell<Node>>>::new();
        // generate influence set
        // (nodes_idx, influence)
        let mut inf_set_tmp = Vec::<(usize, u16)>::new();
        for nidx in 0..self.nodes.len() {
            inf_set_tmp.push((
                nidx,
                self.nodes[nidx].node.borrow().get_influence().unwrap(),
            ));
            nds_tmp.insert(nidx, self.nodes[nidx].node.clone());
        }
        // sort by influence
        // if is_least_inf {
        //     inf_set_tmp.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0))); // inf_set_tmp[0] is least influencial, if equal smallest id is first
        // } else {
        //     inf_set_tmp.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0))); // inf_set_tmp[0] is most influencial, if equal biggest id is first
        // }
        // println!("{:?}", inf_set_tmp);
        let mut ig_set = Vec::<BondingSite>::new();

        // while there are unassinged nodes
        while &nds_tmp.len() != &0 {
            if is_least_inf {
                inf_set_tmp.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0))); // inf_set_tmp[0] is least influencial, if equal smallest id is first
            } else {
                inf_set_tmp.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0))); // inf_set_tmp[0] is most influencial, if equal biggest id is first
            }
            //println!("{:?}", inf_set_tmp);
            let mut interaction_group = Vec::<Rc<RefCell<Node>>>::new();
            let mut current_node_idx = inf_set_tmp.pop().unwrap().0; // take a node from the ordered list
                                                                     //println!("current node {}", current_node_idx);
            let mut current_ig_size = 1;
            interaction_group.push(nds_tmp[&current_node_idx].clone());
            nds_tmp.remove(&current_node_idx);
            //while the curren interaction group is not full
            while current_ig_size < max_group_size {
                current_ig_size += 1;
                //remove the node from the working list
                nds_tmp.remove(&current_node_idx);
                // get the input with the most influence
                let next_idx = self.nodes[current_node_idx]
                    .node
                    .borrow()
                    .get_input_by_inf(is_least_inf)
                    .expect("Node has no inputs")
                    .borrow()
                    .get_id() as usize;
                //println!("\t Cur: {} Next node: {}", current_node_idx, next_idx);
                // if that input is still in the list
                if nds_tmp.contains_key(&next_idx) {
                    // select it as next node
                    current_node_idx = next_idx;
                    //add it to the interaciton group
                    interaction_group.push(nds_tmp[&current_node_idx].clone());
                    //remove the node from the list of available nodes
                    nds_tmp.remove(&next_idx);
                } else {
                    // if the most influential is no longer in the list take the other
                    let next_idx = self.nodes[current_node_idx]
                        .node
                        .borrow()
                        .get_input_by_inf(!is_least_inf)
                        .expect("Node has no inputs")
                        .borrow()
                        .get_id() as usize;
                    // println!("\t\t Visited, trying: {}", next_idx);
                    if nds_tmp.contains_key(&next_idx) {
                        // select it as next node
                        current_node_idx = next_idx;
                        //add it to the interaciton group
                        interaction_group.push(nds_tmp[&current_node_idx].clone());
                        //remove the node from the list of available nodes
                        nds_tmp.remove(&next_idx);
                    } else {
                        // println!("\t\t Both Missing , end of list");
                        // if both are missing then end the interaction_group
                        break;
                    }
                }
            }
            // interaction_group is full
            ig_set.push(BondingSite::new(interaction_group));
            // now we get a new list of nodes based on what's left in nds_tmp
            inf_set_tmp = vec![];
            for nd in &nds_tmp {
                inf_set_tmp.push((*nd.0, nd.1.borrow().get_influence().unwrap()));
            }
            // sort by influence
            // if is_least_inf {
            //     inf_set_tmp.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0))); // inf_set_tmp[0] is least influencial, if equal smallest id is first
            // } else {
            //     inf_set_tmp.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0))); // inf_set_tmp[0] is most influencial, if equal biggest id is first
            // }
            // println!("{:?}", inf_set_tmp);
        }
        // println!("{}", ig_set.len());
        for bond in &ig_set {
            println!("{}", bond)
        }

        return ig_set;
    }

    fn calculate_cycle_ln(&mut self, init_state: Temperature, verbose: bool) -> u64 {
        let mut hare: RBNState;
        let mut tortoise: RBNState;
        let mut cycle_count = 1;
        let mut power = 1;

        tortoise = RBNState::from(init_state);
        self.set_state(&tortoise);
        if verbose {
            println!("{}", self.fmt_state());
        }
        hare = self.step();
        self.sync();
        if verbose {
            println!("{}", self.fmt_state());
        }
        while tortoise != hare {
            if power == cycle_count {
                tortoise = hare;
                power = power * 2;
                cycle_count = 0;
            }
            hare = self.step();
            self.sync();
            if verbose {
                println!("{}", self.fmt_state());
            }
            cycle_count += 1;
        }
        self.cycle_len = Some(cycle_count);
        return cycle_count;
    }

    fn calculate_transient_ln(&mut self, init_state: Temperature, verbose: bool) -> u64 {
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
        for n in &self.nodes {
            n.node.borrow_mut().update_trans_liveliness();
        }
    }
    fn update_node_cycle_liveliness(&self) {
        for n in &self.nodes {
            n.node.borrow_mut().update_cycle_liveliness();
        }
    }
    fn reset_node_liveliness(&self) {
        for n in &self.nodes {
            n.node.borrow_mut().reset_liveliness();
        }
    }
    fn calculate_liveliness(&self, init_state: Temperature, verbose: bool) {
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
        if verbose {
            println!("-------------------------- \n Transient");
            println!("{}", self.fmt_state());
        }
        for _idx in 1..mu {
            // starting from 1 because set_state above is the first in transient
            self.step();
            self.sync();
            self.update_node_trans_liveliness();
            if verbose {
                println!("{}", self.fmt_state());
            }
        }
        if verbose {
            println!("-------------------------- \n Cycle");
        }
        for _idx in 0..cl {
            self.step();
            self.sync();
            self.update_node_cycle_liveliness();
            if verbose {
                println!("{}", self.fmt_state());
            }
        }
    }
}

impl IsSynchronous for RBN {
    /// Update Nodes for next time step
    fn step(&self) -> RBNState {
        let mut state = RBNState::from(0 as u64);
        let mut idx = 0;
        for nds in &self.nodes {
            // get current state of inputs:
            // When a node refers to itself we have no issue since the references are immutable (so
            // we can borrow more then once) if either the node borrow or the input borrow is
            // borrow_mut() the thing will panic at runtime
            // Test case panics on inputs[1] if there is a borrow_mut()
            let l = nds.node.borrow().inputs[0].borrow().get_current_state();
            let r = nds.node.borrow().inputs[1].borrow().get_current_state();
            let mut sum = 0;
            if l {
                sum += 1;
            }
            if r {
                sum += 2;
            }
            state.pattern[idx] = nds.node.borrow_mut().calc_next_state(sum);
            idx += 1;
        }
        state
    }

    /// Sync all Nodes to the new timestep
    fn sync(&self) {
        for nds in &self.nodes {
            nds.node.borrow_mut().update_state();
        }
    }
}

impl IsSubSymbolic for RBN {
    fn calculate_particle(&mut self, init_state: Temperature, verbose: bool) -> Stability {
        let cl = self.calculate_cycle_ln(init_state, verbose);
        let tran = self.calculate_transient_ln(init_state, verbose);
        self.calculate_liveliness(init_state, verbose);
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
/// Default print of RBN shows structure and node truth tables
impl fmt::Display for RBN {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut form_string = String::new();
        form_string.push_str("ID\tFunction\tStruct\tInfluence\n");
        for n in &self.nodes {
            form_string.push_str(&format!("{},\t", n.node.borrow().get_id()));
            for val in n.node.borrow().get_function_table() {
                form_string.push_str(&format!("{},", *val as u8));
            }
            form_string.push_str(&format!(
                "\t{},{},\t{}\n",
                n.node.borrow().inputs[0].borrow().get_id(),
                n.node.borrow().inputs[1].borrow().get_id(),
                n.node.borrow().get_influence().unwrap_or(0)
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
    fn test_rbn_interaction_list() {
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
        newrbn.calculate_particle(0b000000000101, true);
        let bonding_sites = newrbn.generate_interaction_groups_inf(23, false);
        let mut generated_sites = String::new();
        for bonding_site in bonding_sites {
            generated_sites.push_str(&format!("{}", bonding_site));
        }
        let expected_sites = "[4, 3, 1, 5, 6, 8, 2, 0][11, 9][10][7]";
        assert_eq!(generated_sites, expected_sites);

        let bonding_sites = newrbn.generate_interaction_groups_inf(23, true);
        let mut generated_sites = String::new();
        for bonding_site in bonding_sites {
            generated_sites.push_str(&format!("{}", bonding_site));
        }
        let expected_sites = "[7, 11, 9, 2, 10, 0, 5, 6, 8, 3, 1][4]";
        assert_eq!(generated_sites, expected_sites);

        let bonding_sites = newrbn.generate_interaction_groups_inf(2, false);
        let mut generated_sites = String::new();
        for bonding_site in bonding_sites {
            generated_sites.push_str(&format!("{}", bonding_site));
        }
        let expected_sites = "[4, 3][5, 6][11, 8][2, 0][10][9][1][7]";
        assert_eq!(generated_sites, expected_sites);

        let bonding_sites = newrbn.generate_interaction_groups_inf(1, false);
        let mut generated_sites = String::new();
        for bonding_site in bonding_sites {
            generated_sites.push_str(&format!("{}", bonding_site));
        }
        let expected_sites = "[4][5][3][11][8][2][0][10][9][6][1][7]";
        assert_eq!(generated_sites, expected_sites);
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
        newrbn.calculate_particle(0b000000000101, true);
        //This generates a cycle length of 4
        assert_eq!(Some(4), newrbn.cycle_len);
        assert_eq!(Some(5), newrbn.trans_len);
        let expected_struct = "ID\tFunction\tStruct\tInfluence\n0,\t1,0,0,1,\t4,5,\t2\n1,\t1,1,0,0,\t3,5,\t1\n2,\t1,0,0,0,\t0,10,\t2\n3,\t1,0,1,1,\t1,4,\t3\n4,\t1,1,0,0,\t3,4,\t6\n5,\t0,0,0,0,\t4,6,\t3\n6,\t1,1,1,0,\t4,8,\t1\n7,\t0,1,1,0,\t11,4,\t0\n8,\t1,0,0,0,\t2,3,\t2\n9,\t0,0,1,0,\t2,11,\t1\n10,\t0,1,1,0,\t0,5,\t1\n11,\t1,0,1,1,\t9,8,\t2\n";
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
