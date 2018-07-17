// Main function to invoke lexer, parser, codegen modules

use std::env;
use std::fs::File;
use std::io::prelude::*;

mod lexer;
mod parser;
mod cliftinstbuilder;
mod patternmatcher;

fn main () {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Error: not enough arguments passed to souper parser");
    }

    let filename = &args[1];
    let mut file = File::open(filename).expect("file not found");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    // Start Tokenizing
    lexer::start_lexer(&contents);

    // Start Parsing
    let souper_insts = parser::parse(&contents);

    // Codegen
    let clift_insts = cliftinstbuilder::transform_souper_to_clift_insts(souper_insts);

    for c in clift_insts {
        println!("======== Cton inst created =======");
        cliftinstbuilder::getCtonValDefName(c.valuedef);
        cliftinstbuilder::getCtonOpCodeName(c.opcode);
        if let Some(cops) = c.cops {
            for op in cops {
                println!("idx_val = {:?}, const val = {:?}", op.idx_val, op.const_val);
            }
        }
    }

    // Codegen Phase 2: Generate pattern matcher
//    patternmatcher::generate_patterns(clift_insts);
}
