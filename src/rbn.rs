use rand::prelude::*;
use std::fmt;
use std::fmt::{Display};

pub struct node{
   function_table   : Vec<bool>, // truth table
   s_t              : bool,       //tstae at current time step 
   s_nt             : bool,      //state at next time step 
   pub tbl_size     : usize,
}

pub struct RBN<'n>{
    nodes   : Vec<(node , &'n node, &'n node)>,
    
}

impl node{
    /*
     * New node with randome boolean table of size 2^<no_in>  
     * */
    pub fn new(no_in: u8) -> node{

        let mut tbl_sz; 
        match 1u8.checked_shl(no_in.into()){ //table has 2^no_in entries 
                    Some(shift) => {tbl_sz = shift},
                    None        => {tbl_sz = 0},
        };

        if tbl_sz == 0{
            panic!("Node with too many inputs, k = {}, lookup table too large\n", no_in);
        }
        let mut table = Vec::new();
        for _x in 0.. tbl_sz {
            table.push(rand::random::<bool>()); // generate bool function
        }
        
        let sz = table.len();
        if sz.count_ones() != 1{
            panic!{"Table size not factor of 2, invalid node"};
        }

        node {function_table: table, // node
                s_t : true,
                s_nt: false,
                tbl_size: sz ,
            }
    }
    /*
     *New node with specific truth table <tbl>
     */
    pub fn new_with_tbl(tbl: Vec<bool>)-> node{
        let sz = tbl.len();
        if sz.count_ones() != 1{
            panic!{"Table size not factor of 2, invalid node"};
        }
        node{function_table: tbl,
                s_t : true,
                s_nt: true,
                tbl_size: sz,
            }
    }
}
impl Display for node{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        let mut fmt_string  = String::new();
        let mut n =0;
        for state in &self.function_table {
            fmt_string.push_str(&n.to_string());
            fmt_string.push_str(": \t");
            fmt_string.push_str(&state.to_string());
            fmt_string.push_str("\n");
            n += 1;
        }
        write!(f,"{}",  fmt_string)

    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn make_node_2(){
        let n = node::new(2);         
        assert_eq!(n.tbl_size, 4);
    }

    #[test]
    fn make_node_3(){
        let n = node::new(3);         
        assert_eq!(n.tbl_size, 8);
    }

    #[test]
    fn make_node_7(){
        let n = node::new(7);         
        assert_eq!(n.tbl_size, 128);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 8, lookup table too large\n")]
    fn make_node_8(){
        let n = node::new(8);         
        assert_eq!(n.tbl_size, 8);
    }

    #[test]
    #[should_panic(expected = "Node with too many inputs, k = 255, lookup table too large\n")]
    fn make_node_max(){
        let n = node::new(std::u8::MAX );         
        assert_eq!(n.tbl_size, 8);
    }

    #[test]
    fn make_from_tbl(){
        let mut tbl = Vec::new();
        tbl.push(true);  
        tbl.push(true);  
        tbl.push(true);  
        tbl.push(true);  
        let four = tbl.len();
        
        let n = node::new_with_tbl(tbl);
        assert_eq!(n.tbl_size, 4);
    }
}