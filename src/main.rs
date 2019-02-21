extern crate rand; 

mod rbn;

fn main() {
    //let n = rbn::node::new(std::u8::MAX/3 );         
    let n = rbn::node::new(2);         
    //assert_eq!(n.tbl_size, 8);
    let mut tbl = Vec::new();
    tbl.push(true);  
    tbl.push(true);  
    tbl.push(true);  
    let three = tbl.len();
    tbl.push(true);  
    let four = tbl.len();
    tbl.push(false);  
    let five = tbl.len();
    tbl.push(false);  
    let six = tbl.len();
    tbl.push(false);  
    tbl.push(false);  
    let eight = tbl.len();

    tbl.push(false);  
    tbl.push(false);  
    tbl.push(false);  
    tbl.push(false);  
    tbl.push(false);  
    tbl.push(false);  
    tbl.push(false);  
    tbl.push(false);  
    let sixteen = tbl.len();
    
    tbl.push(false);  
    tbl.push(false);  
    tbl.push(false);  

    let not16 = tbl.len();
    
    let t = rbn::node::new_with_tbl(tbl);
    println!{"three = {}", three.count_ones()};
    println!{"four = {}", four.count_ones()};
    println!{"five = {}", five.count_ones()};
    println!{"six = {}", six.count_ones()};
    println!{"eight = {}", eight.count_ones()};
    println!{"sixteen = {}", sixteen.count_ones()};
    println!{"not16 = {}", not16.count_ones()};
    
}

