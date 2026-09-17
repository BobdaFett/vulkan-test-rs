use std::collections::HashMap;
use std::sync::Arc;
use vulkano::image::view::ImageView;
use vulkano::image::sampler::Sampler;

/// A texture handle that contains the texture's [`ImageView`] and [`Sampler`].
pub struct TextureHandle {
    /// The texture's [`ImageView`].
    pub image_view: Arc<ImageView>,
    /// The [`Sampler`] associated with the image. This may be a shared sampler,
    /// as they do not contain image-specific information.
    pub sampler: Arc<Sampler>
}

pub struct TextureRegistry {
    textures: HashMap<String, TextureHandle>,
}

impl TextureRegistry {
    pub fn new() -> Self {
        TextureRegistry {
            textures: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: String, texture: TextureHandle) {
        self.textures.insert(id, texture);
    }

    pub fn get(&self, id: &str) -> Option<&TextureHandle> {
        self.textures.get(id)
    }
}
