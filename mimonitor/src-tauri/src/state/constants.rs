// Picture mode IDs
pub const MODE_STANDARD: i32 = 14;
pub const MODE_GAME: i32 = 10;
pub const MODE_MOVIE: i32 = 9;

// Picture mode groups (all scene IDs that map to each mode)
pub const STANDARD_SCENES: &[i32] = &[14, 64, 65, 66, 67, 68];
pub const GAME_SCENES: &[i32] = &[10, 25, 26, 27, 28, 29];
pub const MOVIE_SCENES: &[i32] = &[9];

// Xiaomi to MTK color temperature mapping
pub const XIAOMI_TO_MTK_COLOR_TEMP: &[(i32, i32)] = &[
    (0, 1), // Cold
    (1, 2), // Standard
    (2, 3), // Warm
    (3, 0), // Custom
    (4, 4),
    (5, 5),
    (8, 6), // Native
];

pub const MTK_TO_XIAOMI_COLOR_TEMP: &[(i32, i32)] = &[
    (1, 0), (2, 1), (3, 2), (0, 3), (4, 4), (5, 5), (6, 8),
];

pub const CUSTOM_COLOR_TEMP_VALUE: i32 = 3;

// Input source IDs
pub const SOURCE_HDMI1: i32 = 23;
pub const SOURCE_HDMI2: i32 = 24;
pub const SOURCE_DP: i32 = 29;
pub const SOURCE_USBC: i32 = 30;

// JNI keys
pub const JNI_BACKLIGHT: &str = "g_disp__disp_back_light";
pub const JNI_CLR_TEMP: &str = "g_video__clr_temp";
pub const JNI_CLR_GAIN_R: &str = "g_video__clr_gain_r";
pub const JNI_CLR_GAIN_G: &str = "g_video__clr_gain_g";
pub const JNI_CLR_GAIN_B: &str = "g_video__clr_gain_b";
pub const JNI_LOCAL_DIMMING: &str = "g_video__vid_local_dimming";
pub const JNI_GAMUT: &str = "g_video__vid_gamut_mapping_mode";
pub const JNI_RESPONSE_TIME: &str = "g_video__vid_od_response_time";
pub const JNI_INSERT_BLACK: &str = "g_video__vid_insert_black";
pub const JNI_FREESYNC_DP: &str = "g_video__dp_adaptive_sync";
pub const JNI_FREESYNC_HDMI: &str = "g_video__freesync_switch";
pub const JNI_EDID_DP: &str = "g_fusion_picture__dp_edid_version";
pub const JNI_EDID_HDMI: &str = "g_fusion_picture__hdmi_edid_version";
pub const JNI_RESET: &str = "g_fusion_picture__pic_reset_def_bypicmode";

// Android settings keys
pub const SET_PICTURE_MODE: &str = "picture_mode";
pub const SET_BACKLIGHT: &str = "picture_backlight";
pub const SET_BACKLIGHT_XIAOMI: &str = "xiaomi_picture_backlight";
pub const SET_BRIGHTNESS: &str = "picture_brightness";
pub const SET_CONTRAST: &str = "picture_contrast";
pub const SET_SATURATION: &str = "picture_saturation";
pub const SET_HUE: &str = "picture_hue";
pub const SET_SHARPNESS: &str = "picture_sharpness";
pub const SET_COLOR_TEMP: &str = "picture_color_temperature";
pub const SET_RED_GAIN: &str = "picture_red_gain";
pub const SET_GREEN_GAIN: &str = "picture_green_gain";
pub const SET_BLUE_GAIN: &str = "picture_blue_gain";
pub const SET_LOCAL_DIMMING: &str = "picture_local_dimming";
pub const SET_LOCAL_DIMMING_TV: &str = "tv_picture_video_local_dimming";
pub const SET_DYNAMIC_DEF: &str = "picture_dynamic_definition";
pub const SET_RESPONSE_TIME: &str = "picture_response_time";
pub const SET_COLOR_SPACE: &str = "tv_picture_advanced_video_color_space";
pub const SET_COLOR_SPACE_TV: &str = "tv_picture_video_color_space";
pub const SET_SOURCE: &str = "mitv.tvplayer.hdmi.last.source";

// Game settings
pub const SET_CROSSHAIR: &str = "front_sight_index";
pub const SET_DYNAMIC_CROSSHAIR: &str = "mt_game_dynamic_ft";
pub const SET_SNIPER_SCOPE: &str = "mt_game_scope";
pub const SET_SCOPE_NIGHT: &str = "mt_game_scope_night";
pub const SET_FPS_COUNTER: &str = "monitor_menu_fps_counter";
pub const SET_STOPWATCH: &str = "monitor_menu_stopwatch";
pub const SET_TIMER: &str = "monitor_menu_timer";

// Light settings
pub const SET_LED_MODE: &str = "atmosphere_light_switcher_pm2";
pub const SET_LED_BRIGHTNESS: &str = "atmosphere_light_illumination";
pub const SET_LED_COLOR_TEMP: &str = "atmosphere_light_color_temp";
pub const SET_LED_COLOR_VALUE: &str = "atmosphere_light_color_value";

// Remote key codes
pub const KEY_POWER: &str = "KEYCODE_POWER";
pub const KEY_HOME: &str = "KEYCODE_HOME";
pub const KEY_MENU: &str = "KEYCODE_MENU";
pub const KEY_BACK: &str = "KEYCODE_BACK";
pub const KEY_DPAD_UP: &str = "KEYCODE_DPAD_UP";
pub const KEY_DPAD_DOWN: &str = "KEYCODE_DPAD_DOWN";
pub const KEY_DPAD_LEFT: &str = "KEYCODE_DPAD_LEFT";
pub const KEY_DPAD_RIGHT: &str = "KEYCODE_DPAD_RIGHT";
pub const KEY_DPAD_CENTER: &str = "KEYCODE_DPAD_CENTER";
pub const KEY_VOLUME_UP: &str = "KEYCODE_VOLUME_UP";
pub const KEY_VOLUME_DOWN: &str = "KEYCODE_VOLUME_DOWN";
pub const KEY_MUTE: &str = "KEYCODE_VOLUME_MUTE";
