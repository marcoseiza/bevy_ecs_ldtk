//! [Component]s and [Bundle]s used by the plugin.

pub use crate::ldtk::EntityInstance;
use crate::ldtk::{LayerInstance, Type};
use bevy::prelude::*;

use std::{
    collections::HashSet,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

#[allow(unused_imports)]
use crate::{
    assets::LdtkLevel,
    prelude::{LdtkEntity, LdtkIntCell},
    resources::{LevelSelection, LevelSpawnBehavior},
    utils::ldtk_grid_coords_to_grid_coords,
};

use bevy_ecs_tilemap::tiles::{TileBundle, TilePos};

/// [Component] added to any `IntGrid` tile by default.
///
/// When loading levels, you can flesh out `IntGrid` entities in your own system by querying for
/// `Added<IntGridCell>`.
/// Or, you can hook into the entity's spawning process using [LdtkIntCell].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct IntGridCell {
    pub value: i32,
}

/// [Component] that determines the desired levels to be loaded for an [LdtkWorldBundle].
///
/// There is an abstraction for this in the form of the [LevelSelection] resource.
/// This component does not respond to the
/// [LevelSpawnBehavior::UseWorldTranslation::load_level_neighbors] option at all, while the
/// [LevelSelection] does.
/// If a [LevelSelection] is inserted, the plugin will update this component based off its value.
/// If not, [LevelSet] allows you to have more direct control over the levels you spawn.
///
/// Changes to this component are idempotent, so levels won't be respawned greedily.
#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct LevelSet {
    pub iids: HashSet<String>,
}

/// [Component] that indicates that an ldtk entity should be a child of the world, not the level.
///
/// By default, [LdtkEntity]s are children of the level they spawn in.
/// This can be a problem if that entity is supposed to travel across multiple levels, since they
/// will despawn the moment the level they were born in despawns.
///
/// This component makes them children of the [LdtkWorldBundle] (after one update),
/// so they can traverse levels without despawning.
/// Furthermore, this component prevents respawns of the same entity if the level they were born in
/// despawns/respawns.
/// For this purpose, it uses the `iid` stored in this component to uniquely identify ldtk
/// entities.
///
/// Implements [LdtkEntity], and can be added to an [LdtkEntity] bundle with the `#[worldly]` field
/// attribute. See [LdtkEntity#worldly] for more details.
#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct Worldly {
    pub entity_iid: String,
}

impl Worldly {
    /// Creates a [Worldly] from the entity information available to the
    /// [LdtkEntity::bundle_entity] method.
    ///
    /// Used for the `#[worldly]` attribute macro for `#[derive(LdtkEntity)]`.
    /// See [LdtkEntity#worldly] for more info.
    pub fn from_entity_info(entity_instance: &EntityInstance) -> Worldly {
        Worldly {
            entity_iid: entity_instance.iid.clone(),
        }
    }
}

/// [Component] that stores grid-based coordinate information.
///
/// For Tile, AutoTile, and IntGrid layers, all tiles have this component by default.
///
/// Can be added to an [LdtkEntity] bundle with the `#[grid_coords]` attribute.
/// Then, it will be spawned with the initial grid-based position of the entity in LDtk.
/// See [LdtkEntity#grid_coords] for attribute macro usage.
///
/// Note that the plugin will not automatically update the entity's [Transform] when this component
/// is updated, nor visa versa.
/// This is left up to the user since there are plenty of scenarios where this behavior needs to be
/// custom.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct GridCoords {
    pub x: i32,
    pub y: i32,
}

impl From<IVec2> for GridCoords {
    fn from(i_vec_2: IVec2) -> Self {
        GridCoords {
            x: i_vec_2.x,
            y: i_vec_2.y,
        }
    }
}

impl From<GridCoords> for IVec2 {
    fn from(grid_coords: GridCoords) -> Self {
        IVec2::new(grid_coords.x, grid_coords.y)
    }
}

impl From<TilePos> for GridCoords {
    fn from(tile_pos: TilePos) -> Self {
        GridCoords {
            x: tile_pos.x as i32,
            y: tile_pos.y as i32,
        }
    }
}

impl From<GridCoords> for TilePos {
    fn from(grid_coords: GridCoords) -> Self {
        TilePos::new(grid_coords.x as u32, grid_coords.y as u32)
    }
}

impl Add<GridCoords> for GridCoords {
    type Output = GridCoords;
    fn add(self, rhs: GridCoords) -> Self::Output {
        GridCoords {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<GridCoords> for GridCoords {
    fn add_assign(&mut self, rhs: GridCoords) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<GridCoords> for GridCoords {
    type Output = GridCoords;
    fn sub(self, rhs: GridCoords) -> Self::Output {
        GridCoords {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign<GridCoords> for GridCoords {
    fn sub_assign(&mut self, rhs: GridCoords) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<GridCoords> for GridCoords {
    type Output = GridCoords;
    fn mul(self, rhs: GridCoords) -> Self::Output {
        GridCoords {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign<GridCoords> for GridCoords {
    fn mul_assign(&mut self, rhs: GridCoords) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl GridCoords {
    pub fn new(x: i32, y: i32) -> GridCoords {
        GridCoords { x, y }
    }

    /// Creates a [GridCoords] from the entity information available to the
    /// [LdtkEntity::bundle_entity] method.
    ///
    /// Used for the `#[grid_coords]` attribute macro for `#[derive(LdtkEntity)]`.
    /// See [LdtkEntity#grid_coords] for more info.
    pub fn from_entity_info(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
    ) -> GridCoords {
        ldtk_grid_coords_to_grid_coords(entity_instance.grid, layer_instance.c_hei)
    }
}

/// [Component] for storing user-defined custom data for a paticular tile in an LDtk tileset
/// definition.
///
/// Automatically inserted on any tiles with metadata.
#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct TileMetadata {
    pub data: String,
}

/// [Component] for storing user-defined, enum-based tags for a particular tile in an LDtk tileset
/// definition.
///
/// Automatically inserted on any tiles with enum tags.
#[derive(Clone, Eq, PartialEq, Debug, Default, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct TileEnumTags {
    pub tags: Vec<String>,
    pub source_enum_uid: Option<i32>,
}

/// [Component] for storing some LDtk layer information on layer entities.
///
/// Based on [LayerInstance], but without the fields with tile and entity information.
///
/// Automatically inserted for IntGrid, AutoTile, and Tile layers.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct LayerMetadata {
    /// Grid-based height
    pub c_hei: i32,

    /// Grid-based width
    pub c_wid: i32,

    /// Grid size
    pub grid_size: i32,

    /// Layer definition identifier
    pub identifier: String,

    /// Layer opacity as Float [0-1]
    pub opacity: f32,

    /// Total layer X pixel offset, including both instance and definition offsets.
    pub px_total_offset_x: i32,

    /// Total layer Y pixel offset, including both instance and definition offsets.
    pub px_total_offset_y: i32,

    /// The definition UID of corresponding Tileset, if any.
    pub tileset_def_uid: Option<i32>,

    /// The relative path to corresponding Tileset, if any.
    pub tileset_rel_path: Option<String>,

    /// Layer type (possible values: IntGrid, Entities, Tiles or AutoLayer)
    pub layer_instance_type: Type,

    /// Unique layer instance identifier
    pub iid: String,

    /// Reference the Layer definition UID
    pub layer_def_uid: i32,

    /// Reference to the UID of the level containing this layer instance
    pub level_id: i32,

    /// An Array containing the UIDs of optional rules that were enabled in this specific layer
    /// instance.
    pub optional_rules: Vec<i32>,

    /// This layer can use another tileset by overriding the tileset UID here.
    pub override_tileset_uid: Option<i32>,

    /// X offset in pixels to render this layer, usually 0 (IMPORTANT: this should be added to
    /// the `LayerDef` optional offset, see `__pxTotalOffsetX`)
    pub px_offset_x: i32,

    /// Y offset in pixels to render this layer, usually 0 (IMPORTANT: this should be added to
    /// the `LayerDef` optional offset, see `__pxTotalOffsetY`)
    pub px_offset_y: i32,

    /// Random seed used for Auto-Layers rendering
    pub seed: i32,

    /// Layer instance visibility
    pub visible: bool,
}

impl From<&LayerInstance> for LayerMetadata {
    fn from(instance: &LayerInstance) -> Self {
        LayerMetadata {
            c_hei: instance.c_hei,
            c_wid: instance.c_wid,
            grid_size: instance.grid_size,
            identifier: instance.identifier.clone(),
            opacity: instance.opacity,
            px_total_offset_x: instance.px_total_offset_x,
            px_total_offset_y: instance.px_total_offset_y,
            tileset_def_uid: instance.tileset_def_uid,
            tileset_rel_path: instance.tileset_rel_path.clone(),
            layer_instance_type: instance.layer_instance_type.clone(),
            iid: instance.iid.clone(),
            layer_def_uid: instance.layer_def_uid,
            level_id: instance.level_id,
            optional_rules: instance.optional_rules.clone(),
            override_tileset_uid: instance.override_tileset_uid,
            px_offset_x: instance.px_offset_x,
            px_offset_y: instance.px_offset_y,
            seed: instance.seed,
            visible: instance.visible,
        }
    }
}

/// [Component] that indicates that an LDtk level or world should respawn.
///
/// Inserting this component on an entity with either `Handle<LdtkAsset>` or `Handle<LdtkLevel>`
/// components will cause it to respawn.
/// This can be used to implement a simple level-restart feature.
/// Internally, this is used to support the entire level spawning process
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component, Reflect)]
#[reflect(Component)]
pub struct Respawn;

#[derive(Copy, Clone, Debug, Default, Bundle)]
pub(crate) struct TileGridBundle {
    #[bundle]
    pub tile_bundle: TileBundle,
    pub grid_coords: GridCoords,
}

#[derive(Clone, Default, Bundle)]
pub(crate) struct IntGridCellBundle {
    pub int_grid_cell: IntGridCell,
}

#[derive(Clone, Bundle, Default)]
pub(crate) struct EntityInstanceBundle {
    pub entity_instance: EntityInstance,
}

/// [Bundle] for spawning LDtk worlds and their levels. The main bundle for using this plugin.
///
/// After the ldtk file is done loading, the levels you've chosen with [LevelSelection] or
/// [LevelSet] will begin to spawn.
/// Each level is its own entity, with the [LdtkWorldBundle] as its parent.
/// Each level has a `Handle<LdtkLevel>` component.
///
/// All non-Entity layers (IntGrid, Tile, and AutoTile) will also spawn as their own entities.
/// Each layer's parent will be the level entity.
/// Each layer will have a [LayerMetadata] component, and are [bevy_ecs_tilemap::TilemapBundle]s.
/// Each tile in these layers will have the layer entity as its parent.
///
/// For Entity layers, all LDtk entities in the level are spawned as children to the level entity,
/// unless marked by a [Worldly] component.
#[derive(Clone, Default, Bundle)]
pub struct LdtkWorldBundle {
    pub ldtk_handle: Handle<crate::assets::LdtkAsset>,
    pub level_set: LevelSet,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}
