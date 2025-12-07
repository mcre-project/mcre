use bevy::{asset::LoadState, platform::collections::HashMap, prelude::*};

use crate::{
    AppState, LoadingState,
    chunk::{
        Chunk, ChunkComponent,
        asset::ChunkAssetLoader,
        generate::spawn_test_chunk,
        math::{pos::ChunkPosition, size::ChunkSize},
    },
    textures::BlockTextures,
};

const DEFAULT_CHUNK_RADIUS: usize = 10;
const DEFAULT_CHUNK_BATCH: usize = 20;
const DEFAULT_CHUNK_LOADING_BATCH: usize = 100;
const DEFAULT_CHUNK_SIZE: ChunkSize = ChunkSize::new(16);

pub struct ChunkLoaderPlugin {
    pub config: ChunkLoaderConfig,
}

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Chunk>()
            .init_resource::<ChunkLoader>()
            .init_asset_loader::<ChunkAssetLoader>()
            .insert_resource(self.config.clone())
            .insert_resource(Time::from_seconds(1. / 20.))
            .add_systems(
                FixedUpdate,
                (
                    ChunkLoader::read_chunks,
                    ChunkLoader::load_chunks,
                    ChunkLoader::spawn_chunks,
                    |loader: Res<ChunkLoader>, mut next_state: ResMut<NextState<AppState>>| {
                        if loader.unloaded_chunks.is_empty() && loader.rendering_chunks.is_empty() {
                            next_state.set(AppState::InGame);
                        }
                    },
                )
                    .chain()
                    .run_if(in_state(LoadingState::Chunks)),
            )
            .add_systems(
                FixedUpdate,
                (
                    ChunkLoader::read_chunks,
                    ChunkLoader::load_chunks,
                    ChunkLoader::spawn_chunks,
                    ChunkLoader::despawn_chunks,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

impl Default for ChunkLoaderPlugin {
    fn default() -> Self {
        Self {
            config: ChunkLoaderConfig {
                chunk_radius: DEFAULT_CHUNK_RADIUS,
                chunk_size: DEFAULT_CHUNK_SIZE,
                batch_size: DEFAULT_CHUNK_BATCH,
            },
        }
    }
}

#[derive(Clone, Resource)]
pub struct ChunkLoaderConfig {
    /// Number of chunks rendered around the camera in the x, y, z directions
    pub chunk_radius: usize,
    pub chunk_size: ChunkSize,
    pub batch_size: usize,
}

#[derive(Resource, Default, Debug)]
pub struct ChunkLoader {
    //TODO: Convert to some faster lookups
    unloaded_chunks: HashMap<ChunkPosition, Handle<Chunk>>,
    rendering_chunks: HashMap<ChunkPosition, Handle<Chunk>>,
    loaded_chunks: HashMap<ChunkPosition, Handle<Chunk>>,
}

impl ChunkLoader {
    pub fn unloaded_chunks(&self) -> usize {
        self.unloaded_chunks.len()
    }

    pub fn rendering_chunks(&self) -> usize {
        self.rendering_chunks.len()
    }

    pub fn loaded_chunks(&self) -> usize {
        self.loaded_chunks.len()
    }

    fn contains(&self, pos: &ChunkPosition) -> bool {
        self.unloaded_chunks.contains_key(pos)
            || self.rendering_chunks.contains_key(pos)
            || self.loaded_chunks.contains_key(pos)
    }

    pub fn read_chunks(
        camera: Query<&Transform, With<Camera>>,
        assets: Res<AssetServer>,
        config: Res<ChunkLoaderConfig>,
        mut loader: ResMut<ChunkLoader>,
    ) {
        let camera_loc = camera.single().unwrap().translation;
        let cur_chunk = config.chunk_size.chunk_coord(camera_loc);
        let mut insert_count = 0;
        for loc in cur_chunk.iter_around(config.chunk_radius as u64) {
            if !loader.contains(&loc) {
                insert_count += 1;
                loader.unloaded_chunks.insert(
                    loc,
                    assets.load(format!("chunks/{}_{}_{}.mcra", loc.x, loc.y, loc.z)),
                );
            }
        }
        if insert_count > 0 {
            info!("Loading Chunks: {insert_count}");
        }
    }

    pub fn load_chunks(
        mut loader: ResMut<ChunkLoader>,
        mut chunks: ResMut<Assets<Chunk>>,
        assets: Res<AssetServer>,
        config: Res<ChunkLoaderConfig>,
    ) {
        let mut new_chunks = Vec::new();
        loader.unloaded_chunks.retain(|loc, handle| {
            match assets.get_load_state(handle.id()) {
                None => {
                    if chunks.get(handle.id()).is_some() {
                        // Chunk is already loaded as an asset
                        return false;
                    }
                }
                Some(LoadState::Failed(_)) => {
                    // Chunk failed to load so we regenerate chunk
                    new_chunks.push((*loc, chunks.add(spawn_test_chunk(config.chunk_size, *loc))));
                    return false;
                }
                Some(LoadState::Loaded) => {
                    new_chunks.push((*loc, handle.clone()));
                    return false;
                }
                _ => {
                    // waiting to finish loading
                }
            }
            true
        });
        if !new_chunks.is_empty() {
            loader.rendering_chunks.extend(new_chunks);
        }
    }

    /// Spawn chunks that are in the `UnloadedChunk` state
    pub fn spawn_chunks(
        mut commands: Commands,
        mut loader: ResMut<ChunkLoader>,
        mut meshes: ResMut<Assets<Mesh>>,
        state: Res<State<AppState>>,
        textures: Res<BlockTextures>,
        config: Res<ChunkLoaderConfig>,
        chunks: Res<Assets<Chunk>>,
    ) {
        if loader.rendering_chunks.is_empty() {
            return;
        }
        let length = loader.rendering_chunks.len();
        let batch_size = if let AppState::Loading = state.get() {
            DEFAULT_CHUNK_LOADING_BATCH
        } else {
            config.batch_size
        }
        .min(length);
        let batch = loader
            .rendering_chunks
            .keys()
            .copied()
            .take(batch_size)
            .collect::<Vec<_>>();
        if batch.is_empty() {
            return;
        }

        let span = info_span!("chunk_spawning");
        span.in_scope(|| {
            info!("Rendering Chunks: {} / {}", batch_size, length,);
            let batch = batch
                .into_iter()
                .filter_map(|i| loader.rendering_chunks.remove(&i))
                .collect::<Vec<_>>();
            for new_chunk in batch {
                let chunk = chunks.get(new_chunk.id()).unwrap();
                loader.loaded_chunks.insert(chunk.loc, new_chunk.clone());
                commands.spawn((
                    ChunkComponent(new_chunk),
                    chunk.transform(),
                    MeshMaterial3d(textures.texture().unwrap().clone()),
                    Mesh3d(meshes.add(chunk.generate_mesh(&textures))),
                ));
            }
        });
    }

    pub fn despawn_chunks(
        mut commands: Commands,
        camera: Query<&Transform, With<Camera>>,
        components: Query<(Entity, &ChunkComponent)>,
        mut chunks: ResMut<Assets<Chunk>>,
        config: Res<ChunkLoaderConfig>,
        mut loader: ResMut<ChunkLoader>,
    ) {
        if components.is_empty() {
            return;
        }
        let camera_loc = camera.single().unwrap().translation;
        let cur_chunk = config.chunk_size.chunk_coord(camera_loc);
        let radius = config.chunk_radius as u64;
        let remove_chunks = components
            .iter()
            .filter_map(|(entity, chunk)| {
                let id = chunk.0.id();
                let chunk = chunks.get(id)?;

                cur_chunk
                    .outside_radius(chunk.loc, radius)
                    .then_some((entity, chunk.loc, id))
            })
            .collect::<Vec<_>>();
        if !remove_chunks.is_empty() {
            info!("Despawning Chunks: {}", remove_chunks.len());
        }
        for (entity, loc, id) in remove_chunks {
            //TODO: Save to disk
            loader.loaded_chunks.remove(&loc);
            commands.entity(entity).despawn();
            chunks.remove(id);
        }
    }
}
