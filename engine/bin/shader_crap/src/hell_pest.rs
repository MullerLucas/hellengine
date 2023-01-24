use hell_error::HellResult;
use pest::{self, Parser};
use pest_derive::{self, Parser};


#[derive(Parser)]
#[grammar = "pest/test.pest"]
pub struct CSVParser;

pub fn run() -> HellResult<()> {
    let input = std::fs::read_to_string("pest/test.glsl").unwrap();
    println!("START");
    let file = CSVParser::parse(Rule::file, &input).unwrap()
        .next().unwrap();

    println!("DONE");

    println!("{:#?}", file);
    // for record in file.into_inner() {
    //     match record.as_rule() {
    //         Rule::record => {
    //             for field in record.into_inner() {
    //                 println!("field: '{}'", field.as_str());
    //             }
    //         }
    //
    //         // Rule::file => todo!(),
    //         // Rule::EOI => todo!(),
    //         // Rule::field => todo!(),
    //
    //         _ => { }
    //     }
    // }

    Ok(())
}
