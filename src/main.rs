use crate::compile::compiler::Compiler;
use crate::execution::vm::VM;
use crate::parsing::ast::Expr;
use peg::error::ParseError;
use std::env;
extern crate indexmap;

mod compile;
mod data;
mod execution;
mod parsing;
#[cfg(test)]
mod test;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("please provide file to run as argument.");
        return;
    }
    let filename = args.get(1).unwrap();
    let file_content = std::fs::read_to_string(filename).expect("failed to read file");

    //normalize string - make all lines joined with \n (incase running on windows)
    let file_content = file_content.lines().collect::<Vec<_>>().join("\n");
    let tokens = match parsing::lexer::tokenize(&file_content) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    #[cfg(feature = "print-tokens")]
    {
        for token in &tokens {
            print!("{}", token.kind);
        }
        println!();
    }

    use parsing::parser::program_parser;

    let statements = match program_parser::program(&tokens) {
        Ok(s) => s,
        Err(ParseError { location, expected }) => {
            println!("{:?}", ParseError { location, expected });
            println!("{:?}", tokens[location]);
            return;
        }
    };

    #[cfg(feature = "print-ast")]
    println!("{:?}", statements);

    let (variable_types, closed_names) = compile::syntax_level_check::check(&statements).unwrap();
    for (token, index_map) in &variable_types {
        println!("{:?}", token);
        for item in index_map.iter() {
            println!("     {} {:?}", item.0, item.1);
        }
    }

    let statements = compile::syntax_level_opt::optimize(statements);

    #[cfg(feature = "print-ast")]
    println!("{:?}", statements);

    let (per_chunk_indices, chunks) =
        Compiler::compile(&statements, variable_types, closed_names).unwrap();

    #[cfg(feature = "print-chunk")]
    for chunk in &chunks {
        println!("{}", chunk);
    }

    println!("running");
    let mut vm = VM::new();
    vm.run(&chunks).unwrap_or_else(|error| {
        println!(
            "error {:?} at instruction {}\nat line {}",
            error,
            chunks[error.chunk_index].code[error.opcode_index],
            per_chunk_indices[error.chunk_index][error.opcode_index],
        );
    }); /**/
}

fn normalize_string(s: String) -> String {
    s.lines().collect::<Vec<_>>().join("\n")
}

pub fn run_file(filename: &str) -> Result<(), String> {
    let file_content = std::fs::read_to_string(filename)
        .map_err(|_e| format!("failed to read file {}", filename))?;
    let file_content = normalize_string(file_content);
    let tokens = parsing::lexer::tokenize(&file_content)?;
    use parsing::parser::program_parser;

    let statements: Box<Expr> = program_parser::program(&tokens)
        .map_err(|e| format!("{:?}\n{:?}", e, tokens[e.location]))?;

    let (variable_types, closed_names) = compile::syntax_level_check::check(&statements)?;
    let statements = compile::syntax_level_opt::optimize(statements);
    let (per_chunk_indices, chunks) = Compiler::compile(&statements, variable_types, closed_names)?;
    let mut vm = VM::new();
    vm.run(&chunks).map_err(|error| {
        format!(
            "error {:?} at instruction {}\nat line {}",
            error,
            chunks[error.chunk_index].code[error.opcode_index],
            per_chunk_indices[error.chunk_index][error.opcode_index],
        )
    })?;

    Ok(())
}
