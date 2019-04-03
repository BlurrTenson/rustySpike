use std::fmt;
use std::fmt::Display;
use rand::Rng;
use rand::prelude::IteratorRandom;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct node {
    function_table: Vec<bool>, // truth table
    s_t: Option<bool>,                 //tstae at current time step
    s_nt: Option<bool>,                //state at next time step
    pub tbl_size: usize,
}

#[derive(Debug)]
pub struct RBN {
    nodes : Vec<Rc<RefCell<node>>>, 
    connections: Vec<(Rc<RefCell<node>>, Rc<RefCell<node>>, Rc<RefCell<node>>)>,
}
impl RBN{
    pub fn new (k: u8, n: u16) -> RBN{//max rbn size is std::u16::MAX() 
        let mut inv_nodes = Vec::new(); 
        for _x in 0..n{
            inv_nodes.push(Rc::new(
                                RefCell::new(
                                    node::new(k)
                                            )
                                  )
                          );
        }
        let mut links = Vec::new();
        let mut rnjesus = rand::thread_rng();
        for idx in 0..inv_nodes.len(){
            links.push((inv_nodes[idx].clone(),
                            inv_nodes.iter().choose(&mut rnjesus).expect("Node list empty").clone(),  
                            inv_nodes.iter().choose(&mut rnjesus).expect("Node list empty").clone()  
                       ))

        }
        RBN {
            connections: links,
            nodes: inv_nodes,
        }
    }
    /*
    * Creates a new RBN with a predefined structure. Nodes defined by truth tables in <nd_tbls>
    * links defined by indexes in strct_tbl 
    */
    pub fn new_from_def (nd_tbls: Vec<Vec<bool>>, strct_tbl:Vec<(usize, usize)>) -> RBN{
        if nd_tbls.len() != strct_tbl.len(){
            panic!("Length mismatch number of nodes = {}, structure table lenght = {}\n",
                        nd_tbls.len(),
                        strct_tbl.len()
                  ); 
        }
        let mut inv_nodes = Vec::new(); 
        for n in nd_tbls.into_iter(){
            inv_nodes.push(Rc::new(
                                RefCell::new(
                                    node::new_with_tbl(n)
                                            )
                                  )
                          );
        }
        let mut links = Vec::new();
        for idx in 0..inv_nodes.len(){
            links.push((inv_nodes[idx].clone(),
                            inv_nodes[strct_tbl[idx].0].clone(),  
                            inv_nodes[strct_tbl[idx].1].clone(), 
                       ))

        }
        RBN {
            connections: links,
            nodes: inv_nodes,
        }
    }
    /*
     * Update nodes for next time step
     */
    pub fn step(&self){
        for nds in &self.connections{
             let l = nds.1.borrow_mut().get_current_state();
             let r = nds.2.borrow_mut().get_current_state();
             let mut sum =0; 
             if l 
                { sum += 1; }
             if r
                { sum += 2; }
             nds.0.borrow_mut().calc_next_state(sum);
        }
    }
    /*
     * Sync all nodes to the new timestep
     */
    pub fn sync(&self){
        for nds in &self.nodes{
             nds.borrow_mut().update_state(); 
        }
    }

}


impl node {
    /*
     * New node with randome boolean table of size 2^<no_in>
     * */
    pub fn new(no_in: u8) -> node {
        let mut tbl_sz;
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
            panic! {"Table size not factor of 2, invalid node"};
        }

        node {
            function_table: table, // node
            s_t: Some(true),
            s_nt: Some(false),
            tbl_size: sz,
        }
    }
    /*
     *New node with specific truth table <tbl>
     */
    pub fn new_with_tbl(tbl: Vec<bool>) -> node {
        let sz = tbl.len();
        if sz.count_ones() != 1 {
            panic! {"Table size not factor of 2, invalid node"};
        }
        node {
            function_table: tbl,
            s_t: Some(true),
            s_nt: Some(true),
            tbl_size: sz,
        }
    }
    /*
     *Get node state by <in_sum> which is combination of input nodes 
     * TODO Remove this it's only really for testing.
     */
    pub fn get_state(&mut self , in_sum : usize) -> bool{
        if self.function_table.len() < in_sum {
            panic!{"Node lookup out of range", }
        }
        self.s_t = Some(self.function_table[in_sum]);
       return self.function_table[in_sum]; 
    }
    /*
     * Calculates next state by <in_sum> which is combination of input nodes 
     */
    pub fn calc_next_state(&mut self, in_sum: usize){
        if self.function_table.len() < in_sum {
            panic!{"Node lookup out of range", }
        }
        self.s_nt = Some(self.function_table[in_sum]);
    }
    /*
     * Updates the current state with the new calculated resets the next state
     */
    pub fn update_state(&mut self){
        self.s_t = self.s_nt; 
        self.s_nt = None;
    }
    /*
     * Gets the current timestep state 
     */
    pub fn get_current_state(&self) -> bool{
        return match self.s_t {
                Some(thing) => thing,
                None        => panic!("There is no current state , I'm confused"),
        };
    }
}
impl Display for node {
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
        let n = node::new(2);
        assert_eq!(n.tbl_size, 4);
    }

    #[test]
    fn make_node_3() {
        let n = node::new(3);
        assert_eq!(n.tbl_size, 8);
    }

    #[test]
    fn make_node_7() {
        let n = node::new(7);
        assert_eq!(n.tbl_size, 128);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 8, lookup table too large\n")]
    fn make_node_8() {
        let n = node::new(8);
        assert_eq!(n.tbl_size, 8);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 255, lookup table too large\n")]
    fn make_node_max() {
        let n = node::new(std::u8::MAX);
        assert_eq!(n.tbl_size, 8);
    }

    #[test]
    fn make_from_tbl() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(true);
        tbl.push(true);
        tbl.push(true);
        let four = tbl.len();

        let n = node::new_with_tbl(tbl);
        assert_eq!(n.tbl_size, 4);
    }

    #[test]
    fn get_state() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(true);
        tbl.push(false);

        let mut n = node::new_with_tbl(tbl);
        assert_eq!(true, n.get_state(0));
        assert_eq!(false, n.get_state(1));
        assert_eq!(true, n.get_state(2));
        assert_eq!(false, n.get_state(3));
    }
    
    #[test]
    #[should_panic(expected = "Node lookup out of range")]
    fn get_state_oversize() {
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);

        let mut n = node::new_with_tbl(tbl);
        assert_eq!(false, n.get_state(3));
    }
}
