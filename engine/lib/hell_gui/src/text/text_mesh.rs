use hell_common::transform::Transform;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct HellFont {
    mesh: usize,
    material: usize,
}

impl HellFont {
    pub fn new(mesh: usize, material: usize) -> Self {
        Self {
            mesh, material
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct TextMesh {
    font: Option<HellFont>,
    transform: Transform,

    char_transforms: Vec<Transform>,
    txt: Option<String>,
}

impl TextMesh {
    pub fn new(font: Option<HellFont>) -> Self {
        let transform = Transform::default();

        Self {
            font,
            transform,

            char_transforms: vec![],
            txt: None,
        }
    }

    pub fn char_transforms(&self) -> &[Transform] {
        &self.char_transforms
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn set_text(&mut self, txt: impl Into<String>) {
        let txt = txt.into();
        let new_len = txt.len();

        self.txt = Some(txt);

        self.char_transforms.resize_with(new_len, Transform::default);

        let mut curr_x = 0.0;
        for t in &mut self.char_transforms {
            t.translation = glam::vec3(curr_x, 0.0, 0.0);
            curr_x += 1.0;
        }
    }

    pub fn set_font(&mut self, font: Option<HellFont>) {
        self.font = font;
    }
}
