use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLines;
use oxidized_navigation::NavMesh;


fn process_navmesh_render(
    state: Res<MyEditorState>,
    nav_mesh: Res<NavMesh>,
    mut lines: ResMut<DebugLines>,
) {
    if !state.navmesh_render_enabled {
        return;
    }

    if let Ok(nav_mesh) = nav_mesh.get().read() {
        for (tile_coord, tile) in nav_mesh.get_tiles().iter() {
            let tile_color = Color::Rgba {
                red: 0.0,
                green: (tile_coord.x % 10) as f32 / 10.0,
                blue: (tile_coord.y % 10) as f32 / 10.0,
                alpha: 1.0,
            };
            // Draw polygons.
            for poly in tile.polygons.iter() {
                let indices = &poly.indices;
                for i in 0..indices.len() {
                    let a = tile.vertices[indices[i] as usize];
                    let b = tile.vertices[indices[(i + 1) % indices.len()] as usize];

                    lines.line_colored(a, b, 0.0, tile_color);
                }
            }

            // Draw vertex points.
            for vertex in tile.vertices.iter() {
                lines.line_colored(*vertex, *vertex + Vec3::Y, 0.0, tile_color);
            }
        }
    }
}
