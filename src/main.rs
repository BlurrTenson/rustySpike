extern crate rand; 

mod rbn;

fn main() {
        
        let mut tbl = Vec::new();
        tbl.push(true);
        tbl.push(false);
        tbl.push(true);
        tbl.push(false);
        let mut nds = Vec::new();
        nds.push(tbl);
        /*let mut n = rbn::node::new_with_tbl(tbl);
        assert_eq!(true, n.get_state(0));
        assert_eq!(false, n.get_state(1));
        assert_eq!(true, n.get_state(2));
        assert_eq!(false, n.get_state(3));
        */ 
        let newrbn = rbn::RBN::new(2, 2);
        println!("{:?}", newrbn);
        newrbn.step();
        newrbn.sync();
        println!("{:?}", newrbn);
}

