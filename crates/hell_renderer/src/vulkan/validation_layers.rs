use hell_utils::conversion;



pub fn check_validation_layer_support(entry: &ash::Entry, required_layers: &[&str]) -> bool {
    let props = entry.enumerate_instance_layer_properties().unwrap();

    for layer in required_layers {

        let res = props.iter()
            .map(|p| p.layer_name)
            .find(|p| conversion::c_str_from_char_slice(p).to_str().unwrap() == *layer);

        if res.is_some()  {
            println!("validation-layer: {layer} is supported!");
        } else {
            eprintln!("validation-layer: {layer} is not supported!");
            return false;
        }
    }

    true
}

