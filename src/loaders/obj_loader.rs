use std::path::Path;
use wavefront::Obj;
use crate::gpu::vertex3::Vertex3;
use crate::loaders::mesh_loader::{MeshFileLoader, MeshInfo};

pub struct ObjLoader;

impl MeshFileLoader for ObjLoader {
    fn load_mesh(&self, path: &Path) -> anyhow::Result<MeshInfo> {
        let obj_info = Obj::from_file(path)
            .expect("Couldn't read wavefront file");

        let index_list = obj_info
            .triangles()
            .flat_map(|t| {
                t.iter()
                    .map(|i| i.position_index() as u32)
                    .collect::<Vec<u32>>()
            })
            .collect::<Vec<u32>>();

        Ok(MeshInfo {
            vertices: obj_info.positions().to_vec(),
            normals: obj_info.normals().to_vec(),
            indices: index_list,
            uvs: obj_info.uvs().to_vec()
        })
    }
}