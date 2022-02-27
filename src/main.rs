

#[macro_use(shards)]
#[macro_use(convert_2D_slices)]
extern crate reed_solomon_erasure;

use reed_solomon_erasure::galois_8::ReedSolomon;
// or use the following for Galois 2^16 backend
//use reed_solomon_erasure::galois_16::ReedSolomon;

use std::io::{self, Read, Write, ErrorKind, BufReader};
use fixed_buffer::{FixedBuf};
use std::fs::File;
use std::path::Path;

fn main() {
    code_it();
    playground();
    vec_test();
    //read_and_code_it();
}

fn read_and_code_it(){
    let path = Path::new("image.png");
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    const LEN: usize = 4096;
    let mut lines:Vec<[u8; LEN]> = Vec::new();
    let mut buffer = [0u8; LEN];
    while file.read(&mut buffer).unwrap() > 0 {
        lines.push(buffer);
        buffer = [0u8; LEN];
        //println!("{:?}", buffer);
    }
    println!("Vector size is {} lines.", lines.len());

    let parity = 10;
    for _i in 0..parity {
        lines.push([0; LEN]);
        //println!("{}", _i);
    }

        
    /*for _i in 0..lines.len(){
        println!("row {} = {:?}", _i, lines[_i]);
    }*/
    println!("Matrix is of length {}", (lines.len()));
    println!("Vector size is {} lines plus {} lines of parity.", lines.len()-parity, parity);

    //const ROWS: usize = lines.len() as usize;
    //let mut line_array =[[0u8; LEN]; ROWS];

    let r = ReedSolomon::new(lines.len()-parity, parity).unwrap();
    r.encode(&mut lines).unwrap();

    //create a copy of the encoded file to work with.
    let mut shards: Vec<_> = lines.iter().cloned().map(Some).collect();

    //remove 2 shards for reconstruction later
    shards[0] = None;
    shards[4] = None;

    // Try to reconstruct missing shards
    //r.reconstruct(&mut shards).unwrap();

    // Convert back to normal shard arrangement
    let result: Vec<_> = shards.into_iter().filter_map(|x| x).collect();

    assert!(r.verify(&result).unwrap());
    assert_eq!(lines, result);
}

fn code_it(){
    let r = ReedSolomon::new(3, 2).unwrap(); // 3 data shards, 2 parity shards

    let mut master_copy = shards!(
        [0, 1,  2,  3],
        [4, 5,  6,  7],
        [8, 9, 10, 11],
        [0, 0,  0,  0], // last 2 rows are parity shards
        [0, 0,  0,  0]
    );

    // Construct the parity shards
    r.encode(&mut master_copy).unwrap();

    // Make a copy and transform it into option shards arrangement
    // for feeding into reconstruct_shards
    let mut shards: Vec<_> = master_copy.iter().cloned().map(Some).collect();

    // We can remove up to 2 shards, which may be data or parity shards
    shards[0] = None;
    shards[4] = None;

    // Try to reconstruct missing shards
    r.reconstruct(&mut shards).unwrap();

    // Convert back to normal shard arrangement
    let result: Vec<_> = shards.into_iter().filter_map(|x| x).collect();

    assert!(r.verify(&result).unwrap());
    assert_eq!(master_copy, result);
    println!("{}", "finished code_it.")
}

fn playground(){
    let r = ReedSolomon::new(4, 2).unwrap();

    let mut master_copy = shards!(
        [0, 1,  2,  3, 9],
        [3, 9,  7,  3, 9],
        [4, 5,  6,  7, 12],
        [8, 9, 10, 11, 80],
        [0, 0,  0,  0, 0], // last 2 rows are parity shards
        [0, 0,  0,  0, 0]
    );

    // Construct the parity shards
    r.encode(&mut master_copy).unwrap();

    // Make a copy and transform it into option shards arrangement
    // for feeding into reconstruct_shards
    let mut shards: Vec<_> = master_copy.iter().cloned().map(Some).collect();

    // We can remove up to 2 shards, which may be data or parity shards
    shards[0] = None;
    shards[4] = None;

    // Try to reconstruct missing shards
    r.reconstruct(&mut shards).unwrap();

    // Convert back to normal shard arrangement
    let result: Vec<_> = shards.into_iter().filter_map(|x| x).collect();

    assert!(r.verify(&result).unwrap());
    assert_eq!(master_copy, result);
    println!("{}", "finished playground.")
}

fn vec_test(){
    let r = ReedSolomon::new(4, 2).unwrap(); // 4 data shards, 2 parity shards

    let master_copy = shards!(
        [0, 1,  2,  3, 9],
        [3, 9,  7,  3, 9],
        [4, 5,  6,  7, 12],
        [8, 9, 10, 11, 80],
        [0, 0,  0,  0, 0], // last 2 rows are parity shards
        [0, 0,  0,  0, 0]
    );


    const LEN: usize = 5;
    let mut lines: std::vec::Vec<std::vec::Vec<u8>> = Vec::new();

    for _i in 0..4 {
        let mut row_vec:Vec<u8> = Vec::new();

        for _j in 0..LEN{
            row_vec.push(master_copy[_i][_j])
        }

        lines.push(row_vec);
        println!("line row {} = {:?}", _i, lines[_i]);
    }

    //add two parity rows
    let mut row_vec:Vec<u8> = Vec::new();
    for _i in 0..lines[0].len(){
        row_vec.push(0);
    }
    lines.push(row_vec);
    let mut row_vec:Vec<u8> = Vec::new();
    for _i in 0..lines[0].len(){
        row_vec.push(0);
    }
    lines.push(row_vec);

    r.encode(&mut lines).unwrap();
    println!(" created {} shards,", r.data_shard_count());
    println!(" created {} parity,", r.parity_shard_count());

    let mut _i = 0;
    for shard in lines.iter(){
        println!("shard {} of line = {:?}", _i, shard);
        _i = _i +1;
    }

    //clone lines into shards so some shards can be removed
    let mut shards: Vec<_> = lines.iter().cloned().map(Some).collect();

    // We can remove up to 2 shards, which may be data or parity shards
    shards[0] = None;
    shards[4] = None;

    let mut _i = 0;
    for shard in shards.iter(){
        println!("shard {} of shards = {:?}", _i, shards[_i]);
        _i = _i + 1;
    }

    // Try to reconstruct missing shards
    r.reconstruct(&mut shards).unwrap();

    let mut _i = 0;
    for shard in shards.iter(){
        println!("shard {} of shards = {:?}", _i, shards[_i]);
        _i = _i + 1;
    }

    // Convert back to normal shard arrangement
    let result: Vec<_> = shards.into_iter().filter_map(|x| x).collect();
    
    assert!(r.verify(&result).unwrap());
    assert_eq!(lines, result);

    println!("{}", "finished vec_test()")
}