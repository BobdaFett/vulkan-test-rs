use std::path::Path;
use wavefront::Obj;
use crate::assets::loaders::common::{MeshLoadInfo, Submesh};

pub struct ObjLoader;

impl ObjLoader {
    fn load_mesh(&self, path: &Path) -> anyhow::Result<MeshLoadInfo> {
        let obj_info = Obj::from_file(path).expect("Couldn't read wavefront file");

        let index_list = obj_info
            .triangles()
            .flat_map(|t| {
                t.iter()
                    .map(|i| i.position_index() as u32)
                    .collect::<Vec<u32>>()
            })
            .collect::<Vec<u32>>();

        // Wavefront `.mtl` material groups aren't loaded yet, so the whole mesh is treated as a
        // single submesh with no assigned material.
        let submeshes = vec![Submesh {
            material_index: None,
            index_start: 0,
            index_count: index_list.len(),
        }];

        Ok(MeshLoadInfo {
            vertices: obj_info.positions().to_vec(),
            normals: obj_info.normals().to_vec(),
            indices: index_list,
            uvs: obj_info.uvs().to_vec(),
            submeshes,
        })
    }
}
