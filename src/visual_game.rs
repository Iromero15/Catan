use bevy::prelude::*;

use crate::types::*;
use crate::game_logic::*;
use crate::setup::*;

// =====================================================
// PLUGIN
// =====================================================

pub struct VisualGamePlugin;

impl Plugin for VisualGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.18)))
            .init_resource::<VisualBoard>()
            .init_resource::<GameUiState>()
            .add_systems(Startup, setup_visual_board)
            .add_systems(Update, (
                update_hover_vertex,
                handle_clicks,
                repaint_from_board,
                update_ui_text,
            ));
    }
}

// =====================================================
// RECURSOS
// =====================================================

#[derive(Resource)]
pub struct VisualBoard {
    pub board: Board,
    /// centros de tiles precalculados: index = tile_id
    pub tile_centers: Vec<Vec2>,
}

impl Default for VisualBoard {
    fn default() -> Self {
        let mut board = setup_board();

        // 💡 agregamos al menos 2 jugadores para que place_house no falle
        add_player(&mut board);
        add_player(&mut board);
        // si querés los 4:
        // add_player(&mut board);
        // add_player(&mut board);

        let tile_centers = generate_tile_centers();
        Self { board, tile_centers }
    }
}

#[derive(Resource)]
pub struct GameUiState {
    pub current_player: PlayerType,
    pub current_tool: CurrentTool,
    pub hovered_vertex: Option<usize>,
}

impl Default for GameUiState {
    fn default() -> Self {
        Self {
            current_player: PlayerType::Player1,
            current_tool: CurrentTool::PlaceSettlement,
            hovered_vertex: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CurrentTool {
    PlaceSettlement,
    PlaceRoad,
    MoveRobber,
}

// =====================================================
// COMPONENTES
// =====================================================

#[derive(Component)]
struct TileViz {
    pub tile_id: usize,
}

#[derive(Component)]
struct VertexViz {
    pub vertex_id: usize,
}

#[derive(Component)]
struct EdgeViz {
    pub edge_id: usize,
}

#[derive(Component)]
struct UiTextTag;

// =====================================================
// STARTUP
// =====================================================

fn setup_visual_board(
    mut commands: Commands,
    vis_board: Res<VisualBoard>,
    asset_server: Res<AssetServer>,
) {
    // cámara
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("FiraSans-Bold.ttf");;

    // HUD
    commands.spawn((
        TextBundle::from_section(
            "Catan Visual",
            TextStyle {
                font: font.clone(),
                font_size: 20.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..Default::default()
        }),
        UiTextTag,
    ));

    // ------------------ TILES ------------------
    for (i, tile) in vis_board.board.tiles.iter().enumerate() {
        let center = vis_board.tile_centers[i];

        let color = match tile.material {
            MaterialType::Wood   => Color::srgb(0.22, 0.65, 0.25),
            MaterialType::Brick  => Color::srgb(0.70, 0.35, 0.25),
            MaterialType::Sheep  => Color::srgb(0.75, 0.9, 0.6),
            MaterialType::Wheat  => Color::srgb(0.9, 0.8, 0.5),
            MaterialType::Stone  => Color::srgb(0.6, 0.6, 0.7),
            MaterialType::Dessert => Color::srgb(0.9, 0.85, 0.65),
        };

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(95., 95.)), // un “hex” cuadrado
                    ..Default::default()
                },
                transform: Transform::from_xyz(center.x, center.y, 0.0),
                ..Default::default()
            },
            TileViz { tile_id: i },
        ));

        // número / ladrón
        let text = if tile.has_robber || tile.material == MaterialType::Dessert {
            "R".to_string()
        } else {
            format!("{}", tile.number)
        };

        commands.spawn(Text2dBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    font: font.clone(),
                    font_size: 26.0,
                    color: Color::BLACK,
                },
            )
            .with_justify(JustifyText::Center),
            transform: Transform::from_xyz(center.x, center.y + 2.0, 1.0),
            ..Default::default()
        });
    }

    // ------------------ VÉRTICES ------------------
    for vid in 0..vis_board.board.vertices.len() {
        if let Some((tile_idx, corner_idx)) =
            find_tile_and_corner_for_vertex(&vis_board.board, vid)
        {
            let tile_center = vis_board.tile_centers[tile_idx];
            let off = corner_offset_pointy(corner_idx, 55.0);
            let pos = tile_center + off;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.12, 0.12, 0.12),
                        custom_size: Some(Vec2::new(16., 16.)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(pos.x, pos.y, 5.0),
                    ..Default::default()
                },
                VertexViz { vertex_id: vid },
            ));
        }
    }

    // ------------------ EDGES ------------------
    for (i, edge) in vis_board.board.edges.iter().enumerate() {
        let v1 = edge.vertices.0;
        let v2 = edge.vertices.1;

        if let (Some(p1), Some(p2)) =
            (vertex_world_pos(&vis_board, v1), vertex_world_pos(&vis_board, v2))
        {
            let mid = (p1 + p2) / 2.0;
            let dir = p2 - p1;
            let len = dir.length();
            let angle = dir.y.atan2(dir.x);

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.45, 0.45, 0.45),
                        custom_size: Some(Vec2::new(len, 5.0)),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(mid.x, mid.y, 2.0),
                        rotation: Quat::from_rotation_z(angle),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                EdgeViz { edge_id: i },
            ));
        }
    }
}

// =====================================================
// SISTEMAS
// =====================================================

fn update_hover_vertex(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    vis_board: Res<VisualBoard>,
    mut ui_state: ResMut<GameUiState>,
) {
    let window = windows.single();
    if let Some(cursor) = window.cursor_position() {
        let (camera, cam_tf) = camera_q.single();
        if let Some(ray) = camera.viewport_to_world(cam_tf, cursor) {
            let world = ray.origin.truncate();
            let mut best: Option<(usize, f32)> = None;

            for vid in 0..vis_board.board.vertices.len() {
                if let Some(vpos) = vertex_world_pos(&vis_board, vid) {
                    let d = vpos.distance(world);
                    if d < 18.0 {
                        match best {
                            None => best = Some((vid, d)),
                            Some((_, bd)) if d < bd => best = Some((vid, d)),
                            _ => {}
                        }
                    }
                }
            }

            ui_state.hovered_vertex = best.map(|(id, _)| id);
        } else {
            ui_state.hovered_vertex = None;
        }
    }
}

fn handle_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    mut vis_board: ResMut<VisualBoard>,
    mut ui_state: ResMut<GameUiState>,
    // ParamSet: 0 = vértices, 1 = edges
    mut q: ParamSet<(
        Query<(&VertexViz, &mut Sprite)>,
        Query<(&EdgeViz, &mut Sprite)>,
    )>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(vertex_id) = ui_state.hovered_vertex else {
        return;
    };

    match ui_state.current_tool {
        CurrentTool::PlaceSettlement => {
            match place_house(&mut vis_board.board, ui_state.current_player, vertex_id, false) {
                Ok(_) => {
                    // pintamos vértice
                    let mut vertex_query = q.p0();
                    for (vv, mut sprite) in vertex_query.iter_mut() {
                        if vv.vertex_id == vertex_id {
                            sprite.color = player_color(ui_state.current_player);
                        }
                    }
                }
                Err(msg) => {
                    println!("Error: {}", msg);
                }
            }
        }
        CurrentTool::PlaceRoad => {
            if let Some(edge_id) = first_free_edge_adjacent_to(&vis_board.board, vertex_id) {
                match place_road(
                    &mut vis_board.board,
                    ui_state.current_player,
                    edge_id,
                    TurnPhase::Normal,
                ) {
                    Ok(_) => {
                        // pintamos edge
                        let mut edge_query = q.p1();
                        for (ev, mut sprite) in edge_query.iter_mut() {
                            if ev.edge_id == edge_id {
                                sprite.color = player_color(ui_state.current_player);
                            }
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            } else {
                println!("No hay edge libre adyacente.");
            }
        }
        CurrentTool::MoveRobber => {
            println!("Mover ladrón: lo hacemos con click en tile más adelante.");
        }
    }
}


fn repaint_from_board(
    vis_board: Res<VisualBoard>,
    mut q: ParamSet<(
        Query<(&VertexViz, &mut Sprite)>,
        Query<(&EdgeViz, &mut Sprite)>,
    )>,
) {
    // si el Board no cambió, no hacemos nada
    if !vis_board.is_changed() {
        return;
    }

    // 1) vértices
    {
        let mut vertex_q = q.p0();
        for (vv, mut sprite) in vertex_q.iter_mut() {
            let v = &vis_board.board.vertices[vv.vertex_id];
            sprite.color = match v.owner {
                Some(p) => player_color(p),
                None => Color::srgb(0.12, 0.12, 0.12),
            };
        }
    }

    // 2) edges
    {
        let mut edge_q = q.p1();
        for (ev, mut sprite) in edge_q.iter_mut() {
            let e = &vis_board.board.edges[ev.edge_id];
            sprite.color = match e.owner {
                Some(p) => player_color(p),
                None => Color::srgb(0.45, 0.45, 0.45),
            };
        }
    }
}

fn update_ui_text(
    vis_board: Res<VisualBoard>,
    ui_state: Res<GameUiState>,
    mut q: Query<&mut Text, With<UiTextTag>>,
) {
    let mut text = q.single_mut();
    let hovered = ui_state
        .hovered_vertex
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());

    let tool = match ui_state.current_tool {
        CurrentTool::PlaceSettlement => "Asentamiento",
        CurrentTool::PlaceRoad => "Camino",
        CurrentTool::MoveRobber => "Ladrón",
    };

    text.sections[0].value = format!(
        "Jugador: {:?}\nHerramienta: {}\nHover vértice: {}\nCartas dev: {}\n",
        ui_state.current_player,
        tool,
        hovered,
        vis_board.board.development_cards.len()
    );
}

// =====================================================
// HELPERS DE POSICIÓN
// =====================================================

fn player_color(p: PlayerType) -> Color {
    match p {
        PlayerType::Player1 => Color::srgb(0.9, 0.25, 0.25),
        PlayerType::Player2 => Color::srgb(0.25, 0.9, 0.25),
        PlayerType::Player3 => Color::srgb(0.25, 0.25, 0.9),
        PlayerType::Player4 => Color::srgb(0.9, 0.9, 0.25),
    }
}

/// Genera los centros de los 19 tiles usando TU orden de setup.rs
fn generate_tile_centers() -> Vec<Vec2> {
    // parámetros de layout
    let tile_w: f32 = 110.0;
    let tile_h: f32 = 95.0;
    let start_y: f32 = 200.0;
    let center_x: f32 = 0.0;

    let rows: Vec<Vec<usize>> = vec![
        vec![0, 1, 2],
        vec![11, 12, 13, 3],
        vec![10, 17, 18, 14, 4],
        vec![9, 16, 15, 5],
        vec![8, 7, 6],
    ];

    let mut centers = vec![Vec2::ZERO; 19];

    for (row_idx, row) in rows.iter().enumerate() {
        let y = start_y - (row_idx as f32) * tile_h;
        let row_len = row.len() as f32;
        let total_w = (row_len - 1.0) * tile_w;
        let start_x = center_x - total_w / 2.0;

        for (col_idx, &tile_id) in row.iter().enumerate() {
            let x = start_x + (col_idx as f32) * tile_w;
            centers[tile_id] = Vec2::new(x, y);
        }
    }

    centers
}

/// busca una tile que use ese vértice y devuelve (tile_idx, corner_idx)
fn find_tile_and_corner_for_vertex(board: &Board, vertex_id: usize) -> Option<(usize, usize)> {
    for (ti, tile) in board.tiles.iter().enumerate() {
        for (ci, &v) in tile.vertices.iter().enumerate() {
            if v == vertex_id {
                return Some((ti, ci));
            }
        }
    }
    None
}

// dibujamos los 6 vértices de un hex pointy (arriba) con radio dado
fn corner_offset_pointy(corner_idx: usize, radius: f32) -> Vec2 {
    let angle_deg: f32 = match corner_idx {
        0 => 90.0,
        1 => 30.0,
        2 => -30.0,
        3 => -90.0,
        4 => -150.0,
        5 => 150.0,
        _ => 0.0,
    };
    let rad = angle_deg.to_radians();
    Vec2::new(rad.cos() * radius, rad.sin() * radius)
}

/// posición world de un vértice
fn vertex_world_pos(vis_board: &VisualBoard, vertex_id: usize) -> Option<Vec2> {
    let mut acc = Vec2::ZERO;
    let mut count = 0.0_f32;

    for (tile_idx, tile) in vis_board.board.tiles.iter().enumerate() {
        for (corner_idx, &v) in tile.vertices.iter().enumerate() {
            if v == vertex_id {
                let center = vis_board.tile_centers[tile_idx];
                let off = corner_offset_pointy(corner_idx, 55.0);
                acc += center + off;
                count += 1.0;
            }
        }
    }

    if count > 0.0 {
        Some(acc / count)
    } else {
        None
    }
}

/// edge libre adyacente a un vértice
fn first_free_edge_adjacent_to(board: &Board, vertex_id: usize) -> Option<usize> {
    for (i, e) in board.edges.iter().enumerate() {
        if e.owner.is_some() {
            continue;
        }
        if e.vertices.0 == vertex_id || e.vertices.1 == vertex_id {
            return Some(i);
        }
    }
    None
}
