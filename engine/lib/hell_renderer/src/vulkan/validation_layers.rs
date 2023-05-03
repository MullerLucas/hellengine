use hell_core::error::{ErrToHellErr, HellResult};
use hell_utils::conversion;



pub fn check_validation_layer_support(entry: &ash::Entry, required_layers: &[&str]) -> HellResult<bool> {
    let props = entry.enumerate_instance_layer_properties().to_render_hell_err()?;

    for layer in required_layers {

        let res = props.iter()
            .map(|p| p.layer_name)
            .find(|p| {
                match conversion::c_str_from_char_slice(p).to_str() {
                    Ok(s)  => s == *layer,
                    Err(_) => false,
                }
            });

        if res.is_some()  {
            println!("validation-layer: {layer} is supported!");
        } else {
            eprintln!("validation-layer: {layer} is not supported!");
            return Ok(false);
        }
    }

    Ok(true)
}

