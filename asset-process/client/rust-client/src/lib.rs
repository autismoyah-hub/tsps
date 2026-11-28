//! Native revision 239.1 client components; see PARITY_GAPS.md for remaining coverage.
pub mod date_of_birth;
pub mod script_date;
pub mod remote_json;
pub mod remote_request;
pub mod remote_registry;
pub mod remote_url;
pub mod remote_widget;
pub mod ambient_sound;
pub mod ambient_runtime;
pub mod world_list;
pub mod world_http;
pub mod mouse_record;
pub mod mouse_worker;
pub mod mouse_click;
pub mod inventory_def;
pub mod image_decode;
pub mod context_menu;
pub mod title;
pub mod title_input;
pub mod login_response;
pub mod buffer;
pub mod cache;
pub mod cache_recovery;
pub mod collision;
pub mod route;
pub mod rsa;
pub mod login;
pub mod minimap;
pub mod model;
pub mod model_light;
pub mod model_draw;
pub mod model_bounds;
pub mod model_near_clip;
pub mod model_merge;
pub mod object_model;
pub mod tile_mesh;
pub mod render3d;
pub mod world_mesh;
pub mod viewport;
pub mod viewport_shape;
pub mod animation;
pub mod audio_envelope;
pub mod audio_filter;
pub mod audio_synth;
pub mod audio_runtime;
pub mod audio_vorbis;
pub mod audio_output;
pub mod deform;
pub mod curve;
pub mod cached_animation;
pub mod skeletal;
pub mod animation_runtime;
pub mod object;
pub mod item;
pub mod item_search;
pub mod item_model;
pub mod item_icon;
pub mod compression;
pub mod crypto;
pub mod disk_cache;
pub mod huffman;
pub mod game_packet;
pub mod group_state;
pub mod skybox;
pub mod floor;
pub mod font;
pub mod widget;
pub mod widget_layout;
pub mod interface_packet;
pub mod interface_state;
pub mod interface_render;
pub mod interface_model;
pub mod interface_events;
pub mod interface_action;
pub mod interface_script;
pub mod client_state;
pub mod client_preferences;
pub mod device_options;
mod volume_tables;
pub mod enum_def;
pub mod param_def;
pub mod script_data;
pub mod script_string;
pub mod script_item;
pub mod script_event;
pub mod script_logout;
pub mod inventory;
pub mod input;
pub mod windows_keys;
pub mod js5;
pub mod protocol;
pub mod npc;
pub mod npc_model;
pub mod npc_info;
pub mod actor_update;
pub mod identity_kit;
pub mod player_colors;
pub mod player_model;
pub mod player_appearance;
pub mod script_npc;
pub mod interface_animation;
pub mod cs1;
pub mod protocol239;
pub mod raster;
pub mod scene;
pub mod scene_walk;
pub mod click_cross;
pub mod rebuild;
pub mod script;
pub mod script_math;
pub mod script_runtime;
pub mod session;
pub mod sprite;
pub mod sprite_pixels;
pub mod terrain;
pub mod texture;
pub mod writer;
pub mod vm;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Io(String),
    PendingAsset { archive: u8, group: u32 },
    Truncated,
    Invalid(&'static str),
    Unsupported(&'static str),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub mod player_update;
pub mod player_info;
pub mod live_runtime;
pub mod world_actors;
pub mod world_action;
pub mod script_params;
pub mod string_compare;
pub mod live_smoke;
pub mod chat;

pub mod database;

pub mod database_defs;

pub mod script_array;

pub mod canvas_transform;
pub mod window_layout;
pub mod username;
pub mod social;
pub mod clan_settings;
pub mod clan_channel;

pub mod clan_delta;

pub mod clan_profile;

pub mod chat_packet;

pub mod map_element;
pub mod hint;
pub mod zone;

pub mod ground_model;

pub mod location_update;
pub mod location_animation;
pub mod location_merge;

pub mod area_sound;

pub mod actor_sequence;

pub mod actor_path;

pub mod sequence_sound;

pub mod actor_freeze;

pub mod actor_exact;

pub mod actor_turn;
pub mod actor_target;

pub mod actor_pose;

pub mod spot_animation;

pub mod actor_spot;

pub mod world_spot;

pub mod projectile;
pub mod scripted_projectile;

pub mod actor_overlay;

pub mod overlay_render;

pub mod overhead;

pub mod head_overlay;
pub mod friends_chat;
pub mod social_sort;
pub mod java_sort;

pub mod script_array_storage;
pub mod script_array_ops;

pub mod client_metrics;
pub mod saved_preferences;
pub mod clipboard;
pub mod saved_varcs;
pub mod camera;
pub mod camera_tween;

pub mod menu_settings;

pub mod entity_ops;

pub mod spell_action;

pub mod widget_drag;
pub mod world_entity;
pub mod npc_runtime;

pub mod title_fire;

pub mod music_track;

pub mod midi;

pub mod music_patch;
pub mod music_channel;
pub mod music_voice;
pub mod audio_stream;
pub mod music_samples;
pub mod music_synth;
pub mod music_player;
pub mod music_transition;
pub mod music_runtime;
pub mod music_request;
pub mod market;
pub mod interaction_policy;
pub mod steering;
pub mod steering_scene;

pub mod player_runtime;

pub mod world_view;
pub mod actor_visual;
pub mod world_audio;
pub mod projectile_world;
pub mod world_overlap;
pub mod world_pick;

pub mod game_clock;
pub mod roof;
pub mod camera_focus;
pub mod audio_pump;
#[cfg(windows)] pub mod audio_worker;

pub mod appearance_preview;

pub mod widget_line;

pub mod widget_appearance;
pub mod widget_tooltip;

pub mod scene_slots;

pub mod player_preview;

pub mod widget_text;

pub mod font_icons;

pub mod widget_key_modes;

pub mod editable_text_layout;

pub mod editable_text_raster;

pub mod editable_text;

pub mod widget_editor;

pub mod native_editor_probe;

pub mod terrain_light;

pub mod scene_light;

pub mod model_contour;

pub mod scene_normal;

pub mod player_model_cache;

pub mod viewport_steering;

pub mod model_pick;

pub mod matrix_pick;
pub mod model_alpha;
pub mod flat_raster;
pub mod gouraud_raster;
pub mod texture_span;
pub mod texture_raster;

pub mod terrain_texture_mapping;

pub mod legacy_raster;
pub mod model_order;

pub mod world_select;

pub mod keyboard_report;

pub mod keyboard_input;

pub mod auxiliary_input;

pub mod heatmap;
pub mod map_area;
pub mod map_overview_flash;
pub mod map_hit;
pub mod map_hover;
pub mod map_event;
pub mod map_menu;
pub mod map_pointer;
pub mod map_local;
pub mod world_map;
pub mod map_composite;
pub mod map_geography;
pub mod map_mask;
pub mod map_color;
pub mod map_tile;
pub mod map_floor;
pub mod map_decoration;
pub mod map_scale;
pub mod map_view;

pub mod map_lookup;
pub mod map_settings;
pub mod map_icon;
pub mod map_label;

pub mod map_flash;
pub mod map_symbols;

pub mod map_locations;

pub mod map_heatmap;

pub mod map_index;

pub mod scene_zone;
pub mod scene_visibility;
pub mod terrain_submission;
pub mod object_submission;
pub mod object_scene_order;
pub mod scene_temp_order;
pub mod scene_footprint;

pub mod npc_submission;

pub mod player_submission;

pub mod actor_submission;

pub mod map_loading;
pub mod widget_scope;

pub mod widget_shortcuts;

pub mod world_entity_limit;

pub mod actor_height;

pub mod animation_readiness;

pub mod widget_query;

pub mod interface_mounts;

pub mod dialog_resume;

pub mod named_player_action;

pub mod script_drag;

pub mod world_pointer;

pub mod interface_menu_order;

pub mod menu_sort;

pub mod menu_limits;

pub mod menu_label;

pub mod menu_priority;

pub mod world_entity_menu;

pub mod scene_menu;

pub mod actor_menu;

pub mod ground_menu;

pub mod object_menu;

mod widget_selection;

pub mod widget_navigation;

pub mod widget_delete;

mod widget_geometry;
pub mod widget_resize;
pub mod widget_deferred;
pub mod widget_argument_operation;

#[cfg(test)]
mod widget_open_geometry_tests;

#[cfg(test)]
mod widget_move_geometry_tests;

#[cfg(test)]
mod widget_root_geometry_tests;

#[cfg(test)]
mod widget_position_packet_tests;

#[cfg(test)]
mod widget_automatic_resize_tests;

pub mod uid192;

pub mod remote_crm;
pub mod remote_card;

pub mod remote_input;

pub mod packet_failure;

pub mod outbound;
pub mod outgoing_batch;

pub mod token_auth;

pub mod keepalive;
pub mod receive_timeout;

pub mod world_scene_instances;

pub mod object_traversal;

pub mod scene_object_layout;

pub mod decoration_offset;
pub mod location_draw_position;

pub mod item_scene;
pub mod item_submission;

pub mod terrain_scene;
pub mod scene_raster_order;

#[cfg(feature = "custom-assets")]
pub mod asset_package;

#[cfg(all(windows, not(feature = "offscreen")))]
pub mod presentation;

#[cfg(feature = "custom-assets")]
pub mod authored_floor;

#[cfg(feature = "custom-assets")]
pub mod environment_style;

#[cfg(feature = "custom-assets")]
pub mod environment_cutaway;

#[cfg(feature = "custom-assets")]
pub mod environment_minimap;
#[cfg(feature = "custom-assets")]
pub mod environment_motion;

#[cfg(feature = "custom-assets")]
pub mod authored_content;

#[cfg(feature = "private-test-adapter")]
mod adapter_action;
mod pick_probe;
#[cfg(feature = "private-test-adapter")]
pub mod private_adapter;
/// #181: passive client journal, wired only into the private test adapter path.
#[cfg(feature = "private-test-adapter")]
pub mod client_journal;
#[cfg(feature = "private-test-adapter")]
pub mod draw_custody;
#[cfg(feature = "private-test-adapter")]
pub mod location_animation_receipt;
#[cfg(feature = "private-test-adapter")]
pub mod location_animation_draw;
#[cfg(all(feature = "offscreen", not(feature = "private-test-adapter")))]
compile_error!("feature offscreen requires private-test-adapter");
#[cfg(all(feature = "offscreen", any(feature = "enhanced-renderer", feature = "native-profiling")))]
compile_error!("feature offscreen excludes enhanced-renderer and native-profiling (window-only D3D, presentation and tracing paths)");
#[cfg(all(test, feature = "offscreen"))]
mod offscreen_tests;
#[cfg(feature = "offscreen")]
pub mod offscreen_cli;

#[cfg(feature="native-profiling")]
pub mod ui_profile;
#[cfg(feature="native-profiling")]
pub mod benchmark_run;
#[cfg(feature="native-profiling")]
pub mod benchmark_scene;
#[cfg(feature="native-profiling")]
pub mod study_profile;
#[cfg(feature="native-profiling")]
pub mod presentation_trace;
#[cfg(all(feature="native-profiling",feature="enhanced-renderer"))]
pub mod enhanced_trace;

pub(crate) mod native_smoke;

#[cfg(feature = "enhanced-renderer")]
pub mod enhanced;
mod layout_cache;
