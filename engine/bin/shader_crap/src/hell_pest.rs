use std::{hint::unreachable_unchecked, collections::HashMap};

use hell_error::HellResult;
use pest::{self, Parser, iterators::{Pair, Pairs}};
use pest_derive::{self, Parser};


#[derive(Parser)]
#[grammar = "pest/test.pest"]
pub struct CSVParser;

pub fn run() -> HellResult<()> {
    let input = std::fs::read_to_string("pest/test.glsl").unwrap();
    println!("START");
    let file = CSVParser::parse(Rule::file, &input).unwrap()
        .next().unwrap()
        .into_inner();

    println!("DONE");

    // println!("{:#?}", file);
    let mut res = CrapFile::new();

    for pair in file {
        // println!("PAIR: {:?}", pair);

        match pair.as_rule() {
            Rule::info_decl   => {
                res.info = Some(CrapInfoDef::new(pair.into_inner())?);
            },
            Rule::scope_decl  => {
                let scope = CrapScopeDef::new(pair.into_inner())?;
                res.scopes.insert(scope.name.clone().to_string(), scope);
            },
            Rule::shader_decl => {
                let shader = CrapShaderDef::new(pair.into_inner())?;
                res.shaders.insert(shader.name.clone(), shader);
            },
            Rule::EOI => { }
            _ => {
                println!("INVALID: {:?}", pair.as_rule());
                println!("{:#?}", pair);
                unreachable!();
            }
        }
    }

    println!("RESULT: ======================================================================");
    println!("{:#?}", res);

    Ok(())
}

// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapFile {
    pub info: Option<CrapInfoDef>,
    pub scopes: HashMap<String, CrapScopeDef>,
    pub shaders: HashMap<String, CrapShaderDef>,
}

impl CrapFile {
    pub fn new() -> Self {
        Self {
            info: None,
            scopes: HashMap::new(),
            shaders: HashMap::new(),
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapInfoDef { }

impl CrapInfoDef {
    pub fn new(pairs: Pairs<Rule>) -> HellResult<Self> {
        println!("info =============: {:?}", pairs);
        Ok(Self { })
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapScopeDef {
    pub name: String,
}

impl CrapScopeDef {
    pub fn new(mut pairs: Pairs<Rule>) -> HellResult<Self> {
        println!("scope =============: {:?}", pairs);
        let name = pairs.next().unwrap().as_str().to_lowercase();

        Ok(Self {
            name,
        })
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapShaderDef {
    pub name: String,
}

impl CrapShaderDef {
    pub fn new(mut pairs: Pairs<Rule>) -> HellResult<Self> {
        println!("shader =============: {:?}", pairs);
        let name = pairs.next().unwrap().as_str().to_lowercase();

        Ok(Self {
            name,
        })
    }
}

// -----------------------------------------------------------------------------
