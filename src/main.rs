

#[macro_use(shards)]
extern crate reed_solomon_erasure;

use reed_solomon_erasure::galois_8::ReedSolomon;
// or use the following for Galois 2^16 backend
//use reed_solomon_erasure::galois_16::ReedSolomon;

//https://docs.rs/reed-solomon-erasure/latest/reed_solomon_erasure/

use std::io::{self, Read, Write, ErrorKind, BufReader};
use std::fs::File;
use std::path::Path;
                                                                                                                                                             
fn main() {
    code_it();
    playground();
    vec_test();
    read_and_code_it();
}

//This function reads in an image file, shards it, deletes some shards and reassembles it. The file is then written as a copy of the image
fn read_and_code_it(){
    let path = Path::new("image.jpg");
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    const LEN: usize = 4096;
    let mut file_slices: std::vec::Vec<std::vec::Vec<u8>> = Vec::new();
    let mut slice_buffer = [0u8; LEN];
    let mut bytes_in_last_row = 0;

    //slice the file into 4096 byte slices
    let mut end_of_file = false;
    while  end_of_file == false {
        let bytes_read = file.read(&mut slice_buffer).unwrap();

        if bytes_read != 0{
            let mut slice_vec: Vec<u8> = Vec::new();
            bytes_in_last_row = bytes_read;
            for _i in 0..LEN{
                slice_vec.push(slice_buffer[_i]);
            }

            /*for (place, data) in line_vec.iter_mut().zip(line_buffer.iter()){
                line_vec.push(*data);
                *place = *data;
            }*/

            file_slices.push(slice_vec);
            slice_buffer = [0u8; LEN];
        } else {
            end_of_file = true;
        }
    }    
    println!("Vector size is {} lines.", file_slices.len());

    //add parity slices
    let parity = 10;
    for _i in 0..parity {
        let mut parity_vec: Vec<u8> = Vec::new();
        for _j in 0..LEN{
            parity_vec.push(0);
        }
        file_slices.push(parity_vec);
    }

    //print out some details    
    /*for _i in 0..lines.len(){
        println!("row {} = {:?}", _i, lines[_i]);
    }*/
    println!("Matrix is of length {}", (file_slices.len()));
    println!("Vector size is {} lines plus {} lines of parity.", file_slices.len()-parity, parity);

    let r = ReedSolomon::new(file_slices.len()-parity, parity).unwrap();
    r.encode(&mut file_slices).unwrap();

    //create a copy of the encoded file to work with.
    let mut shards: Vec<_> = file_slices.iter().cloned().map(Some).collect();

    //remove 2 shards for reconstruction later
    shards[0] = None;
    shards[4] = None;
    shards[15] = None;

    // Try to reconstruct missing shards
    r.reconstruct(&mut shards).unwrap();

    // Convert back to normal shard arrangement
    let result: Vec<_> = shards.into_iter().filter_map(|x| x).collect();

    assert!(r.verify(&result).unwrap());
    assert_eq!(file_slices, result);

    /*for slice in result.iter(){
        println!("{:?}", slice);
    }*/
    println!("{}", "File reconstruction successful.");

    //write the file to new file to test the image
    let path = Path::new("copy.jpg");
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let mut slice_buffer = [0u8; LEN];
    for row in 0..result.len()-parity{
        for col in 0..result[row].len(){
            if row < result.len()-parity-1{
                slice_buffer[col] = result[row][col];
            } else{
                if col < bytes_in_last_row {
                    slice_buffer[col] = result[row][col];
                    let byte_buffer = [slice_buffer[col]];
                    file.write_all(&byte_buffer);
                } else {
                    break;
                }
            }
        }
        if row < result.len()-parity-1{
            file.write_all(&slice_buffer).unwrap();
        }
    }

    /*for row in &result{
        let data = row;
        for i in data.iter(){
            write!(f,"{}", i); 
        }
    }*/
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

//some simple manipulation of examples found on the web
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

//translating the static example into a dynamic vector.
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

    //start a test to dynamically allocate the slices
    const LEN: usize = 5;
    let mut lines: std::vec::Vec<std::vec::Vec<u8>> = Vec::new();

    //create a row for each row in master_copy
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

    //let mut _i = 0;
    for _r in 0..shards.len(){    
        println!("shard {} of shards = {:?}", _r, shards[_r]);
    }

    // Try to reconstruct missing shards
    r.reconstruct(&mut shards).unwrap();

    //just print the shards for visual inspection
    for _r in 0..shards.len(){    
        println!("shard {} of shards = {:?}", _r, shards[_r]);
    }

    // Convert back to normal shard arrangement
    let result: Vec<_> = shards.into_iter().filter_map(|x| x).collect();
    
    assert!(r.verify(&result).unwrap());
    assert_eq!(lines, result);

    println!("{}", "finished vec_test()")
}