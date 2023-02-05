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
                res.info = Some(CrapInfoDef::new(pair.into_inner()));
            },
            Rule::scope_decl  => {
                let scope = CrapScopeDef::new(pair.into_inner());
                res.scopes.insert(scope.name.clone().to_string(), scope);
            },
            Rule::shader_decl => {
                let shader = CrapShaderDef::new(pair.into_inner());
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
pub struct CrapInfoDef {
    pub crap_ver: CrapInfoVarDef,
    pub name: CrapInfoVarDef,
}

impl CrapInfoDef {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        let mut info_block = pairs.next().unwrap().into_inner();
        let crap_ver = CrapInfoVarDef::new(info_block.next().unwrap().into_inner());
        let name = CrapInfoVarDef::new(info_block.next().unwrap().into_inner());

        Self {
            crap_ver,
            name,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapInfoVarDef {
    pub ident: String,
    pub val: String,
}

impl CrapInfoVarDef {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        let ident = pairs.next().unwrap().as_str().to_lowercase();
        let val = pairs.next().unwrap().as_str().to_lowercase();

        Self {
            ident,
            val,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapScopeDef {
    pub name: String,
    pub buffers: Vec<CrapUniformBufferDef>,
    pub samplers: Vec<CrapUniformSamplerDef>,
}

impl CrapScopeDef {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        // println!("scope =============: {:?}", pairs);

        let name = pairs.next().unwrap().as_str().to_lowercase();
        let scope_block = pairs.next().unwrap().into_inner();

        let mut buffers = Vec::new();
        let mut samplers = Vec::new();

        for pair in scope_block {
            match pair.as_rule() {
                Rule::uniform_buffer => {
                    buffers.push(CrapUniformBufferDef::new(pair.into_inner()));
                }
                Rule::uniform_sampler => {
                    samplers.push(CrapUniformSamplerDef::new(pair.into_inner()));
                },
                _ => unreachable!()
            }
        }

        Self {
            name,
            buffers,
            samplers,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapUniformBufferDef {
    pub var_ubos: Vec<CrapVarUboDef>,
    pub ident: String,
}

impl CrapUniformBufferDef {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        let mut var_ubos = Vec::new();

        while let Rule::var_ubo = pairs.peek().unwrap().as_rule() {
            var_ubos.push(CrapVarUboDef::new(pairs.next().unwrap().into_inner()));
        }

        let ident = pairs.next().unwrap().as_str().to_lowercase();

        Self {
            ident,
            var_ubos,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapVarUboDef {
    pub type_ubo: String,
    pub ident: String
}

impl CrapVarUboDef {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        let type_ubo = pairs.next().unwrap().as_str().to_lowercase();
        let ident = pairs.next().unwrap().as_str().to_lowercase();

        Self {
            type_ubo,
            ident,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapUniformSamplerDef {
    pub type_sampler: String,
    pub ident: String,
}

impl CrapUniformSamplerDef {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        let type_sampler = pairs.next().unwrap().as_str().to_string();
        let ident = pairs.next().unwrap().as_str().to_lowercase();

        Self {
            ident,
            type_sampler,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapShaderDef {
    pub name: String,
    pub uniform_usages: Vec<CrapUniformUsage>,
    pub raw_code: CrapRawCode,
}

impl CrapShaderDef {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        println!("shader =============: {:?}", pairs);
        let name = pairs.next().unwrap().as_str().to_lowercase();
        let mut shader_block = pairs.next().unwrap().into_inner();
        let mut uniform_usages = Vec::new();

        while let(Rule::uniform_usage) = shader_block.peek().unwrap().as_rule() {
            uniform_usages.push(CrapUniformUsage::new(shader_block.next().unwrap().into_inner()))
        }

        let raw_code = CrapRawCode::new(&shader_block.next().unwrap());


        Self {
            name,
            uniform_usages,
            raw_code,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapUniformUsage {
    pub scope_ident: String,
    pub field_ident: String,
}

impl CrapUniformUsage {
    pub fn new(mut pairs: Pairs<Rule>) -> Self {
        let scope_ident = pairs.next().unwrap().as_str().to_lowercase();
        let field_ident = pairs.next().unwrap().as_str().to_lowercase();

        Self {
            scope_ident,
            field_ident,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapRawCode {
    pub code: String,
}

impl CrapRawCode {
    pub fn new(pair: &Pair<Rule>) -> Self {
        let code = pair.as_str().to_owned();

        Self {
            code ,
        }
    }
}


// -----------------------------------------------------------------------------
