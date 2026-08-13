use std::collections::HashMap;

pub struct MaterialRegistry {
    materials: HashMap<String, Material>
}

impl MaterialRegistry {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new()
        }
    }

    pub fn register(&mut self, id: String, material: Material) {
        self.materials.insert(id, material);
    }

    pub fn get(&self, id: &String) -> Option<&Material> {
        self.materials.get(id)
    }
}

/// Encapsulates all the information that makes up a PBR (physically based rendering) material.
/// These materials should be stored in a [`MaterialRegistry`], with a unique key.
pub struct Material {
    /// The base color of the material.
    pub base_color: [f32; 4],
    /// The metallic value of the material. This should be a number between 0 and 1.
    pub metallic: f32,
    /// The roughness value of the material. This should be a number between 0 and 1.
    pub roughness: f32,
    /// The path of the normal map for this material, if any.
    pub normal_map: Option<String>,
    /// The path of the base color's texture, if any.
    pub base_color_texture: Option<String>,
}
